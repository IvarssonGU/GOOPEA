use crate::ir::*;
use crate::simple_ast::Operator;
use std::fmt::{Debug, Display};

use super::load::extract_body;

#[derive(Debug, Clone)]
pub enum IOperand {
    Ident(String),
    Int(i64),
    // Atom(i64),
    // Pointer(usize),
}

#[allow(unused)]
impl IOperand {
    pub fn from_op(operand: &Operand) -> Self {
        match operand {
            Operand::Ident(id) => Self::Ident(id.clone()),
            Operand::Int(i) => Self::Int(*i),
            Operand::NonShifted(i) => Self::Int(*i),
        }
    }

    pub fn unwrap_id(&self) -> String {
        match self {
            IOperand::Ident(s) => s.clone(),
            IOperand::Int(_) => panic!("Not an identifier"),
        }
    }

    pub fn unwrap_int(&self) -> i64 {
        match self {
            IOperand::Ident(_) => panic!("Not an int"),
            IOperand::Int(i) => *i,
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum IStatement {
    Decl(String),
    IfExpr(Vec<(IOperand, Vec<IStatement>)>),
    InitConstructor(String, i64),
    Return(IOperand),
    Print(IOperand),
    Inc(IOperand),
    Dec(IOperand),
    Assign(String, IOperand),
    AssignToField(String, i64, IOperand),
    AssignFromField(String, i64, IOperand),
    AssignBinaryOperation(String, Operator, IOperand, IOperand),
    AssignConditional(String, bool, IOperand, i64),
    FunctionCall(String, Vec<IOperand>),
    AssignReturnvalue(String),
}

impl Display for IStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IStatement::IfExpr(items) => write!(
                f,
                "IfExpr {:?}",
                items.iter().map(|(operand, _)| operand).collect::<Vec<_>>()
            ),
            x => write!(f, "{:?}", x),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IDef {
    pub id: String,
    pub args: Vec<String>,
    pub body: Vec<IStatement>,
}

impl IDef {
    pub fn from_def(def: &Def) -> Self {
        let mut iter = def
            .body
            .iter()
            .filter(|&s| {
                if matches!(
                    s,
                    Statement::Inc(Operand::Int(_)) | Statement::Inc(Operand::NonShifted(_))
                ) {
                    false
                } else {
                    true
                }
            })
            .filter(|&s| !matches!(s, Statement::Decl(_)))
            .peekable();
        let body = extract_body(&mut iter);
        IDef {
            id: def.id.clone(),
            args: def.args.clone(),
            body: body,
        }
    }
}
