use std::fmt::{Debug, Display, Formatter, Result};

// A program consists of a list of definitions

#[derive(Debug, Clone)]
pub struct Program {
    pub adt_definitions: Vec<ADTDefinition>,
    pub fun_definitions: Vec<FunctionDefinition>
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FID(pub String); // Function ID, (also including ADT constructors)

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VID(pub String); // Variable ID

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AID(pub String); // ADT ID



#[derive(Debug, Clone)]
pub struct ADTDefinition {
    pub id: AID,
    pub constructors: Vec<ConstructorDefinition> 
}

#[derive(Debug, Clone)]
pub struct ConstructorDefinition {
    pub id: FID,
    pub argument: TupleType
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
    pub args: Vec<String>,
    pub body: Expression,
    pub signature: FunctionSignature
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub argument_type: TupleType,
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
    FunctionCall(FID, TupleExpression),
    Identifier(VID),
    Integer(i32),
    Match(MatchExpression, Type),
    Operation(Operator, Box<Expression>, Box<Expression>),
    Constructor(FID, Vec<Expression>)
}

#[derive(Debug, Clone)]
pub enum Operator {
    Equal, NotEqual, Less, LessOrEq, Greater, GreaterOrEqual,
    Add, Sub, Mul, Div 
}

// An expression to create a tuple is a list of other expressions
#[derive(Debug, Clone)]
pub struct TupleExpression(pub Vec<Expression>);

#[derive(Debug, Clone)]
pub struct MatchExpression {
    pub exp: Box<Expression>, // What to match on
    pub cases: Vec<MatchCase>
}

// A case in a match statement
#[derive(Debug, Clone)]
pub struct MatchCase {
    pub pattern: Pattern,
    pub body: Expression,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Identifier(VID),
    Integer(i32),
    Wildcard,
    Constructor(FID, Vec<Option<VID>>)
}

fn write_indent(f: &mut Formatter, indent: usize) -> std::fmt::Result {
    write!(f, "{}", "\t".repeat(indent))
}

/* impl Display for Program {
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
} */

impl Display for ADTDefinition {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "enum {} = \n", self.id.0)?;
        write_indent(f, 1)?;

        write!(f, "{}", self.constructors[0])?;
        for cons in self.constructors.iter().skip(1) {
            writeln!(f, ",")?;
            write_indent(f, 1)?;
            write!(f, "{cons}")?;
        }

        Ok(())
    }
}

impl Display for ConstructorDefinition {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.id.0, self.argument)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::ADT(id) => write!(f, "{}", id.0),
            Type::Tuple(tuple_type) => write!(f, "{tuple_type}")
        }
    }
}

impl Display for TupleType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;

        let mut iter = self.0.iter();
        if let Some(t) = iter.next() {
            write!(f, "{t}")?;
        }

        for t in iter {
            write!(f, ", {t}")?;
        }

        write!(f, ")")
    }
}

impl Display for FunctionDefinition {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        todo!()
    }
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