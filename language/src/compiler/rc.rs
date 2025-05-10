use crate::compiler::borrow::{Status, get_ownership};
use crate::compiler::crux::Type;
use crate::compiler::stir::{Body, Constant, Exp, Function, Stir, Var, free_vars};
use std::collections::{HashMap, HashSet};

pub fn add_rc(prog: &Stir, flag: bool) -> Stir {
    let ownership = get_ownership(prog);
    if flag {
        prog.iter()
            .map(|func| insert_rc_fun(func, &ownership))
            .collect()
    } else {
        let mut ownership = HashMap::new();
        for func in prog {
            ownership.insert(func.id.clone(), vec![Status::Owned; func.args.len()]);
        }
        prog.iter()
            .map(|func| insert_rc_fun(func, &ownership))
            .collect()
    }
}

fn insert_rc_fun(func: &Function, beta_map: &HashMap<Constant, Vec<Status>>) -> Function {
    let mut betal = HashMap::new();
    for (i, status) in beta_map.get(&func.id).unwrap().iter().enumerate() {
        betal.insert(func.args[i].clone(), *status);
    }
    Function {
        fip: func.fip,
        id: func.id.clone(),
        typ: func.typ.clone(),
        args: func.args.clone(),
        body: owned_minus_all(
            func.args.clone(),
            &insert_rc_body(&func.body, &betal, beta_map),
            &betal,
        ),
    }
}

fn insert_rc_body(
    body: &Body,
    betal: &HashMap<Var, Status>,
    beta_map: &HashMap<Constant, Vec<Status>>,
) -> Body {
    match body {
        Body::Ret(var) => owned_plus(var.clone(), HashSet::new(), body, betal),
        Body::Match(var, branches) => {
            let vars = Vec::from_iter(free_vars(body).iter().cloned());
            Body::Match(
                var.clone(),
                branches
                    .iter()
                    .map(|(i, branch)| {
                        (
                            *i,
                            owned_minus_all(
                                vars.clone(),
                                &insert_rc_body(branch, betal, beta_map),
                                betal,
                            ),
                        )
                    })
                    .collect(),
            )
        }
        Body::Let(var, exp, next) => match exp {
            Exp::Proj(_, proj_var)
                if default_betal(proj_var, betal) == Status::Owned && var.1 == Type::Heaped =>
            {
                Body::Let(
                    var.clone(),
                    exp.clone(),
                    Body::Inc(
                        var.clone(),
                        owned_minus(proj_var, &insert_rc_body(next, betal, beta_map), betal).into(),
                    )
                    .into(),
                )
            }
            Exp::Proj(_, proj_var)
                if default_betal(proj_var, betal) == Status::Borrowed || var.1 != Type::Heaped =>
            {
                let mut new_betal = betal.clone();
                new_betal.insert(var.clone(), Status::Borrowed);
                Body::Let(
                    var.clone(),
                    exp.clone(),
                    insert_rc_body(next, &new_betal, beta_map).into(),
                )
            }
            Exp::Reset(_) => Body::Let(
                var.clone(),
                exp.clone(),
                insert_rc_body(next, betal, beta_map).into(),
            ),
            Exp::App(fid, args) => cappy(
                args.clone(),
                beta_map.get(fid).unwrap().clone(),
                &Body::Let(
                    var.clone(),
                    exp.clone(),
                    insert_rc_body(next, betal, beta_map).into(),
                ),
                betal,
            ),
            Exp::Ctor(_, args) => cappy(
                args.clone(),
                vec![Status::Owned; args.len()],
                &Body::Let(
                    var.clone(),
                    exp.clone(),
                    insert_rc_body(next, betal, beta_map).into(),
                ),
                betal,
            ),
            Exp::UTuple(args) => cappy(
                args.clone(),
                vec![Status::Owned; args.len()],
                &Body::Let(
                    var.clone(),
                    exp.clone(),
                    insert_rc_body(next, betal, beta_map).into(),
                ),
                betal,
            ),
            Exp::Reuse(_, _, args) => cappy(
                args.clone(),
                vec![Status::Owned; args.len()],
                &Body::Let(
                    var.clone(),
                    exp.clone(),
                    insert_rc_body(next, betal, beta_map).into(),
                ),
                betal,
            ),
            Exp::Int(_) => Body::Let(
                var.clone(),
                exp.clone(),
                insert_rc_body(next, betal, beta_map).into(),
            ),
            Exp::Op(_, _, _) => Body::Let(
                var.clone(),
                exp.clone(),
                insert_rc_body(next, betal, beta_map).into(),
            ),
            _ => panic!("Shouldn't be possible!"),
        },
        _ => todo!(),
    }
}

fn cappy(
    mut vars: Vec<Var>,
    mut stats: Vec<Status>,
    body: &Body,
    betal: &HashMap<Var, Status>,
) -> Body {
    let Body::Let(var, exp, next) = body else {
        panic!("Expected a Let body");
    };
    if vars.is_empty() {
        body.clone()
    } else {
        let top = vars.pop().unwrap();
        let new_vars = vars.clone();
        let top_status = stats.pop().unwrap();
        let new_stats = stats.clone();
        if top_status == Status::Owned {
            let mut live_vars = free_vars(next);
            live_vars.extend(new_vars.clone());
            owned_plus(
                top.clone(),
                live_vars,
                &cappy(new_vars, new_stats, body, betal),
                betal,
            )
        } else {
            cappy(
                new_vars,
                new_stats,
                &Body::Let(
                    var.clone(),
                    exp.clone(),
                    owned_minus(&top, next, betal).into(),
                ),
                betal,
            )
        }
    }
}

fn default_betal(var: &Var, map: &HashMap<Var, Status>) -> Status {
    if let Some(status) = map.get(var) {
        *status
    } else {
        Status::Owned
    }
}

fn owned_plus(
    var: Var,
    live_vars: HashSet<Var>,
    body: &Body,
    betal: &HashMap<Var, Status>,
) -> Body {
    if default_betal(&var, betal) == Status::Owned && !live_vars.contains(&var) {
        body.clone()
    } else if var.1 == Type::Heaped {
        Body::Inc(var.clone(), body.clone().into())
    } else {
        body.clone()
    }
}

fn owned_minus(var: &Var, body: &Body, betal: &HashMap<Var, Status>) -> Body {
    if default_betal(var, betal) == Status::Owned
        && !free_vars(body).contains(var)
        && var.1 != Type::Int
    {
        Body::Dec(var.clone(), body.clone().into())
    } else {
        body.clone()
    }
}

fn owned_minus_all(vars: Vec<Var>, body: &Body, betal: &HashMap<Var, Status>) -> Body {
    let mut new_body = body.clone();
    for var in vars {
        new_body = owned_minus(&var, &new_body, betal);
    }
    new_body
}
