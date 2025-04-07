use crate::ast;
use crate::scoped;
use core::panic;
use std::fmt::{Display, Formatter, Result};

pub type Program = Vec<FunctionDefinition>;

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub return_type_len: u8,
    pub id: String,
    pub args: Vec<String>,
    pub body: Expression,
}

#[derive(Debug, Clone)]
pub enum Expression {
    App(String, Vec<Expression>),
    Ident(String),
    Integer(i64),
    Match(Box<Expression>, Vec<(Pattern, Expression)>),
    Constructor(i64, Vec<Expression>),
    Projection(i64, Box<Expression>),
    Operation(Operator, Box<Expression>, Box<Expression>),
    LetApp(Vec<String>, Box<Expression>, Box<Expression>),
    Let(String, Box<Expression>, Box<Expression>),
    UTuple(Vec<Expression>),
    Inc(String, Box<Expression>),
    Dec(String, Box<Expression>),
}

#[derive(Debug, Clone, Copy)]
pub enum Operator {
    Equal,
    NotEqual,
    Less,
    LessOrEq,
    Greater,
    GreaterOrEqual,
    Add,
    Sub,
    Mul,
    Div,
}

type Pattern = (i64, Vec<Binder>);

#[derive(Debug, Clone)]
pub enum Binder {
    Variable(String),
    Wildcard,
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Operator::Equal => write!(f, "=="),
            Operator::NotEqual => write!(f, "!="),
            Operator::Less => write!(f, "<"),
            Operator::LessOrEq => write!(f, "<="),
            Operator::Greater => write!(f, ">"),
            Operator::GreaterOrEqual => write!(f, ">="),
            Operator::Add => write!(f, "+"),
            Operator::Sub => write!(f, "-"),
            Operator::Mul => write!(f, "*"),
            Operator::Div => write!(f, "/"),
        }
    }
}

pub fn from_scoped(ast: &scoped::ScopedProgram) -> Program {
    let mut program = Vec::new();

    for (id, scoped_fun) in &ast.functions {
        program.push(FunctionDefinition {
            return_type_len: scoped_fun.def.signature.result_type.0.len() as u8,
            id: id.clone(),
            args: scoped_fun.def.variables.0.clone(),
            body: from_expression(scoped_fun.body.expr, ast),
        });
    }
    program
}

fn from_expression(expr: &ast::Expression, ast: &scoped::ScopedProgram) -> Expression {
    match expr {
        ast::Expression::FunctionCall(id, args) => match id.as_str() {
            "+" => Expression::Operation(
                Operator::Add,
                Box::from(from_expression(&args.0[0], ast)),
                Box::from(from_expression(&args.0[1], ast)),
            ),
            "-" => Expression::Operation(
                Operator::Sub,
                Box::from(from_expression(&args.0[0], ast)),
                Box::from(from_expression(&args.0[1], ast)),
            ),
            "*" => Expression::Operation(
                Operator::Mul,
                Box::from(from_expression(&args.0[0], ast)),
                Box::from(from_expression(&args.0[1], ast)),
            ),
            "/" => Expression::Operation(
                Operator::Div,
                Box::from(from_expression(&args.0[0], ast)),
                Box::from(from_expression(&args.0[1], ast)),
            ),
            _ => match ast.get_constructor(id) {
                Ok(cons) => Expression::Constructor(
                    cons.internal_id as i64,
                    args.0.iter().map(|arg| from_expression(arg, ast)).collect(),
                ),
                _ => Expression::App(
                    id.clone(),
                    args.0.iter().map(|arg| from_expression(arg, ast)).collect(),
                ),
            },
        },
        ast::Expression::Integer(i) => Expression::Integer(*i),
        ast::Expression::Variable(id) => match ast.get_constructor(id) {
            Ok(cons) if cons.constructor.arguments.0.is_empty() => {
                Expression::Constructor(cons.internal_id as i64, Vec::new())
            }
            _ => match id.as_str() {
                "true" => Expression::Constructor(1, Vec::new()),
                "false" => Expression::Constructor(0, Vec::new()),
                "True" => Expression::Constructor(1, Vec::new()),
                "False" => Expression::Constructor(0, Vec::new()),
                _ => Expression::Ident(id.clone()),
            },
        },
        ast::Expression::Match(match_exp) => Expression::Match(
            Box::from(from_expression(&match_exp.expr, ast)),
            match_exp
                .cases
                .iter()
                .map(|case| {
                    (
                        {
                            match &case.pattern {
                                ast::Pattern::Integer(_) => todo!(),
                                ast::Pattern::UTuple(utuple) => todo!(),
                                ast::Pattern::Constructor(fid, vars) => {
                                    if vars.0.is_empty() {
                                        (
                                            ast.get_constructor(fid).unwrap().internal_id as i64,
                                            vec![],
                                        )
                                    } else {
                                        (
                                            ast.get_constructor(fid).unwrap().internal_id as i64,
                                            vars.0
                                                .iter()
                                                .map(|var| Binder::Variable(var.clone()))
                                                .collect(),
                                        )
                                    }
                                }
                            }
                        },
                        from_expression(&case.body, ast),
                    )
                })
                .collect(),
        ),
        ast::Expression::UTuple(exps) => Expression::UTuple(
            exps.0
                .iter()
                .map(|expr| from_expression(expr, ast))
                .collect(),
        ),
        //ast::Expression::LetEqualIn(ids, left, right) => Expression::Let(ids.0.clone(), Box::from(from_expression(&left, ast)), Box::from(from_expression(&right, ast))),
    }
}

pub fn add_refcounts(prog: &Program) -> Program {
    let mut new_prog = Vec::new();
    for fun in prog {
        new_prog.push(FunctionDefinition {
            return_type_len: fun.return_type_len,
            id: fun.id.clone(),
            args: fun.args.clone(),
            body: {
                let orig_body_exp = fun
                    .args
                    .clone()
                    .iter()
                    .fold(add_refcounts_expr(&fun.body), |acc, arg| {
                        Expression::Dec(arg.clone(), Box::from(acc))
                    });
                scuff_decs(&orig_body_exp)
            },
        });
    }
    new_prog
}

fn add_refcounts_expr(exp: &Expression) -> Expression {
    match exp {
        Expression::Ident(id) => {
            Expression::Inc(id.clone(), Box::from(Expression::Ident(id.clone())))
        }
        Expression::Integer(i) => Expression::Integer(*i),
        Expression::App(id, exps) => {
            let arguments = exps.iter().map(add_refcounts_expr).collect();
            Expression::App(id.clone(), arguments)
        }
        Expression::Constructor(tag, exps) => {
            let arguments = exps.iter().map(add_refcounts_expr).collect();
            Expression::Constructor(*tag, arguments)
        }
        Expression::Match(exp, cases) => match &**exp {
            Expression::Ident(id) => Expression::Match(Box::from(Expression::Ident(id.clone())), {
                let mut new_cases = Vec::new();
                for ((tag, binders), exp) in cases {
                    let new_exp =
                        binders
                            .iter()
                            .fold(add_refcounts_expr(exp), |acc, binder| match binder {
                                Binder::Variable(id) => Expression::Inc(
                                    id.clone(),
                                    Box::from(Expression::Dec(id.clone(), Box::from(acc))),
                                ),
                                _ => acc,
                            });
                    new_cases.push(((*tag, binders.clone()), new_exp));
                }
                new_cases
            }),
            _ => panic!("currently we can only have idents here!!"),
        },
        Expression::Operation(op, left, right) => Expression::Operation(
            *op,
            Box::from(add_refcounts_expr(left)),
            Box::from(add_refcounts_expr(right)),
        ),
        Expression::Let(id, exp1, exp2) => Expression::Let(
            id.clone(),
            Box::from(add_refcounts_expr(exp1)),
            Box::from(Expression::Dec(
                id.clone(),
                Box::from(add_refcounts_expr(exp2)),
            )),
        ),
        _ => panic!("not implemented"),
    }
}

fn scuff_decs(exp: &Expression) -> Expression {
    match exp {
        Expression::Operation(op, left, right) => Expression::Operation(
            *op,
            Box::from(scuff_decs(left)),
            Box::from(scuff_decs(right)),
        ),
        Expression::App(id, exps) => {
            Expression::App(id.clone(), exps.iter().map(scuff_decs).collect())
        }
        Expression::Constructor(tag, exps) => {
            Expression::Constructor(*tag, exps.iter().map(scuff_decs).collect())
        }
        Expression::Ident(id) => Expression::Ident(id.clone()),
        Expression::Integer(i) => Expression::Integer(*i),
        Expression::Inc(inc_id, next_exp) => {
            Expression::Inc(inc_id.clone(), Box::from(scuff_decs(next_exp)))
        }
        Expression::Dec(dec_id, exp) => {
            scuff_dec(&Expression::Dec(dec_id.clone(), Box::from(scuff_decs(exp))))
        }
        Expression::Match(match_exp, cases) => Expression::Match(
            Box::from(scuff_decs(match_exp)),
            cases
                .iter()
                .map(|(pat, exp)| (pat.clone(), scuff_decs(exp)))
                .collect(),
        ),
        _ => panic!("not implemented"),
    }
}

fn scuff_dec(dec_exp: &Expression) -> Expression {
    let Expression::Dec(dec_id, match_exp) = dec_exp else {
        panic!("Encountered non-dec expression")
    };
    if !search(dec_id, match_exp) {
        return dec_exp.clone();
    }
    match &**match_exp {
        Expression::Ident(_) => panic!("should not be possible!"),
        Expression::Integer(_) => panic!("should not be possible!"),
        Expression::App(fid, exps) => {
            let mut new_exps = exps.clone();
            for i in (0..new_exps.len()).rev() {
                if search(dec_id, &exps[i]) {
                    new_exps[i] =
                        scuff_dec(&Expression::Dec(dec_id.clone(), Box::from(exps[i].clone())));
                    break;
                }
            }
            Expression::App(fid.clone(), new_exps)
        }
        Expression::Constructor(tag, exps) => {
            let mut new_exps = exps.clone();
            for i in (0..new_exps.len()).rev() {
                if search(dec_id, &new_exps[i]) {
                    new_exps[i] =
                        scuff_dec(&Expression::Dec(dec_id.clone(), Box::from(exps[i].clone())));
                    break;
                }
            }
            Expression::Constructor(*tag, new_exps)
        }
        Expression::Operation(op, left, right) => {
            if search(dec_id, right) {
                Expression::Operation(
                    *op,
                    left.clone(),
                    Box::from(scuff_dec(&Expression::Dec(dec_id.clone(), right.clone()))),
                )
            } else {
                Expression::Operation(
                    *op,
                    Box::from(scuff_dec(&Expression::Dec(dec_id.clone(), left.clone()))),
                    right.clone(),
                )
            }
        }
        Expression::Inc(inc_id, next_exp) => {
            if inc_search(dec_id, next_exp) {
                Expression::Inc(
                    inc_id.clone(),
                    Box::from(scuff_dec(&Expression::Dec(
                        dec_id.clone(),
                        next_exp.clone(),
                    ))),
                )
            } else {
                *next_exp.clone()
            }
        }
        Expression::Match(match_exp, cases) => {
            let mut new_cases = Vec::new();
            for (pat, exp) in cases {
                if search(dec_id, exp) {
                    new_cases.push((
                        pat.clone(),
                        scuff_dec(&Expression::Dec(dec_id.clone(), Box::from(exp.clone()))),
                    ));
                } else {
                    new_cases.push((
                        pat.clone(),
                        fun(&Expression::Dec(dec_id.clone(), Box::from(exp.clone()))),
                    ));
                }
            }
            Expression::Match(match_exp.clone(), new_cases)
        }
        Expression::Dec(id, exp) => Expression::Dec(
            id.clone(),
            Box::from(scuff_dec(&Expression::Dec(dec_id.clone(), exp.clone()))),
        ),
        _ => panic!("not implemented"),
    }
}

fn fun(dec_exp: &Expression) -> Expression {
    match dec_exp {
        Expression::Dec(dec_id, match_exp) => match &**match_exp {
            Expression::Inc(inc_id, next_exp) => Expression::Inc(
                inc_id.clone(),
                Box::from(fun(&Expression::Dec(dec_id.clone(), next_exp.clone()))),
            ),
            _ => dec_exp.clone(),
        },
        _ => panic!("not implemented"),
    }
}

fn search(id: &str, e: &Expression) -> bool {
    match e {
        Expression::Ident(var_id) => id == var_id,
        Expression::Integer(_) => false,
        Expression::App(_, exps) => exps.iter().any(|exp| search(id, exp)),
        Expression::Constructor(_, exps) => exps.iter().any(|exp| search(id, exp)),
        Expression::Operation(_, left, right) => search(id, left) || search(id, right),
        Expression::Inc(inc_id, exp) => id == inc_id.as_str() || search(id, exp),
        Expression::Dec(_, exp) => search(id, exp),
        Expression::Match(exp, cases) => {
            search(id, exp) || cases.iter().any(|(_, e)| search(id, e))
        }
        _ => false,
    }
}

fn inc_search(id: &str, e: &Expression) -> bool {
    match e {
        Expression::Ident(_) => false,
        Expression::Integer(_) => false,
        Expression::App(_, exps) => exps.iter().any(|exp| inc_search(id, exp)),
        Expression::Constructor(_, exps) => exps.iter().any(|exp| inc_search(id, exp)),
        Expression::Operation(_, left, right) => search(id, left) || inc_search(id, right),
        Expression::Inc(inc_id, exp) => id == inc_id.as_str() || inc_search(id, exp),
        Expression::Dec(_, exp) => inc_search(id, exp),
        Expression::Match(exp, cases) => {
            inc_search(id, exp) || cases.iter().any(|(_, e)| inc_search(id, e))
        }
        _ => false,
    }
}
