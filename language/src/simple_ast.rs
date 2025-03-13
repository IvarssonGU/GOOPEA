use std::fmt::{Display, Formatter, Result};
use crate::ast_wrappers::ast_wrapper::{self, ExprChildren};
use crate::ast;
use crate::ast_wrappers::scope_wrapper::ScopedProgram;
use crate::ast_wrappers::type_wrapper::{TypeWrapper, TypedProgram};

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

pub fn from_scoped(ast: &TypedProgram) -> Program {
    let mut program = Vec::new();
    for (id, func) in &ast.functions {
        program.push(FunctionDefinition {
            id: id.clone(),
            args: func.vars.0.clone(),
            body: from_expression(&func.body, ast),
        });
    }
    program
}

fn from_expression(expr: &TypeWrapper, ast: &TypedProgram) -> Expression {
    match &expr.data.prev.prev {
        ast::Expression::FunctionCall(id) => {
            match id.as_str() {
                "+" => Expression::Operation(Operator::Add, Box::from(from_expression(expr.children.all_children()[0], ast)), Box::from(from_expression(expr.children.all_children()[1], ast))),
                "-" => Expression::Operation(Operator::Sub, Box::from(from_expression(expr.children.all_children()[0], ast)), Box::from(from_expression(expr.children.all_children()[1], ast))),
                "*" => Expression::Operation(Operator::Mul, Box::from(from_expression(expr.children.all_children()[0], ast)), Box::from(from_expression(expr.children.all_children()[1], ast))),
                "/" => Expression::Operation(Operator::Div, Box::from(from_expression(expr.children.all_children()[0], ast)), Box::from(from_expression(expr.children.all_children()[1], ast))),
                _ => match ast.constructors.get(id) {
                    Some(cons) => Expression::Constructor(cons.internal_id as i64, expr.children.all_children().into_iter().map(|arg| from_expression(arg, ast)).collect()),
                    _ => Expression::FunctionCall(id.clone(), expr.children.all_children().into_iter().map(|arg| from_expression(arg, ast)).collect())
                }
            }
        }
        ast::Expression::Integer(i) => Expression::Integer(*i),
        ast::Expression::Variable(id) => Expression::Identifier(id.clone()),
        ast::Expression::Match(patterns) => {
            let ExprChildren::Match(match_child, case_children) = &expr.children else { panic!() };

            Expression::Match(
                Box::from(from_expression(&match_child, ast)), 
                patterns.iter().zip(case_children.iter()).map(|(pattern, child)| MatchCase {
                    pattern: {
                        match pattern {
                            ast::Pattern::Integer(_) => todo!(),
                            ast::Pattern::UTuple(utuple) => todo!(),
                            ast::Pattern::Constructor(fid, vars) => {
                                if vars.0.len() == 0 {
                                    Pattern::Atom(ast.constructors.get(fid).unwrap().internal_id as i64)
                                }
                                else {
                                    Pattern::Constructor(
                                        ast.constructors.get(fid).unwrap().internal_id as i64, 
                                        vars.0.iter().map(|var| Some(var.clone())).collect())
                                }
                            },
                        }
                    },
                    body: from_expression(child, ast)
            }).collect())
        },
        ast::Expression::UTuple => Expression::UTuple(expr.children.all_children().into_iter().map(|expr| from_expression(expr, ast)).collect()),
        //ast::Expression::LetEqualIn(ids, left, right) => Expression::Let(ids.0.clone(), Box::from(from_expression(&left, ast)), Box::from(from_expression(&right, ast))),
    }
}
