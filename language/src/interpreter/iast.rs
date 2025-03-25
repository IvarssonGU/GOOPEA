use crate::ir::*;
use crate::simple_ast::Operator;
use std::fmt::{Debug, Display, Formatter, Result};

use super::load::extract_body;

#[derive(Debug, Clone)]
pub enum IOperand {
    Ident(String),
    Int(i64),
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
            .filter(|&s| !matches!(s, Statement::Decl(_)))
            .filter(|&s| !matches!(s, Statement::Inc(Operand::Int(_))))
            .peekable();

        let body = extract_body(&mut iter);

        IDef {
            id: def.id.clone(),
            args: def.args.clone(),
            body: body,
        }
    }
}

impl Display for IDef {
    fn fmt(&self, f: &mut Formatter) -> Result {
        fn write_indent(f: &mut Formatter, s: IStatement, indent: usize) -> Result {
            match s {
                IStatement::IfExpr(items) => {
                    let n_cases = items.len();

                    for (i, (operand, statements)) in items.iter().enumerate() {
                        write!(f, "{}", "    ".repeat(indent))?;

                        writeln!(
                            f,
                            "{} {:?}:",
                            if i == 0 { "if" } else { "else if" },
                            operand
                        )?;

                        let n_statements = statements.len();

                        for (j, statement) in statements.iter().enumerate() {
                            write_indent(f, statement.clone(), indent + 1)?;

                            if i < n_cases - 1 || j < n_statements - 1 {
                                writeln!(f, "")?;
                            }
                        }
                    }
                }

                _ => {
                    write!(f, "{}", "    ".repeat(indent))?;

                    write!(f, "{:?}", s)?;
                }
            }

            Ok(())
        }

        writeln!(f, "function {}{:?}:", self.id, self.args)?;

        let len = self.body.len();

        for (i, statement) in self.body.iter().enumerate() {
            write_indent(f, statement.clone(), 1)?;

            if i < len - 1 {
                writeln!(f, "")?;
            }
        }

        Ok(())
    }
}
