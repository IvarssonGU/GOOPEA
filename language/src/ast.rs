use std::fmt::{Debug, Display, Formatter};

// A program consists of a list of definitions
#[derive(Debug, Clone)]
pub struct Program(pub Vec<Definition>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FID(pub String); // Function ID, (also including ADT constructors)

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VID(pub String); // Variable ID

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AID(pub String); // ADT ID

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
    pub body: Expression,
    pub variables: Vec<VID>, // Variables that gets populated by tuple
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
    FunctionCall(FID, Box<TupleExpression>),
    Integer(i32),
    Variable(VID),
    Match(MatchExpression),
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
pub struct MatchExpression {
    pub variable: VID, // What to match on
    pub cases: Vec<MatchCase>
}

// A case in a match statement
#[derive(Debug, Clone)]
pub struct MatchCase {
    pub cons_id: FID,
    pub vars: Vec<VID>, // Variables that gets populated by tuple
    pub body: Expression // Code to execute if the case matches
}

// ====== Pretty Print Code ======

fn write_indent(f: &mut Formatter, indent: usize) -> std::fmt::Result {
    write!(f, "{}", "    ".repeat(indent))
}

impl Display for VID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for FID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for AID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
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

fn write_tuple<T: Display>(f: &mut Formatter<'_>, elements: &Vec<T>) -> std::fmt::Result {
    write!(f, "(")?;

    let mut iter = elements.iter();
    if let Some(t) = iter.next() {
        write!(f, "{t}")?;
    }

    for t in iter {
        write!(f, ", {t}")?;
    }

    write!(f, ")")
}

impl Display for TupleType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_tuple(f, &self.0)
    }
}

impl Display for FunctionDefinition {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f, "{}", self.signature)?;
        write!(f, "{} ", self.id)?;
        write_tuple(f, &self.variables)?;
        writeln!(f, " =")?;

        write_indent(f, 1)?;
        write_expression(f, &self.body, 1)?;
        writeln!(f)
    }
}

impl Display for FunctionSignature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_fip { write!(f, "fip ")?; }

        write!(f, "{}: {}", self.argument_type, self.result_type)
    }
}

fn expression_size(expression: &Expression) -> usize {
    match expression {
        Expression::Integer(x) => x.to_string().len(),
        Expression::Variable(vid) => vid.0.len(),
        Expression::FunctionCall(fid, expr) => fid.0.len() + 1 + tuple_expression_size(expr),
        Expression::Tuple(expr) => tuple_expression_size(expr),
        Expression::Operation(op, e1, e2) => {
            let op_size = match op {
                Operator::GreaterOrEqual|Operator::LessOrEqual|Operator::Equal|Operator::NotEqual => 2,
                _ => 1
            };

            op_size + expression_size(e1) + expression_size(e2)
        },
        _ => todo!()
    }
}

fn tuple_expression_size(expression: &TupleExpression) -> usize {
    2 + 2 * (expression.0.len() - 1) + expression.0.iter().map(|x| expression_size(x)).sum::<usize>()
}

const MAX_EXPRESSION_SIZE: usize = 20;
fn write_expression(f: &mut Formatter, expression: &Expression, indent: usize) -> std::fmt::Result {
    match expression {
        Expression::Integer(x) => write!(f, "{x}"),
        Expression::Variable(id) => write!(f, "{id}"),
        Expression::Tuple(tuple) => write_tuple_expression(f, tuple, indent),
        Expression::FunctionCall(id, arg) => {
            write!(f, "{id} ")?;
            write_tuple_expression(f, arg, indent)
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

            if expression_size(expression) > MAX_EXPRESSION_SIZE {
                write_expression(f, e1, indent)?;
                writeln!(f, " {symbol}")?;
                write_indent(f, indent)?;
                write_expression(f, e2, indent)
            } else {
                write_expression(f, e1, indent)?;
                write!(f, " {} ", symbol)?;
                write_expression(f, e2, indent)
            }
        },
        _ => todo!()
    }
}

fn write_tuple_expression(f: &mut Formatter, tuple: &TupleExpression, indent: usize) -> std::fmt::Result {
    if tuple_expression_size(tuple) > MAX_EXPRESSION_SIZE {
        writeln!(f, "(")?;

        let mut iter = tuple.0.iter();
        if let Some(e) = iter.next() {
            write_indent(f, indent+1)?;
            write_expression(f, e, indent+1)?;
        }

        for e in iter {
            writeln!(f, ",")?;
            write_indent(f, indent+1)?;
            write_expression(f, e, indent+1)?;
        }

        writeln!(f)?;
        write_indent(f, indent)?;
        write!(f, ")")
    } else {
        write!(f, "(")?;

        let mut iter = tuple.0.iter();
        if let Some(e) = iter.next() {
            write_expression(f, e, indent+1)?;
        }

        for e in iter {
            write!(f, ", ")?;
            write_expression(f, e, indent+1)?;
        }

        write!(f, ")")
    }
}