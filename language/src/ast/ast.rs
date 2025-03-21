use std::{collections::HashMap, fmt::{Display, Formatter}, iter, ops::Deref};

use crate::error::CompileResult;

use super::{scoped::Scope, typed::ExpressionType};

pub type FID = String; // Function ID, (also including ADT constructors)
pub type VID = String; // Variable ID
pub type AID = String; // ADT ID

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    ADT(AID)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UTuple<T>(pub Vec<T>);

#[derive(Debug)]
pub struct Program<D, E> {
    pub adts: HashMap<AID, Vec<FID>>,
    pub constructors: HashMap<FID, Constructor>,
    pub functions: HashMap<FID, Function<D, E>>,
}

#[derive(Debug, Clone)]
pub struct Constructor {
    pub adt: AID,
    pub sibling_index: usize,
    pub args: UTuple<Type>
}

#[derive(Debug)]
pub struct Function<D, E> {
    pub vars: UTuple<VID>,
    pub signature: FunctionSignature,
    pub body: ExpressionNode<D, E>,
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub argument_type: UTuple<Type>,
    pub result_type: UTuple<Type>,
    pub is_fip: bool
}

#[derive(Debug)]
pub struct ExpressionNode<D, E> {
    pub expr: E,
    pub data: D
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

#[derive(Debug)]
pub enum FullExpression<'a, D, E> {
    UTuple(&'a UTuple<ExpressionNode<D, E>>),
    FunctionCall(&'a FID, &'a UTuple<ExpressionNode<D, E>>),
    Constructor(&'a FID, &'a UTuple<ExpressionNode<D, E>>),
    Integer(&'a i64),
    Variable(&'a VID),
    Match(&'a Box<ExpressionNode<D, E>>, &'a Vec<(Pattern, ExpressionNode<D, E>)>),
    LetEqualIn(&'a Pattern, &'a Box<ExpressionNode<D, E>>, &'a Box<ExpressionNode<D, E>>),
    Operation(&'a Box<ExpressionNode<D, E>>, &'a Operator, &'a Box<ExpressionNode<D, E>>)
}

#[derive(Debug, Clone)]
pub struct ChainedData<D, P> {
    pub data: D,
    pub next: P
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Integer(i64),
    Constructor(FID, UTuple<VID>),
    UTuple(UTuple<VID>)
}

impl<D, E> ExpressionNode<D, E> {
    pub fn new(data: D, expr: E) -> Self {
        ExpressionNode { data, expr }
    }
}

impl<D, P> Deref for ChainedData<D, P> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

/*
// Neat code to dynamically get data T from a chained data.
// Turned out to to be impractical

pub trait ChainedDataTrait<T> {
    fn data(&self) -> Option<&T>;
}

impl<D : Any, P : Any, T: 'static> ChainedDataTrait<T> for ChainedData<D, P> {
    fn data(&self) -> Option<&T> {
        (&self.data as &dyn Any).downcast_ref::<T>()
        .or((&self.next as &dyn Any).downcast_ref::<T>())
        .or((&self.next as &dyn Any).downcast_ref::<Box<dyn ChainedDataTrait<T>>>().and_then(|x| x.data()))
    }
}*/

impl<D, E> Program<D, E>
    where for<'a> &'a E: Into<FullExpression<'a, D, E>>
{
    pub fn validate_expressions_by(&self, validate: impl Fn(&ExpressionNode<D, E>) -> CompileResult) -> CompileResult {
        for func in self.functions.values() {
            func.body.validate_recursively_by(&validate)?;
        }

        Ok(())
    }
}

impl<D, E> Program<D, E> {
    pub fn map<E2: From<E>>(self) -> Program<D, E2> {
        Program { 
            adts: self.adts, 
            constructors: self.constructors, 
            functions: self.functions.into_iter().map(
                |(fid, func)| {
                    (fid, Function {
                        signature: func.signature,
                        vars: func.vars,
                        body: func.body.map()
                    })
                }
            ).collect::<HashMap<_, _>>() 
        }
    }
}

impl<D, E> ExpressionNode<D, E> {
    pub fn map<E2: From<E>>(self) -> ExpressionNode<D, E2> {
        ExpressionNode { 
            expr: self.expr.into(), 
            data: self.data 
        }
    }
}

impl<D, E> UTuple<ExpressionNode<D, E>> {
    pub fn map<E2: From<E>>(self) -> UTuple<ExpressionNode<D, E2>> {
        UTuple(map_expr_vec(self.0))
    }
}

pub fn map_expr_vec<D, E, E2: From<E>>(vec: Vec<ExpressionNode<D, E>>) -> Vec<ExpressionNode<D, E2>> {
    vec.into_iter().map(|x| x.map()).collect::<Vec<_>>()
}

pub fn map_expr_box<D, E, E2: From<E>>(x: Box<ExpressionNode<D, E>>) -> Box<ExpressionNode<D, E2>> {
    Box::new(x.map())
}

impl<D, E> ExpressionNode<D, E>
    where for<'a> &'a E: Into<FullExpression<'a, D, E>>
{
    pub fn children<'b>(&'b self) -> Box<dyn Iterator<Item = &'b Self> + 'b> {
        match (&self.expr).into() {
            FullExpression::UTuple(utuple) |
            FullExpression::FunctionCall(_, utuple) |
            FullExpression::Constructor(_, utuple) => Box::new(utuple.0.iter()),
            FullExpression::Integer(_) | FullExpression::Variable(_) => Box::new(iter::empty()),
            FullExpression::Match(expression_node, items) 
                => Box::new(iter::once(expression_node.as_ref()).chain(items.iter().map(|tup| &tup.1))),
            FullExpression::LetEqualIn(_, e1, e2) |
            FullExpression::Operation(e1, _, e2) => Box::new(iter::once(e1.as_ref()).chain(iter::once(e2.as_ref()))),
        }
    }

    pub fn validate_recursively_by(&self, validate: &impl Fn(&Self) -> CompileResult) -> CompileResult {
        validate(&self)?;

        for child in self.children() {
            child.validate_recursively_by(validate)?;
        }

        Ok(())
    }
}

// ==== PRETTY PRINT CODE ====

pub fn write_indent(f: &mut Formatter, indent: usize) -> std::fmt::Result {
    write!(f, "{}", "    ".repeat(indent))
}

impl<D: DisplayData, E> Display for Program<D, E>
    where for<'a> &'a E: Into<FullExpression<'a, D, E>>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (aid, constructors) in &self.adts {
            writeln!(f, "enum {aid} = ")?;
            write_separated_list(f, constructors.iter(), ",\n", |f, fid| {
                let args = &self.constructors[fid].args;

                write_indent(f, 1)?;
                write!(f, "{fid}{args}")
            })?;

            writeln!(f)?;
            writeln!(f)?;
        }

        for (fid, func) in &self.functions {
            writeln!(f, "{}\n{fid}{} =", func.signature, func.vars)?;
            write_expression_node(f, &func.body, 1)?;
            write!(f, ";")?;
            writeln!(f)?;
            writeln!(f)?;
        }

        Ok(())
    }
}

fn write_expression_node<D: DisplayData, E>(f: &mut Formatter<'_>, node: &ExpressionNode<D, E>, indent: usize) -> std::fmt::Result
where for<'a> &'a E: Into<FullExpression<'a, D, E>>
{
    node.data.fmt(f, indent)?;

    match (&node.expr).into() {
        FullExpression::UTuple(utuple) => {
            write_indent(f, indent)?;
            write!(f, "(")?;

            if utuple.0.len() > 0 {
                write_separated_list(f, utuple.0.iter(), ",", |f, x| {
                    writeln!(f)?;
                    write_expression_node(f, x, indent+1)
                })?;
                writeln!(f)?;

                write_indent(f, indent)?;
            }

            write!(f, ")")?;

            Ok(())
        },
        FullExpression::FunctionCall(fid, utuple) |
        FullExpression::Constructor(fid, utuple) => {
            write_indent(f, indent)?;

            write!(f, "{fid}(",)?;

            if utuple.0.len() > 0 {
                write_separated_list(f, utuple.0.iter(), ",", |f, x| {
                    writeln!(f)?;
                    write_expression_node(f, x, indent+1)
                })?;
                writeln!(f)?;

                write_indent(f, indent)?;
            }

            write!(f, ")")?;

            Ok(())
        },
        FullExpression::Integer(x) => { 
            write_indent(f, indent)?;
            write!(f, "{x}") 
        },
        FullExpression::Variable(var) => {
            write_indent(f, indent)?;
            write!(f, "{var}")
        },
        FullExpression::Match(expression_node, items) => {
            write_indent(f, indent)?;
            writeln!(f, "match")?;
            write_expression_node(f, expression_node, indent + 1)?;

            writeln!(f)?;
            write_indent(f, indent)?;
            writeln!(f, "{{")?;

            write_separated_list(f, items.iter(), ",\n", |f, (pattern, body)| {
                write_indent(f, indent + 1)?;
                writeln!(f, "{pattern}:")?;
                write_expression_node(f, body, indent + 2)
            })?;

            writeln!(f)?;
            write_indent(f, indent)?;
            write!(f, "}}")?;

            Ok(())
        },
        FullExpression::LetEqualIn(pattern, e1, e2) => {
            write_indent(f, indent)?;
            writeln!(f, "let {pattern} = ")?;
            write_expression_node(f, e1, indent + 1)?;

            writeln!(f)?;
            write_indent(f, indent)?;
            writeln!(f, "in")?;
            write_expression_node(f, e2, indent + 1)?;

            Ok(())
        }
        FullExpression::Operation(e1, op, e2) => {
            write_expression_node(f, e1, indent + 1)?;

            writeln!(f)?;
            write_indent(f, indent)?;
            writeln!(f, "{op}")?;

            write_expression_node(f, e2, indent + 1)?;

            Ok(())
        },
    }
}

trait DisplayData {
    fn fmt(&self, f: &mut Formatter<'_>, indent: usize) -> std::fmt::Result;
}

impl DisplayData for () {
    fn fmt(&self, _: &mut Formatter<'_>, _: usize) -> std::fmt::Result {
        Ok(())
    }
}

impl DisplayData for Scope {
    fn fmt(&self, f: &mut Formatter<'_>, indent: usize) -> std::fmt::Result {
        write_indent(f, indent)?;
        write!(f, "// scope: {{")?;
        write_separated_list(f, self.iter(), ", ", |f, (_, val)| { write!(f, "{}|{}", val.internal_id, val.id) })?;
        writeln!(f, "}}")
    }
}

impl DisplayData for ExpressionType {
    fn fmt(&self, f: &mut Formatter<'_>, indent: usize) -> std::fmt::Result {
        write_indent(f, indent)?;
        writeln!(f, "// type: {}", self)
    }
}

impl Display for ExpressionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressionType::UTuple(utuple) => write!(f, "{utuple}"),
            ExpressionType::Type(tp) => write!(f, "{tp}"),
        }
    }
}

impl<A: DisplayData, B: DisplayData> DisplayData for ChainedData<A, B> {
    fn fmt(&self, f: &mut Formatter<'_>, indent: usize) -> std::fmt::Result {
        self.next.fmt(f, indent)?;
        self.data.fmt(f, indent)
    }
}

impl<D: DisplayData, E> Display for ExpressionNode<D, E>
where for<'a> &'a E: Into<FullExpression<'a, D, E>>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_expression_node(f, self, 0)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::ADT(id) => write!(f, "{}", id)
        }
    }
}

impl Display for FunctionSignature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_fip { write!(f, "fip ")?; }

        write!(f, "{}:{}", self.argument_type, self.result_type)
    }
}

impl<T : Display> Display for UTuple<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_implicit_utuple(f, &self.0, ", ", |f, t| write!(f, "{t}"))
    }
}

pub fn write_implicit_utuple<T>(
    f: &mut Formatter, 
    items: &Vec<T>,
    separator: &str,
    write: impl Fn(&mut Formatter, &T) -> std::fmt::Result
) -> std::fmt::Result
{
    if items.len() == 0 { Ok(()) }
    else {
        write!(f, "(")?;
        write_separated_list(f, items.iter(), separator, write)?;
        write!(f, ")")
    }
}


pub fn write_separated_list<T>(
    f: &mut Formatter, 
    iter: impl Iterator<Item = T>, 
    separator: &str,
    write: impl Fn(&mut Formatter, T) -> std::fmt::Result
) -> std::fmt::Result 
{
    let mut iter = iter.peekable();
    while let Some(item) = iter.next() {
        write(f, item)?;

        if iter.peek().is_some() { write!(f, "{separator}")?; }
    }

    Ok(())
}

impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Pattern::Integer(x) => write!(f, "{x}"),
            Pattern::Constructor(fid, vars) => {
                write!(f, "{}", fid)?;
                write_implicit_utuple(f, &vars.0, ", ", |f, vid| write!(f, "{vid}"))
            },
            Pattern::UTuple(utuple) => {
                write_implicit_utuple(f, &utuple.0, ", ", |f, vid| { write!(f, "{vid}") })
            },
        }
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str: &'static str = self.into();
        write!(f, "{str}")
    }
}

impl TryFrom<&str> for Operator {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "==" => Operator::Equal,
            "!=" => Operator::NotEqual,
            "<=" => Operator::LessOrEq,
            ">=" => Operator::GreaterOrEqual,
            "<" => Operator::Less,
            ">" => Operator::Greater,
            "+" => Operator::Add,
            "-" => Operator::Sub,
            "*" => Operator::Mul,
            "/" => Operator::Div,
            _ => return Err(())
        })
    }
}

impl From<&Operator> for &'static str {
    fn from(value: &Operator) -> Self {
        match value {
            Operator::Equal => "==",
            Operator::NotEqual => "!=",
            Operator::LessOrEq => "<=",
            Operator::GreaterOrEqual => ">=",
            Operator::Less => "<",
            Operator::Greater => ">",
            Operator::Add => "+",
            Operator::Sub => "-",
            Operator::Mul => "*",
            Operator::Div => "/",
        }
    }
}