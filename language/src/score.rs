use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::Hash;
use std::vec;

//score = Stir-to-CORE
use crate::core::{Def, Operand, Prog, Statement, Type};
use crate::stir::{Body, Exp, Function, Operator, Stir, Type as StirType};

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

pub fn translate(prog: &Stir) -> Prog {
    let mut utuples = HashSet::new();
    for def in prog {
        utuples.extend(collect_utuples(&def.body));
    }
    (
        prog.iter()
            .map(|def| Def {
                id: def.id.clone(),
                typ: from_type(&def.typ),
                args: def
                    .args
                    .iter()
                    .map(|(var, _)| var.clone())
                    .collect::<Vec<String>>(),
                body: translate_body(&def.body, vec![], &def.id),
            })
            .collect(),
        utuples.clone(),
    )
}

fn collect_utuples(body: &Body) -> HashSet<u8> {
    match body {
        Body::Ret(_) => HashSet::new(),
        Body::Let(_, exp, next) => {
            let mut set = collect_utuples(next);
            if let Exp::UTuple(vars) = exp {
                set.insert(vars.len() as u8);
            }
            set
        }
        Body::Match(_, branches) => {
            let mut set = HashSet::new();
            for (_, branch) in branches.iter() {
                set.extend(collect_utuples(branch));
            }
            set
        }
        Body::Inc(_, next) => collect_utuples(next),
        Body::Dec(_, next) => collect_utuples(next),
    }
}

fn from_type(typ: &StirType) -> Type {
    match typ {
        StirType::Heaped => Type::Standard,
        StirType::Int => Type::Standard,
        StirType::Unboxed(vec) => Type::Value(vec.len() as u8),
    }
}

fn translate_body(body: &Body, mut stmts: Vec<Statement>, fid: &String) -> Vec<Statement> {
    match body {
        Body::Ret(var) => {
            if fid == "main" {
                stmts.push(Statement::Print(Operand::Ident(var.0.clone())));
                stmts.push(Statement::Return(Operand::NonShifted(0)));
            } else {
                stmts.push(Statement::Return(Operand::Ident(var.0.clone())));
            }

            stmts
        }
        Body::Let(var, exp, next) => {
            match exp {
                Exp::Int(i) => {
                    stmts.push(Statement::Assign(
                        Type::Standard,
                        var.0.clone(),
                        Operand::Int(*i),
                    ));
                }
                Exp::App(id, args) => {
                    stmts.push(Statement::AssignFunctionCall(
                        var.0.clone(),
                        id.clone(),
                        args.iter().map(|a| Operand::Ident(a.0.clone())).collect(),
                        from_type(&var.1),
                    ));
                }
                Exp::Ctor(tag, args) => {
                    if args.is_empty() {
                        stmts.push(Statement::Assign(
                            Type::Standard,
                            var.0.clone(),
                            Operand::Int(*tag as i64),
                        ))
                    } else {
                        stmts.push(Statement::AssignMalloc(
                            Type::VoidPtrPtr,
                            var.0.clone(),
                            args.len() as u8,
                        ));
                        stmts.push(Statement::AssignToField(
                            var.0.clone(),
                            0,
                            Operand::Int(*tag as i64),
                        ));
                        stmts.push(Statement::AssignToField(
                            var.0.clone(),
                            1,
                            Operand::NonShifted(args.len() as i64),
                        ));
                        stmts.push(Statement::AssignToField(
                            var.0.clone(),
                            2,
                            Operand::NonShifted(1),
                        ));
                        for (i, arg) in args.iter().enumerate() {
                            stmts.push(Statement::AssignToField(
                                var.0.clone(),
                                (i + 3) as i64,
                                Operand::Ident(arg.0.clone()),
                            ));
                        }
                    }
                }
                Exp::UTuple(vars) => stmts.push(Statement::AssignUTuple(
                    vars.len() as u8,
                    var.0.clone(),
                    vars.iter()
                        .map(|(var, _)| var.clone())
                        .collect::<Vec<String>>(),
                )),
                Exp::Op(op, left, right) => {
                    stmts.push(Statement::AssignBinaryOperation(
                        var.0.clone(),
                        *op,
                        Operand::Ident(left.0.clone()),
                        Operand::Ident(right.0.clone()),
                    ));
                }
                Exp::Proj(field, projectee) => {
                    if let StirType::Unboxed(_) = projectee.1 {
                        stmts.push(Statement::AssignUTupleField(
                            var.0.clone(),
                            *field as i64,
                            Operand::Ident(projectee.0.clone()),
                        ));
                    } else {
                        stmts.push(Statement::AssignFromField(
                            var.0.clone(),
                            *field as i64 + 3,
                            Operand::Ident(projectee.0.clone()),
                        ));
                    }
                }
                Exp::Reset(reset_var) => stmts.push(Statement::AssignDropReuse(
                    var.0.clone(),
                    reset_var.0.clone(),
                )),
                Exp::Reuse(reuse_var, tag, args) => {
                    let if_stmts = vec![
                        Statement::AssignMalloc(Type::None, reuse_var.0.clone(), args.len() as u8),
                        Statement::AssignToField(
                            reuse_var.0.clone(),
                            1,
                            Operand::NonShifted(args.len() as i64),
                        ),
                        Statement::AssignToField(reuse_var.0.clone(), 2, Operand::NonShifted(1)),
                    ];
                    stmts.push(Statement::IfElse(vec![(
                        Operand::Negate(reuse_var.0.clone()),
                        if_stmts,
                    )]));
                    stmts.push(Statement::AssignToField(
                        reuse_var.0.clone(),
                        0,
                        Operand::Int(*tag as i64),
                    ));
                    for (i, arg) in args.iter().enumerate() {
                        stmts.push(Statement::AssignToField(
                            reuse_var.0.clone(),
                            (i + 3) as i64,
                            Operand::Ident(arg.0.clone()),
                        ));
                    }
                    stmts.push(Statement::Assign(
                        Type::Standard,
                        var.0.clone(),
                        Operand::Ident(reuse_var.0.clone()),
                    ));
                }
            }
            translate_body(next, stmts, fid)
        }
        Body::Match(var, branches) => {
            let mut new_branches = vec![];
            let mut operands = vec![];
            for (i, branch) in branches.iter().enumerate() {
                let match_var = next_var();
                stmts.push(Statement::AssignTagCheck(
                    match_var.clone(),
                    branch.0 != 0,
                    Operand::Ident(var.0.clone()),
                    (i as i64) << 1 | 1,
                ));
                operands.push(Operand::Ident(match_var));
            }
            for (i, (_, branch)) in branches.iter().enumerate() {
                let translated = translate_body(branch, vec![], fid);
                new_branches.push((operands[i].clone(), translated));
            }
            stmts.push(Statement::IfElse(new_branches));
            stmts
        }
        Body::Inc(var, next) => {
            stmts.push(Statement::Inc(var.0.clone()));
            translate_body(next, stmts, fid)
        }
        Body::Dec(var, next) => {
            if let StirType::Unboxed(vec) = &var.1 {
                stmts.push(Statement::DecUTuple(var.0.clone(), vec.len() as u8));
            } else {
                stmts.push(Statement::Dec(var.0.clone()));
            }

            translate_body(next, stmts, fid)
        }
    }
}
