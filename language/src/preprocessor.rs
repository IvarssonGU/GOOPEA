use std::path::Path;

pub fn preprocess<P>(p: P) -> String
where
    P: AsRef<Path>,
{
    let folder = p.as_ref().parent().unwrap();
    let file = std::fs::read_to_string(&p).unwrap();
    let mut out = Vec::new();
    for line in file.lines() {
        let words: Vec<&str> = line.split_whitespace().collect();
        match words[..] {
            ["#include", a] => out.push(preprocess(folder.join(a))),
            ["#include", ..] => panic!("Invalid include: {}", line),
            _ => out.push(line.to_string()),
        }
    }

    out.join("\n")
}
