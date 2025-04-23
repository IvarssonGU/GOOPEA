use std::{collections::HashSet, path::Path};

pub fn preprocess<P>(p: P) -> String
where
    P: AsRef<Path>,
{
    let mut defs = HashSet::new();
    _preprocess(p, &mut defs)
}

fn _preprocess<P>(p: P, defs: &mut HashSet<String>) -> String
where
    P: AsRef<Path>,
{
    let folder = p.as_ref().parent().unwrap();
    let file = std::fs::read_to_string(&p).unwrap();
    let mut out = Vec::new();
    let mut lines = file.lines();
    while let Some(line) = lines.next() {
        let words: Vec<&str> = line.split_whitespace().collect();
        match words[..] {
            ["#include", a] => out.push(_preprocess(folder.join(a), defs)),
            ["#include", ..] => panic!("Invalid include: {}", line),
            ["#def", a] => {
                defs.insert(a.to_string());
            }
            ["#def", ..] => panic!("Invalid def: {}", line),
            ["#if", a] => {
                if !defs.contains(a) {
                    while let Some(line) = lines.next() {
                        let words: Vec<&str> = line.split_whitespace().collect();
                        match words[..] {
                            ["#endif", b] => {
                                if a == b {
                                    break;
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }
            ["#ifnot", a] => {
                if defs.contains(a) {
                    while let Some(line) = lines.next() {
                        let words: Vec<&str> = line.split_whitespace().collect();
                        match words[..] {
                            ["#endif", b] => {
                                if a == b {
                                    break;
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }
            ["#endif", ..] => (),
            ["#undef", a] => {
                defs.remove(a);
            }
            ["#ifnot", ..] => panic!("Invalid if: {}", line),
            _ => out.push(line.to_string()),
        }
    }

    out.join("\n")
}
