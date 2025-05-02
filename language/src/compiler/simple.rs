use std::fmt::{Display, Formatter, Result};

use crate::ast::typed::ExpressionType;
use crate::ast::{
    ast, scoped,
    typed::{TypedNode, TypedProgram},
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Type {
    Int,
    Heaped,
    Unboxed(Vec<Type>),
}

#[derive(Clone, Debug)]
pub enum Simple {
    Ident(String, Type),
    Int(i64, Type),
    Operation(Operator, Box<Simple>, Box<Simple>, Type),
    Constructor(i64, Vec<Simple>, Type),
    App(String, Vec<Simple>, Type),
    Match(Box<Simple>, Vec<(Pattern, Simple)>, Type),
    Let(String, Box<Simple>, Box<Simple>, Type),
    UTuple(Vec<Simple>, Type),
    LetApp(Vec<String>, Box<Simple>, Box<Simple>, Type),
}

type Pattern = (i64, Vec<Binder>);

#[derive(Debug, Clone)]
pub enum Binder {
    Variable(String, Type),
    Wildcard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Mod,
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
            Operator::Mod => write!(f, "%"),
        }
    }
}

pub fn from_typed_expr(expr: &TypedNode, context: &TypedProgram) -> Simple {
    match &expr.expr {
        scoped::SimplifiedExpression::FunctionCall(id, args) => match id.as_str() {
            "+" => Simple::Operation(
                Operator::Add,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            "-" => Simple::Operation(
                Operator::Sub,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            "*" => Simple::Operation(
                Operator::Mul,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            "/" => Simple::Operation(
                Operator::Div,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            ">" => Simple::Operation(
                Operator::Greater,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            "<" => Simple::Operation(
                Operator::Less,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            ">=" => Simple::Operation(
                Operator::GreaterOrEqual,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            "<=" => Simple::Operation(
                Operator::LessOrEq,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            "==" => Simple::Operation(
                Operator::Equal,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            "!=" => Simple::Operation(
                Operator::NotEqual,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            "%" => Simple::Operation(
                Operator::Mod,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            _ => match context.constructors.get(id) {
                Some(cons) => {
                    if args.0.is_empty() {
                        Simple::Int(cons.sibling_index as i64, from_exp_type(&expr.data.data))
                    } else {
                        Simple::Constructor(
                            cons.sibling_index as i64,
                            args.0
                                .iter()
                                .map(|arg| from_typed_expr(arg, context))
                                .collect(),
                            from_exp_type(&expr.data.data),
                        )
                    }
                }
                None => Simple::App(
                    id.clone(),
                    args.0
                        .iter()
                        .map(|arg| from_typed_expr(arg, context))
                        .collect(),
                    from_exp_type(&expr.data.data),
                ),
            },
        },
        scoped::SimplifiedExpression::Integer(i) => Simple::Int(*i, from_exp_type(&expr.data.data)),
        scoped::SimplifiedExpression::Variable(id) => {
            Simple::Ident(id.clone(), from_exp_type(&expr.data.data))
        }
        scoped::SimplifiedExpression::Match(var_node, cases) => Simple::Match(
            Simple::Ident(var_node.expr.clone(), from_exp_type(&var_node.data.data)).into(),
            cases
                .iter()
                .map(|(pattern, exp)| {
                    (
                        {
                            match pattern {
                                ast::Pattern::Integer(_) => todo!(),
                                ast::Pattern::Variable(_) => todo!(),
                                ast::Pattern::Constructor(fid, vars) => {
                                    if vars.0.is_empty() {
                                        (
                                            context.constructors.get(fid).unwrap().sibling_index
                                                as i64,
                                            vec![],
                                        )
                                    } else {
                                        (
                                            context.constructors.get(fid).unwrap().sibling_index
                                                as i64,
                                            vars.0
                                                .iter()
                                                .enumerate()
                                                .map(|(i, var)| {
                                                    Binder::Variable(var.clone(), {
                                                        context.constructors.get(fid).unwrap();
                                                        from_type(
                                                            &context
                                                                .constructors
                                                                .get(fid)
                                                                .unwrap()
                                                                .args
                                                                .0[i],
                                                        )
                                                    })
                                                })
                                                .collect(),
                                        )
                                    }
                                }
                            }
                        },
                        from_typed_expr(exp, context),
                    )
                })
                .collect(),
            from_exp_type(&expr.data.data),
        ),
        scoped::SimplifiedExpression::UTuple(args) => Simple::UTuple(
            args.0
                .iter()
                .map(|arg| from_typed_expr(arg, context))
                .collect(),
            from_exp_type(&expr.data.data),
        ),
        scoped::SimplifiedExpression::LetEqualIn(bindings, exp, next) if bindings.0.len() == 1 => {
            Simple::Let(
                bindings.0[0].clone(),
                from_typed_expr(exp, context).into(),
                from_typed_expr(next, context).into(),
                from_exp_type(&expr.data.data),
            )
        }
        scoped::SimplifiedExpression::LetEqualIn(bindings, exp, next) => Simple::LetApp(
            bindings.0.clone(),
            from_typed_expr(exp, context).into(),
            from_typed_expr(next, context).into(),
            from_exp_type(&expr.data.data),
        ),
    }
}

pub fn from_exp_type(typ: &ExpressionType) -> Type {
    match typ {
        ExpressionType::UTuple(vec) => Type::Unboxed(vec.0.iter().map(from_type).collect()),
        ExpressionType::Type(typ) => from_type(typ),
    }
}

pub fn from_type(typ: &ast::Type) -> Type {
    match typ {
        ast::Type::Int => Type::Int,
        ast::Type::ADT(_) => Type::Heaped,
    }
}

pub fn get_type(expr: &Simple) -> Type {
    match expr {
        Simple::Ident(_, typ) => typ.clone(),
        Simple::Int(_, typ) => typ.clone(),
        Simple::Operation(_, _, _, typ) => typ.clone(),
        Simple::Constructor(_, _, typ) => typ.clone(),
        Simple::App(_, _, typ) => typ.clone(),
        Simple::Match(_, _, typ) => typ.clone(),
        Simple::Let(_, _, _, typ) => typ.clone(),
        Simple::UTuple(_, typ) => typ.clone(),
        Simple::LetApp(_, _, _, typ) => typ.clone(),
    }
}
