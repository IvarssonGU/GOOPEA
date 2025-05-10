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
pub enum Crux {
    Ident(String, Type),
    Int(i64, Type),
    Operation(Operator, Box<Crux>, Box<Crux>, Type),
    Constructor(i64, Vec<Crux>, Type),
    App(String, Vec<Crux>, Type),
    Match(Box<Crux>, Vec<(Pattern, Crux)>, Type),
    Let(String, Box<Crux>, Box<Crux>, Type),
    UTuple(Vec<Crux>, Type),
    LetApp(Vec<String>, Box<Crux>, Box<Crux>, Type),
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

pub fn from_typed_expr(expr: &TypedNode, context: &TypedProgram) -> Crux {
    match &expr.expr {
        scoped::SimplifiedExpression::FunctionCall(id, args) => match id.as_str() {
            "+" => Crux::Operation(
                Operator::Add,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            "-" => Crux::Operation(
                Operator::Sub,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            "*" => Crux::Operation(
                Operator::Mul,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            "/" => Crux::Operation(
                Operator::Div,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            ">" => Crux::Operation(
                Operator::Greater,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            "<" => Crux::Operation(
                Operator::Less,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            ">=" => Crux::Operation(
                Operator::GreaterOrEqual,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            "<=" => Crux::Operation(
                Operator::LessOrEq,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            "==" => Crux::Operation(
                Operator::Equal,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            "!=" => Crux::Operation(
                Operator::NotEqual,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            "%" => Crux::Operation(
                Operator::Mod,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            _ => match context.constructors.get(id) {
                Some(cons) => {
                    if args.0.is_empty() {
                        Crux::Int(cons.sibling_index as i64, from_exp_type(&expr.data.data))
                    } else {
                        Crux::Constructor(
                            cons.sibling_index as i64,
                            args.0
                                .iter()
                                .map(|arg| from_typed_expr(arg, context))
                                .collect(),
                            from_exp_type(&expr.data.data),
                        )
                    }
                }
                None => Crux::App(
                    id.clone(),
                    args.0
                        .iter()
                        .map(|arg| from_typed_expr(arg, context))
                        .collect(),
                    from_exp_type(&expr.data.data),
                ),
            },
        },
        scoped::SimplifiedExpression::Integer(i) => Crux::Int(*i, from_exp_type(&expr.data.data)),
        scoped::SimplifiedExpression::Variable(id) => {
            Crux::Ident(id.clone(), from_exp_type(&expr.data.data))
        }
        scoped::SimplifiedExpression::Match(var_node, cases) => Crux::Match(
            Crux::Ident(var_node.expr.clone(), from_exp_type(&var_node.data.data)).into(),
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
        scoped::SimplifiedExpression::UTuple(args) => Crux::UTuple(
            args.0
                .iter()
                .map(|arg| from_typed_expr(arg, context))
                .collect(),
            from_exp_type(&expr.data.data),
        ),
        scoped::SimplifiedExpression::LetEqualIn(bindings, exp, next) if bindings.0.len() == 1 => {
            Crux::Let(
                bindings.0[0].clone(),
                from_typed_expr(exp, context).into(),
                from_typed_expr(next, context).into(),
                from_exp_type(&expr.data.data),
            )
        }
        scoped::SimplifiedExpression::LetEqualIn(bindings, exp, next) => Crux::LetApp(
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

pub fn get_type(expr: &Crux) -> Type {
    match expr {
        Crux::Ident(_, typ) => typ.clone(),
        Crux::Int(_, typ) => typ.clone(),
        Crux::Operation(_, _, _, typ) => typ.clone(),
        Crux::Constructor(_, _, typ) => typ.clone(),
        Crux::App(_, _, typ) => typ.clone(),
        Crux::Match(_, _, typ) => typ.clone(),
        Crux::Let(_, _, _, typ) => typ.clone(),
        Crux::UTuple(_, typ) => typ.clone(),
        Crux::LetApp(_, _, _, typ) => typ.clone(),
    }
}
