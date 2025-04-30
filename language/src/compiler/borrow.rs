use std::collections::{HashMap, HashSet};

use crate::compiler::stir::{Body, Constant, Exp, Stir, Var};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Status {
    Owned,
    Borrowed,
}

pub fn get_ownership(prog: &Stir) -> HashMap<Constant, Vec<Status>> {
    let mut map = HashMap::new();
    for func in prog {
        map.insert(func.id.clone(), vec![Status::Borrowed; func.args.len()]);
    }
    let mut change = true;
    while change {
        change = false;
        let mut new_map = map.clone();
        for func in prog {
            let vars = collect(&func.body, &new_map);
            for (i, arg) in func.args.iter().enumerate() {
                if vars.contains(arg) {
                    let mut ownership = new_map.get(&func.id).unwrap().clone();
                    ownership[i] = Status::Owned;
                    new_map.insert(func.id.clone(), ownership.clone());
                }
            }
        }
        if map != new_map {
            map = new_map;
            change = true;
        }
    }
    map
}

fn collect(body: &Body, map: &HashMap<Constant, Vec<Status>>) -> HashSet<Var> {
    match body {
        Body::Let(var, e, next) => match e {
            Exp::Int(_) => collect(next, map),
            Exp::Ctor(_, _) => collect(next, map),
            Exp::Reset(var) => {
                let mut set = collect(next, map);
                set.insert(var.clone());
                set
            }
            Exp::Reuse(_, _, _) => collect(next, map),
            Exp::App(fid, args) => {
                let mut set = collect(next, map);
                if let Some(statuses) = map.get(fid) {
                    for i in 0..args.len() {
                        match statuses[i] {
                            Status::Owned => {
                                set.insert(args[i].clone());
                            }
                            Status::Borrowed => (),
                        }
                    }
                }
                set
            }
            Exp::Op(_, _, _) => collect(next, map),
            Exp::Proj(_, v) => {
                let mut set = collect(next, map);
                if set.contains(var) {
                    set.insert(v.clone());
                }
                set
            }
            Exp::UTuple(_) => collect(next, map),
        },
        Body::Ret(_) => HashSet::new(),
        Body::Match(_, branches) => {
            let mut combined = HashSet::new();
            for (_, branch) in branches {
                combined = combined.union(&collect(branch, map)).cloned().collect();
            }
            combined
        }
        _ => panic!("Does not exist at this stage "),
    }
}
