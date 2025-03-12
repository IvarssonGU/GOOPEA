use std::fmt::{Display, Formatter, Result};
use crate::scoped_ast;
use crate::ast;

pub type Program = Vec<FunctionDefinition>;

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub id: String,
    pub args: Vec<String>,
    pub body: Expression,
}

#[derive(Debug, Clone)]
pub enum Expression {
    FunctionCall(String, Vec<Expression>),
    Identifier(String),
    Integer(i64),  
    Match(Box<Expression>, Vec<MatchCase>),
    Operation(Operator, Box<Expression>, Box<Expression>),
    Constructor(i64, Vec<Expression>),
    Let(Vec<String>, Box<Expression>, Box<Expression>),
    UTuple(Vec<Expression>)
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

#[derive(Debug, Clone)]
pub struct MatchCase {
    pub pattern: Pattern,
    pub body: Expression,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Identifier(String),
    Integer(i64),
    Wildcard,
    Atom(i64),
    Constructor(i64, Vec<Option<String>>),
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
                    _ => Expression::FunctionCall(id.clone(), args.0.iter().map(|arg| from_expression(arg, ast)).collect())
                }
            }
        }
        ast::Expression::Integer(i) => Expression::Integer(*i),
        ast::Expression::Variable(id) => Expression::Identifier(id.clone()),
        ast::Expression::Match(match_exp) => Expression::Match(
            Box::from(from_expression(&match_exp.expr, ast)), 
            match_exp.cases.iter().map(|case| MatchCase {
                pattern: {
                    match &case.pattern {
                        ast::Pattern::Integer(_) => todo!(),
                        ast::Pattern::UTuple(utuple) => todo!(),
                        ast::Pattern::Constructor(fid, vars) => {
                            if vars.0.len() == 0 {
                                Pattern::Atom(ast.get_constructor(fid).unwrap().internal_id as i64)
                            }
                            else {
                                Pattern::Constructor(
                                    ast.get_constructor(fid).unwrap().internal_id as i64, 
                                    vars.0.iter().map(|var| Some(var.clone())).collect())
                            }
                        },
                    }
                },
                body: from_expression(&case.body, ast)
            }).collect()),
        ast::Expression::UTuple(exps) => Expression::UTuple(exps.0.iter().map(|expr| from_expression(expr, ast)).collect()),
        ast::Expression::LetEqualIn(ids, left, right) => Expression::Let(ids.0.clone(), Box::from(from_expression(&left, ast)), Box::from(from_expression(&right, ast))),
    }
}
