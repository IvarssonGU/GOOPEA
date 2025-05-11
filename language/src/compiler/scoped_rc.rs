use std::{collections::HashSet, vec};

use super::{
    crux::Type,
    stir::{Body, Exp, Function, Stir, Var},
};

pub fn add_rc(stir: &Stir) -> Stir {
    stir.iter().map(insert_rc_fun).collect()
}

fn insert_rc_fun(func: &Function) -> Function {
    Function {
        fip: func.fip,
        id: func.id.clone(),
        typ: func.typ.clone(),
        args: func.args.clone(),
        body: insert_rc_body(&func.body, HashSet::new()),
    }
}

fn insert_rc_body(body: &Body, mut set: HashSet<Var>) -> Body {
    match body {
        Body::Ret(var) => add_dec(set, body, var),
        Body::Let(var, expr, next) => match expr {
            Exp::App(_, _) => {
                set.insert(var.clone());
                Body::Let(
                    var.clone(),
                    expr.clone(),
                    Box::new(insert_rc_body(next, set)),
                )
            }
            Exp::Ctor(_, vars) => {
                set.insert(var.clone());
                add_inc(
                    vars.clone(),
                    &Body::Let(
                        var.clone(),
                        expr.clone(),
                        Box::new(insert_rc_body(next, set)),
                    ),
                )
            }
            Exp::Proj(_, _) => {
                Body::Let(var.clone(), expr.clone(), insert_rc_body(next, set).into())
            }
            Exp::Int(_) => Body::Let(
                var.clone(),
                expr.clone(),
                Box::new(insert_rc_body(next, set)),
            ),
            Exp::Op(_, _, _) => Body::Let(
                var.clone(),
                expr.clone(),
                Box::new(insert_rc_body(next, set)),
            ),
            Exp::UTuple(vars) => {
                set.insert(var.clone());
                add_inc(
                    vars.clone(),
                    &Body::Let(
                        var.clone(),
                        expr.clone(),
                        Box::new(insert_rc_body(next, set)),
                    ),
                )
            }
            Exp::Reset(_) => panic!("Should not be possible"),
            Exp::Reuse(_, _, _) => panic!("Should not be possible"),
        },
        Body::Match(var, branches) => Body::Match(
            var.clone(),
            branches
                .iter()
                .map(|(i, branch)| (*i, insert_rc_body(branch, set.clone())))
                .collect(),
        ),
        Body::Dec(_, _) => panic!("Should not be possible"),
        Body::Inc(_, _) => panic!("Should not be possible"),
    }
}

fn add_inc(vars: Vec<Var>, body: &Body) -> Body {
    vars.iter().fold(body.clone(), |body, var| {
        if var.1 == Type::Heaped {
            Body::Inc(var.clone(), Box::new(body))
        } else {
            body
        }
    })
}

fn add_dec(vars: HashSet<Var>, body: &Body, retvar: &Var) -> Body {
    let ret = if !vars.contains(retvar) && retvar.1 == Type::Heaped {
        Body::Inc(retvar.clone(), Box::new(body.clone()))
    } else {
        body.clone()
    };
    vars.iter().fold(ret, |body, var| {
        if var.1 != Type::Int && var != retvar {
            Body::Dec(var.clone(), Box::new(body))
        } else {
            body
        }
    })
}
