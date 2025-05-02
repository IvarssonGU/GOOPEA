use crate::compiler::simple::Type;
use crate::compiler::stir::{Body, Exp, Function, Stir, Var, next_var};

fn reuse_all_matches(body: &Body) -> Body {
    match body {
        Body::Ret(var) => Body::Ret(var.clone()),
        Body::Let(var, exp, next) => {
            Body::Let(var.clone(), exp.clone(), reuse_all_matches(next).into())
        }
        Body::Match(var, branches) => {
            let mut new_branches = vec![];
            for (cons_len, branch) in branches {
                new_branches.push((
                    *cons_len,
                    evaluate_reuse_in_case(var.clone(), *cons_len, &reuse_all_matches(branch)),
                ));
            }
            Body::Match(var.clone(), new_branches)
        }
        _ => panic!("Does not exist at this stage"),
    }
}

fn evaluate_reuse_in_case(var: Var, len: u8, body: &Body) -> Body {
    match body {
        Body::Match(case_var, branches) => Body::Match(
            case_var.clone(),
            branches
                .iter()
                .map(|(cons_len, branch)| {
                    (*cons_len, evaluate_reuse_in_case(var.clone(), len, branch))
                })
                .collect(),
        ),
        Body::Ret(ret_var) => Body::Ret(ret_var.clone()),
        Body::Let(let_var, exp, next) if exp.member(&var) || next.member(&var) => Body::Let(
            let_var.clone(),
            exp.clone(),
            evaluate_reuse_in_case(var, len, next).into(),
        ),
        _ => {
            let fresh = next_var();
            let try_replace = insert_reuse(fresh.clone(), len, body);
            if try_replace != *body {
                Body::Let((fresh, Type::Heaped), Exp::Reset(var), try_replace.into())
            } else {
                try_replace
            }
        }
    }
}

fn insert_reuse(var: String, len: u8, body: &Body) -> Body {
    match body {
        Body::Let(let_var, exp, next) => match exp {
            Exp::Ctor(tag, vars) if vars.len() as u8 == len => Body::Let(
                let_var.clone(),
                Exp::Reuse((var, Type::Heaped), *tag, vars.clone()),
                next.clone(),
            ),
            _ => Body::Let(
                let_var.clone(),
                exp.clone(),
                insert_reuse(var, len, next).into(),
            ),
        },
        Body::Ret(ret_var) => Body::Ret(ret_var.clone()),
        Body::Match(case_var, branches) => Body::Match(
            case_var.clone(),
            branches
                .iter()
                .map(|(cons_len, branch)| (*cons_len, insert_reuse(var.clone(), len, branch)))
                .collect(),
        ),
        _ => panic!("Does not exist at this stage"),
    }
}

pub fn add_reuse(prog: &Stir) -> Stir {
    prog.iter()
        .map(|func| Function {
            fip: func.fip,
            id: func.id.clone(),
            typ: func.typ.clone(),
            args: func.args.clone(),
            body: if func.fip {
                reuse_all_matches(&func.body)
            } else {
                func.body.clone()
            },
        })
        .collect()
}
