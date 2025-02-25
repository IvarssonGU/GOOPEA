use crate::ast::Operator;


pub type Program = Vec<Definition>;

#[derive(Debug, Clone)]
pub struct Definition {
    pub t: Type,
    pub id: String,
    pub args: Vec<(Type, String)>,
    pub statements: Vec<Statement>
}

#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Adt
}

#[derive(Debug, Clone)]
pub enum Statement {
    Decl(Type, String),
    Init(Type, String, Expression),
    Return(Expression),
    If(Expression),
    ElseIf(Expression),
    EndIf,
    Assign(Expression, Expression),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Integer(i32),
    Ident(String),
    MallocAdt,
    MallocInt,
    Malloc(u32),
    DerefInt(Box<Expression>, u32),
    AccessData(Box<Expression>, u32),
    AccessTag(Box<Expression>),
    Application(String, Vec<Expression>),
    Operation(Box<Expression>, Operator, Box<Expression>)
}