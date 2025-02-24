use std::fmt::{Debug, Display, Formatter, Write};

use rand::distr::Alphanumeric;
use rand::distr::SampleString;

// A program consists of a list of definitions
#[derive(Debug, Clone)]
pub struct Program(pub Vec<Definition>);

pub type FID = String; // Function ID, (also including ADT constructors)
pub type VID = String; // Variable ID
pub type AID = String; // ADT ID

#[derive(Debug, Clone)]
pub enum Definition {
    ADTDefinition(ADTDefinition),
    FunctionDefinition(FunctionDefinition)
}

#[derive(Debug, Clone)]
pub struct ADTDefinition {
    pub id: AID,
    pub constructors: Vec<ConstructorDefinition> 
}

#[derive(Debug, Clone)]
pub struct ConstructorDefinition {
    pub id: FID,
    pub argument: Type
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TupleType(pub Vec<Type>); // Type of a tuple is a list of other types
// Any general type

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Tuple(TupleType),
    Int,
    ADT(AID)
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub id: FID,
    pub body: Expression,
    pub variable: VID,
    pub signature: FunctionSignature
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub argument_type: Type,
    pub result_type: Type,
    pub is_fip: bool
}

// Expressions are something that has a result, for example
// 1
// 4 * 5
// Cons (0, Nil)
// (1, 5, Nil)
#[derive(Debug, Clone)]
pub enum Expression {
    Tuple(TupleExpression),
    FunctionCall(FID, Box<Expression>),
    Integer(i64),
    Variable(VID),
    ADTMatch(ADTMatchExpression),
    TupleMatch(TupleMatchExpression),
    Operation(Operator, Box<Expression>, Box<Expression>)
}

#[derive(Debug, Clone)]
pub enum Operator {
    Equal, NotEqual, Less, LessOrEqual, Greater, GreaterOrEqual,
    Add, Sub, Mul, Div 
}

// An expression to create a tuple is a list of other expressions
#[derive(Debug, Clone)]
pub struct TupleExpression(pub Vec<Expression>);

#[derive(Debug, Clone)]
pub struct ADTMatchExpression {
    pub variable: VID, // What to match on
    pub cases: Vec<ADTMatchCase>
}

// A case in a match statement
#[derive(Debug, Clone)]
pub struct ADTMatchCase {
    pub cons_id: FID,
    pub var: VID,
    pub body: Expression // Code to execute if the case matches
}

#[derive(Debug, Clone)]
pub struct TupleMatchExpression {
    pub match_var: VID, // What to match on
    pub pattern_vars: Vec<VID>,
    pub body: Box<Expression> // Code to execute if the case matches
}

pub fn generate_wildcard_vid() -> VID {
    Alphanumeric.sample_string(&mut rand::rng(), 16)
}

// ====== Pretty Print Code ======

fn write_indent(f: &mut Formatter, indent: usize) -> std::fmt::Result {
    write!(f, "{}", "    ".repeat(indent))
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for definition in &self.0 {
            writeln!(f, "{definition}")?;
            writeln!(f)?;
        }

        Ok(())
    }
}

impl Display for Definition {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Definition::ADTDefinition(def) => write!(f, "{def}"),
            Definition::FunctionDefinition(def) => write!(f, "{def}")
        }
    }
}

impl Display for ADTDefinition {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "enum {} = \n", self.id)?;
        write_indent(f, 1)?;

        write!(f, "{}", self.constructors[0])?;
        for cons in self.constructors.iter().skip(1) {
            writeln!(f, ",")?;
            write_indent(f, 1)?;
            write!(f, "{cons}")?;
        }

        write!(f, ";")?;

        Ok(())
    }
}

impl Display for ConstructorDefinition {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.id, self.argument)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::ADT(id) => write!(f, "{}", id),
            Type::Tuple(tuple_type) => write!(f, "{tuple_type}")
        }
    }
}

impl Display for TupleType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;

        write_separated_list(f, self.0.iter(), ", ", |f, t| {
            write!(f, "{t}")
        })?;

        write!(f, ")")
    }
}

impl Display for FunctionDefinition {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f, "{}", self.signature)?;
        write!(f, "{} {} = ", self.id, self.variable)?;

        write_expression(f, &self.body, 1)?;
        write!(f, ";")
    }
}

impl Display for FunctionSignature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_fip { write!(f, "fip ")?; }

        write!(f, "{}: {}", self.argument_type, self.result_type)
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_expression(f, self, 0)
    }
}

fn write_separated_list<T>(
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

// Counts how long the text would've been
struct WriteCounter<'a> {
    width: &'a mut usize
}

impl<'a> Write for WriteCounter<'a> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        *self.width += s.len();
        Ok(())
    }
}

const MAX_EXPRESSION_WIDTH: usize = 20;
fn write_expression(f: &mut Formatter, expression: &Expression, indent: usize) -> std::fmt::Result {
    let mut inline_width: usize = 0;
    let counter = &mut WriteCounter { width: &mut inline_width };
    let mut counter = f.options().create_formatter(counter);
    write_expression_inline(&mut counter, expression, indent)?;

    if inline_width <= MAX_EXPRESSION_WIDTH {
        write_expression_inline(f, expression, indent)
    } else {
        write_expression_multiline(f, expression, indent)
    }
}

fn write_expression_inline(f: &mut Formatter, expression: &Expression, indent: usize) -> std::fmt::Result {
    match expression {
        Expression::Integer(x) => write!(f, "{x}"),
        Expression::Variable(id) => write!(f, "{id}"),
        Expression::Tuple(tuple) => {
            write!(f, "(")?;

            write_separated_list(f, tuple.0.iter(), ", ", |f, e| {
                write_expression(f, e, indent)
            })?;

            write!(f, ")")
        }
        Expression::FunctionCall(id, arg) => {
            write!(f, "{id} ")?;
            write_expression(f, arg, indent)
        },
        Expression::Operation(op, e1, e2) => {
            let symbol = match op {
                Operator::Add => "+",
                Operator::Sub => "-",
                Operator::Div => "/",
                Operator::Mul => "*",
                Operator::Greater => ">",
                Operator::GreaterOrEqual => ">=",
                Operator::Less => "<",
                Operator::LessOrEqual => "<=",
                Operator::Equal => "==",
                Operator::NotEqual => "!="
            };

            write_expression(f, e1, indent)?;
            write!(f, " {} ", symbol)?;
            write_expression(f, e2, indent)
        },
        Expression::ADTMatch(match_expr) => {
            write!(f, "match x {{ ")?;
            write_separated_list(f, match_expr.cases.iter(), ", ", |f, case| write!(f, "{case} "))?;
            write!(f, "}}")
        },
        Expression::TupleMatch(match_expr) => {
            write!(f, "match {} (", match_expr.match_var)?;

            write_separated_list(f, match_expr.pattern_vars.iter(), ", ", |f, t| {
                write!(f, "{t}")
            })?;

            write!(f, "): ")?;
            write_expression(f, &match_expr.body, indent)
        }
    }
}

fn write_expression_multiline(f: &mut Formatter, expression: &Expression, indent: usize) -> std::fmt::Result {
    match expression {
        Expression::Operation(op, e1, e2) => {
            let symbol = match op {
                Operator::Add => "+",
                Operator::Sub => "-",
                Operator::Div => "/",
                Operator::Mul => "*",
                Operator::Greater => ">",
                Operator::GreaterOrEqual => ">=",
                Operator::Less => "<",
                Operator::LessOrEqual => "<=",
                Operator::Equal => "==",
                Operator::NotEqual => "!="
            };

            write_expression(f, e1, indent)?;
            writeln!(f, " {symbol}")?;
            write_indent(f, indent)?;
            write_expression(f, e2, indent)
        },
        Expression::ADTMatch(match_expr) => {
            writeln!(f, "match x {{")?;

            write_separated_list(f, match_expr.cases.iter(), ",\n", |f, case| {
                write_indent(f, indent+1)?;
                write_adt_match_case(f, case, indent+1)
            })?;

            writeln!(f)?;
            write_indent(f, indent)?;
            write!(f, "}}")
            
        },
        Expression::Tuple(tuple) => {
            writeln!(f, "(")?;

            write_separated_list(f, tuple.0.iter(), ",\n", |f, e| {
                write_indent(f, indent+1)?;
                write_expression(f, e, indent+1)
            })?;
    
            writeln!(f)?;
            write_indent(f, indent)?;
            write!(f, ")")
        }
        _ => write_expression_inline(f, expression, indent)
    }
}

fn write_adt_match_case(f: &mut Formatter, case: &ADTMatchCase, indent: usize) -> std::fmt::Result {
    write!(f, "{} {}: ", case.cons_id, case.var)?;
    write_expression(f, &case.body, indent + 1)
}

impl Display for ADTMatchCase {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_adt_match_case(f, self, 0)
    }
}