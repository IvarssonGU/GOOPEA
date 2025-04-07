use std::cell::RefCell;
use std::vec;

//score = Stir-to-CORE
use crate::core::*;
use crate::stir::*;

fn next_var() -> String {
    thread_local!(
        static COUNTER: RefCell<usize> = Default::default();
    );
    let current = COUNTER.with_borrow_mut(|c| {
        *c += 1;
        *c
    });
    format!("match_var{}", current)
}

pub fn translate(prog: &Stir) -> Vec<Def> {
    prog.iter()
        .map(|def| Def {
            id: def.id.clone(),
            args: def.args.clone(),
            body: translate_body(&def.body, vec![]),
        })
        .collect()
}

fn translate_fun(fun: &Function) -> Def {
    Def {
        id: fun.id.clone(),
        args: fun.args.clone(),
        body: translate_body(&fun.body, vec![]),
    }
}

fn translate_body(body: &Body, mut stmts: Vec<Statement>) -> Vec<Statement> {
    match body {
        Body::Ret(var) => {
            stmts.push(Statement::Return(Operand::Ident(var.clone())));
            stmts
        }
        Body::Let(var, exp, next) => {
            match exp {
                Exp::Int(i) => {
                    stmts.push(Statement::Assign(
                        Type::Value,
                        var.clone(),
                        Operand::Int(*i),
                    ));
                }
                Exp::App(id, args) => {
                    stmts.push(Statement::AssignFunctionCall(
                        var.clone(),
                        id.clone(),
                        args.iter().map(|a| Operand::Ident(a.clone())).collect(),
                    ));
                }
                Exp::Ctor(tag, args) => {
                    if args.is_empty() {
                        stmts.push(Statement::Assign(
                            Type::Value,
                            var.clone(),
                            Operand::Int(*tag as i64),
                        ))
                    } else {
                        stmts.push(Statement::AssignMalloc(
                            Type::VoidPtrPtr,
                            var.clone(),
                            args.len() as u8,
                        ));
                        stmts.push(Statement::AssignToField(
                            var.clone(),
                            0,
                            Operand::Int(*tag as i64),
                        ));
                        stmts.push(Statement::AssignToField(
                            var.clone(),
                            1,
                            Operand::NonShifted(args.len() as i64),
                        ));
                        stmts.push(Statement::AssignToField(
                            var.clone(),
                            2,
                            Operand::NonShifted(1),
                        ));
                        for (i, arg) in args.iter().enumerate() {
                            stmts.push(Statement::AssignToField(
                                var.clone(),
                                (i + 3) as i64,
                                Operand::Ident(arg.clone()),
                            ));
                        }
                    }
                }
                Exp::Op(op, left, right) => {
                    stmts.push(Statement::AssignBinaryOperation(
                        var.clone(),
                        *op,
                        Operand::Ident(left.clone()),
                        Operand::Ident(right.clone()),
                    ));
                }
                Exp::Proj(field, projectee) => {
                    stmts.push(Statement::AssignFromField(
                        var.clone(),
                        *field as i64 + 3,
                        Operand::Ident(projectee.clone()),
                    ));
                }
                Exp::Reset(reset_var) => {
                    stmts.push(Statement::AssignDropReuse(var.clone(), reset_var.clone()))
                }
                Exp::Reuse(reuse_var, tag, args) => {
                    let if_stmts = vec![
                        Statement::AssignMalloc(Type::None, reuse_var.clone(), args.len() as u8),
                        Statement::AssignToField(reuse_var.clone(), 0, Operand::Int(*tag as i64)),
                        Statement::AssignToField(
                            reuse_var.clone(),
                            1,
                            Operand::NonShifted(args.len() as i64),
                        ),
                        Statement::AssignToField(reuse_var.clone(), 2, Operand::NonShifted(1)),
                    ];
                    stmts.push(Statement::IfElse(vec![(
                        Operand::Negate(reuse_var.clone()),
                        if_stmts,
                    )]));

                    for (i, arg) in args.iter().enumerate() {
                        stmts.push(Statement::AssignToField(
                            reuse_var.clone(),
                            (i + 3) as i64,
                            Operand::Ident(arg.clone()),
                        ));
                    }
                    stmts.push(Statement::Assign(
                        Type::Value,
                        var.clone(),
                        Operand::Ident(reuse_var.clone()),
                    ));
                }
            }
            translate_body(next, stmts)
        }
        Body::Match(var, branches) => {
            let mut new_branches = vec![];
            let mut operands = vec![];
            for (i, branch) in branches.iter().enumerate() {
                let match_var = next_var();
                stmts.push(Statement::AssignTagCheck(
                    match_var.clone(),
                    branch.0 != 0,
                    Operand::Ident(var.clone()),
                    (i as i64) << 1 | 1,
                ));
                operands.push(Operand::Ident(match_var));
            }
            for (i, (_, branch)) in branches.iter().enumerate() {
                let translated = translate_body(branch, vec![]);
                new_branches.push((operands[i].clone(), translated));
            }
            stmts.push(Statement::IfElse(new_branches));
            stmts
        }
        Body::Inc(var, next) => {
            stmts.push(Statement::Inc(var.clone()));
            translate_body(next, stmts)
        }
        Body::Dec(var, next) => {
            stmts.push(Statement::Dec(var.clone()));
            translate_body(next, stmts)
        }
    }
}
