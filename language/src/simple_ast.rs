use std::fmt::{Display, Formatter, Result};
use std::sync::atomic::AtomicUsize;
use crate::scoped_ast;
use crate::ast;

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
    Operation(Operator, Box<Expression>, Box<Expression>),
    Constructor(i64, Vec<Expression>),
    LetApp(Vec<String>, Box<Expression>, Box<Expression>),
    Let(String, Box<Expression>, Box<Expression>),
    UTuple(Vec<Expression>),
    Inc(Option<String>, Box<Expression>),
    Dec(Option<String>, Box<Expression>),
}

#[derive(Debug, Clone)]
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
    Div
}

type Pattern = (i64, Vec<Binder>);

#[derive(Debug, Clone)]
pub enum Binder {
    Variable(String),
    Wildcard
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

pub fn from_scoped(ast: &scoped_ast::ScopedProgram) -> Program {
    let mut program = Vec::new();
    
    for (id, scoped_fun) in &ast.functions {
        program.push(FunctionDefinition {
            return_type_len: scoped_fun.def.signature.result_type.0.len() as u8,
            id: id.clone(),
            args: scoped_fun.def.variables.0.clone(),
            body: from_expression(&scoped_fun.body.expr, ast),
        });
    }
    program
}

fn from_expression(expr: &ast::Expression, ast: &scoped_ast::ScopedProgram) -> Expression {
    match expr {
        ast::Expression::FunctionCall(id, args) => {
            match id.as_str() {
                "+" => Expression::Operation(Operator::Add, Box::from(from_expression(&args.0[0], ast)), Box::from(from_expression(&args.0[1], ast))),
                "-" => Expression::Operation(Operator::Sub, Box::from(from_expression(&args.0[0], ast)), Box::from(from_expression(&args.0[1], ast))),
                "*" => Expression::Operation(Operator::Mul, Box::from(from_expression(&args.0[0], ast)), Box::from(from_expression(&args.0[1], ast))),
                "/" => Expression::Operation(Operator::Div, Box::from(from_expression(&args.0[0], ast)), Box::from(from_expression(&args.0[1], ast))),
                _ => match ast.get_constructor(id) {
                    Ok(cons) => Expression::Constructor(cons.internal_id as i64, args.0.iter().map(|arg| from_expression(arg, ast)).collect()),
                    _ => Expression::App(id.clone(), args.0.iter().map(|arg| from_expression(arg, ast)).collect())
                }
            }
        }
        ast::Expression::Integer(i) => Expression::Integer(*i),
        ast::Expression::Variable(id) => match ast.get_constructor(id)  {
            Ok(cons) if cons.constructor.arguments.0.len() == 0 => Expression::Constructor(cons.internal_id as i64, Vec::new()),
            _ => match id.as_str() {
                "true" => Expression::Constructor(1, Vec::new()),
                "false" => Expression::Constructor(0, Vec::new()),
                "True" => Expression::Constructor(1, Vec::new()),
                "False" => Expression::Constructor(0, Vec::new()),
                _ => Expression::Ident(id.clone())
            }   
        },
        ast::Expression::Match(match_exp) => Expression::Match(
            Box::from(from_expression(&match_exp.expr, ast)), 
            match_exp.cases.iter().map(|case| (
                {
                    match &case.pattern {
                        ast::Pattern::Integer(_) => todo!(),
                        ast::Pattern::UTuple(utuple) => todo!(),
                        ast::Pattern::Constructor(fid, vars) => {
                            if vars.0.len() == 0 {
                                (ast.get_constructor(fid).unwrap().internal_id as i64, vec![])
                            }
                            else {
                                (ast.get_constructor(fid).unwrap().internal_id as i64, vars.0.iter().map(|var| Binder::Variable(var.clone())).collect())
                            }
                        },
                    }
                },
                from_expression(&case.body, ast)
            )
            ).collect()),
        ast::Expression::UTuple(exps) => Expression::UTuple(exps.0.iter().map(|expr| from_expression(expr, ast)).collect()),
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
            body: fun.args.clone().iter().fold(add_refcounts_expr(&fun.body), |acc, arg| Expression::Dec(Some(arg.clone()), Box::from(acc))),
        });
    }
    new_prog
}

fn add_refcounts_expr(exp: &Expression) -> Expression {
    match exp {
        Expression::Ident(id) => Expression::Ident(id.clone()),
        Expression::Integer(i) => Expression::Integer(*i),
        Expression::App(id, exps) => {
            let arguments = exps.iter().map(|exp| Expression::Inc(None, Box::from(add_refcounts_expr(exp)))).collect();
            Expression::App(id.clone(), arguments)
        },
        Expression::Constructor(tag, exps) => {
            let arguments = exps.iter().map(|exp| Expression::Inc(None, Box::from(add_refcounts_expr(exp)))).collect();
            Expression::Constructor(tag.clone(), arguments)
        },
        Expression::Let(id, exp1, exp2) => {
            Expression::Let(
                id.clone(), 
                Box::from(add_refcounts_expr(exp1)), 
                Box::from(
                    Expression::Inc(
                        Some(id.clone()),
                        Box::from(Expression::Dec(
                            Some(id.clone()), 
                            Box::from(add_refcounts_expr(exp2))
                        ))
                    )
                )
            )
        },
        Expression::Match(exp, cases) => Expression::Match(
            Box::from(add_refcounts_expr(exp)), 
            {
                let mut new_cases = Vec::new();
                for ((tag, binders), exp) in cases {
                    let new_exp = binders.iter().fold(add_refcounts_expr(exp), |acc, binder| {
                        match binder {
                            Binder::Variable(id) => Expression::Inc(
                                    Some(id.clone()),
                                    Box::from(Expression::Dec(
                                        Some(id.clone()), 
                                        Box::from(acc)
                                    ))
                                ),
                            _ => acc
                        }
                    });
                    new_cases.push(((tag.clone(), binders.clone()), new_exp));
                }
                new_cases
            }
        ),
        Expression::Operation(op, left, right) => Expression::Operation(
                op.clone(), 
                Box::from(add_refcounts_expr(left)), 
                Box::from(add_refcounts_expr(right))
        ),
        _ => panic!("case should not be possible")   
    } 
}