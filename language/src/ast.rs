use std::collections::HashSet;
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
    pub arguments: ConstructorSignature
}

pub type ConstructorSignature = UTuple<Type>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UTuple<T>(pub Vec<T>); // Type of an unboxed tuple is a list of other types

// Any general type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    ADT(AID)
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub id: FID,
    pub body: Expression,
    pub variables: UTuple<VID>,
    pub signature: FunctionSignature
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub argument_type: UTuple<Type>,
    pub result_type: UTuple<Type>,
    pub is_fip: bool
}

// Expressions are something that has a result, for example
// 1
// 4 * 5
// Cons (0, Nil)
// (1, 5, Nil)
#[derive(Debug, Clone)]
pub enum Expression {
    UTuple(UTuple<Expression>),
    FunctionCall(FID, UTuple<Expression>),
    Integer(i64),
    Variable(VID),
    Match(MatchExpression),
    LetEqualIn(UTuple<VID>, Box<Expression>, Box<Expression>) // First expression may only be a function invocation. I don't know how to enforce this by type system without making everything messy.
}

#[derive(Debug, Clone)]
pub struct MatchExpression {
    pub variable: VID, // What to match on
    pub cases: Vec<MatchCase>
}

// A case in a match statement
#[derive(Debug, Clone)]
pub struct MatchCase {
    pub cons_id: FID,
    pub vars: UTuple<VID>,
    pub body: Expression // Code to execute if the case matches
}

pub fn generate_wildcard_vid() -> VID {
    Alphanumeric.sample_string(&mut rand::rng(), 16)
}

impl Definition {
    pub fn fids(&self) -> Vec<&VID> {
        match self {
            Definition::ADTDefinition(def) => def.constructors.iter().map(|cons| &cons.id).collect(),
            Definition::FunctionDefinition(def) => vec![&def.id],
        }
    }

    pub fn aid(&self) -> Option<&AID> {
        match self {
            Definition::ADTDefinition(def) => Some(&def.id),
            Definition::FunctionDefinition(_) => None,
        }
    }
}

impl Program {
    // Checks that there are no top level id conflicts
    pub fn validate_top_level_ids(&self) {
        let mut top_level_fids = HashSet::new();
        let mut top_level_aids = HashSet::new();

        for def in &self.0 {
            for fid in def.fids() {
                if !top_level_fids.insert(fid) {
                    panic!("Name collision for function {}", fid);
                }
            }

            if let Some(aid) = def.aid() {
                if !top_level_aids.insert(aid) {
                    panic!("Name collision for ADT {}", aid);
                }
            }
        }
    }
}
    
// ====== Pretty Print Code ======

pub fn write_indent(f: &mut Formatter, indent: usize) -> std::fmt::Result {
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
        write!(f, "{} {}", self.id, self.arguments)
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

impl Display for FunctionDefinition {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f, "{}", self.signature)?;
        write!(f, "{}{} = ", self.id, self.variables)?;

        write_expression(f, &self.body, 1)?;
        write!(f, ";")
    }
}

impl<T : Display> Display for UTuple<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_implicit_utuple(f, &self.0, ", ", |f, t| write!(f, "{t}"))
    }
}

impl Display for FunctionSignature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_fip { write!(f, "fip")?; }

        write!(f, "{}:{}", self.argument_type, self.result_type)
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_expression(f, self, 0)
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

pub fn write_implicit_utuple<T>(
        f: &mut Formatter, 
        items: &Vec<T>,
        separator: &str,
        write: impl Fn(&mut Formatter, &T) -> std::fmt::Result
    ) -> std::fmt::Result
{
    if items.len() == 0 { Ok(()) }
    else if items.len() == 1 { 
        write(f, &items[0]) 
    } else {
        write!(f, "(")?;
        write_separated_list(f, items.iter(), separator, write)?;
        write!(f, ")")
    }
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

const MAX_EXPRESSION_WIDTH: usize = 30;
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
        Expression::UTuple(tuple) => {
            write!(f, "(")?;

            write_separated_list(f, tuple.0.iter(), ", ", |f, e| {
                write_expression(f, e, indent)
            })?;

            write!(f, ")")
        }
        Expression::FunctionCall(id, arg) => {
            write!(f, "{id} ")?;
            write_implicit_utuple(f, &arg.0, ", ", |f, e| write_expression_inline(f, e, indent))
        },
        Expression::Match(match_expr) => {
            write!(f, "match x {{ ")?;
            write_separated_list(f, match_expr.cases.iter(), ", ", |f, case| write!(f, "{case} "))?;
            write!(f, "}}")
        },
        Expression::LetEqualIn(vars, e1, e2) => {
            write!(f, "let ")?;
            write_implicit_utuple(f, &vars.0, ", ", |f, vid| write!(f, "{vid}"))?;
            write!(f, " = ", )?;
            write_expression_inline(f, e1, indent)?;
            write!(f, " in ")?;
            write_expression_inline(f, e2, indent)
        }
    }
}

fn write_expression_multiline(f: &mut Formatter, expression: &Expression, indent: usize) -> std::fmt::Result {
    match expression {
        Expression::FunctionCall(id, arg) => {
            write!(f, "{id} ")?;
            write_implicit_utuple(f, &arg.0, ",", |f, e| {
                writeln!(f)?;
                write_indent(f, indent+1)?;
                write_expression_multiline(f, e, indent+1)
            })
        },
        Expression::Match(match_expr) => {
            writeln!(f, "match x {{")?;

            write_separated_list(f, match_expr.cases.iter(), ",\n", |f, case| {
                write_indent(f, indent+1)?;
                write_adt_match_case(f, case, indent+1)
            })?;

            writeln!(f)?;
            write_indent(f, indent)?;
            write!(f, "}}")
            
        },
        Expression::UTuple(tuple) => {
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

fn write_adt_match_case(f: &mut Formatter, case: &MatchCase, indent: usize) -> std::fmt::Result {
    write!(f, "{}{}: ", case.cons_id, case.vars)?;
    write_expression(f, &case.body, indent + 1)
}

impl Display for MatchCase {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_adt_match_case(f, self, 0)
    }
}