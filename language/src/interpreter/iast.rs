use crate::compiler::core::{Def, Operand, Statement};
use crate::compiler::simple::Operator;
use itertools::Itertools;
use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Debug, Clone)]
pub enum IOperand {
    Ident(String),
    Negate(String),
    Int(i64),
}

impl IOperand {
    pub fn from_op(operand: &Operand) -> Self {
        match operand {
            Operand::Ident(id) => Self::Ident(id.clone()),
            Operand::Int(i) => Self::Int(*i),
            Operand::NonShifted(i) => Self::Int(*i),
            Operand::Negate(id) => IOperand::Negate(id.clone()),
        }
    }

    pub fn unwrap_id(&self) -> String {
        match self {
            IOperand::Ident(s) => s.clone(),
            IOperand::Int(_) => panic!("Not an identifier"),
            IOperand::Negate(_) => panic!("Not an identifier"),
        }
    }

    pub fn unwrap_int(&self) -> i64 {
        match self {
            IOperand::Ident(_) => panic!("Not an int"),
            IOperand::Int(i) => *i,
            IOperand::Negate(_) => panic!("Not an int"),
        }
    }
}

impl Display for IOperand {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            IOperand::Ident(id) => write!(f, "{id}"),
            IOperand::Int(i) => write!(f, "{i}"),
            IOperand::Negate(id) => write!(f, "!{id}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum IStatement {
    IfExpr(Vec<(IOperand, Vec<IStatement>)>),
    Return(IOperand),
    Print(IOperand),
    AssignMalloc(String, u32),
    Assign(String, IOperand),
    AssignToField(String, i64, IOperand),
    AssignFromField(String, i64, IOperand),
    AssignBinaryOperation(String, Operator, IOperand, IOperand),
    AssignTagCheck(String, bool, IOperand, i64),
    FunctionCall(String, Vec<IOperand>),
    AssignReturnvalue(String),
    AssignDropReuse(String, String),
    Inc(IOperand),
    Dec(IOperand),
    AssignUTuple(usize, String, Vec<String>),
    DecUTuple(String),
    AssignUTupleField(String, usize, IOperand),
}

fn from_statements(statements: Vec<Statement>) -> Vec<IStatement> {
    let mut istatements = Vec::new();
    for statement in statements {
        let s = match statement {
            Statement::IfElse(items) => IStatement::IfExpr(
                items
                    .into_iter()
                    .map(|(operand, _statements)| {
                        (IOperand::from_op(&operand), from_statements(_statements))
                    })
                    .collect(),
            ),
            Statement::Return(operand) => IStatement::Return(IOperand::from_op(&operand)),
            Statement::Print(operand) => IStatement::Return(IOperand::from_op(&operand)),
            Statement::AssignMalloc(_, id, n) => IStatement::AssignMalloc(id, n as u32 + 3),
            Statement::Assign(_, id, operand) => {
                IStatement::Assign(id, IOperand::from_op(&operand))
            }
            Statement::AssignToField(id, i, operand) => {
                IStatement::AssignToField(id, i, IOperand::from_op(&operand))
            }
            Statement::AssignFromField(id, i, operand) => {
                IStatement::AssignFromField(id, i, IOperand::from_op(&operand))
            }
            Statement::AssignBinaryOperation(id, operator, operand, operand1) => {
                IStatement::AssignBinaryOperation(
                    id.clone(),
                    operator.clone(),
                    IOperand::from_op(&operand),
                    IOperand::from_op(&operand1),
                )
            }
            Statement::AssignTagCheck(id, b, operand, i) => {
                IStatement::AssignTagCheck(id, b, IOperand::from_op(&operand), i >> 1)
            }
            Statement::AssignFunctionCall(id, fid, operands, _) => {
                // first add a function call that puts the returned value in a register
                istatements.push(IStatement::FunctionCall(
                    fid.clone(),
                    operands.iter().map(IOperand::from_op).collect(),
                ));
                // then assign the value to the identifier
                IStatement::AssignReturnvalue(id.clone())
            }
            Statement::AssignDropReuse(a, b) => IStatement::AssignDropReuse(a, b),
            Statement::Inc(operand) => IStatement::Inc(IOperand::Ident(operand)),
            Statement::Dec(operand) => IStatement::Dec(IOperand::Ident(operand)),
            Statement::AssignUTuple(n, id, fields) => {
                IStatement::AssignUTuple(n as usize, id, fields)
            }
            Statement::DecUTuple(id, _) => IStatement::DecUTuple(id),
            Statement::AssignUTupleField(id, i, op) => {
                IStatement::AssignUTupleField(id, i as usize, IOperand::from_op(&op))
            }
        };
        istatements.push(s);
    }
    istatements
}

impl Display for IStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IStatement::IfExpr(items) => write!(
                f,
                "IfExpr {:?}",
                items
                    .iter()
                    .map(|(operand, _)| format!("{operand}"))
                    .collect_vec()
            ),
            IStatement::AssignMalloc(id, s) => write!(f, "{id} = malloc({s})"),
            IStatement::Return(ioperand) => write!(f, "Return({ioperand})"),
            IStatement::Print(ioperand) => write!(f, "Print({})", ioperand),
            IStatement::Inc(ioperand) => write!(f, "Inc({})", ioperand),
            IStatement::Dec(ioperand) => write!(f, "Dec({})", ioperand),
            IStatement::Assign(id, ioperand) => write!(f, "{id} = {}", ioperand),
            IStatement::AssignToField(id, ix, ioperand) => write!(f, "{id}[{ix}] = {}", ioperand),
            IStatement::AssignFromField(id, ix, ioperand) => write!(f, "{id} = {}[{ix}]", ioperand),
            IStatement::AssignBinaryOperation(id, operator, ioperand, ioperand1) => {
                write!(f, "{id} = {} {operator} {}", ioperand, ioperand1)
            }
            IStatement::AssignTagCheck(id, b, ioperand, i) => {
                write!(
                    f,
                    "{id} = {}",
                    if *b {
                        format!("tag({}) == {}", ioperand, i)
                    } else {
                        format!("{} == {}", ioperand, i)
                    }
                )
            }
            IStatement::FunctionCall(id, ioperands) => write!(
                f,
                "call {id}{:?}",
                ioperands.iter().map(|iop| format!("{iop}")).collect_vec()
            ),
            IStatement::AssignReturnvalue(id) => write!(f, "{id} = _ret_"),
            IStatement::AssignDropReuse(id1, id2) => write!(f, "DropReuse {} {}", id1, id2),
            IStatement::AssignUTuple(_, id, items) => write!(f, "{} = {:?}", id, items),
            IStatement::DecUTuple(id) => write!(f, "DecUTuple({})", id),
            IStatement::AssignUTupleField(id, i, ioperand) => {
                write!(f, "{} = {}.{}", id, ioperand, i)
            }
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
        IDef {
            id: def.id.clone(),
            args: def.args.clone(),
            body: from_statements(def.body.clone()),
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
                        writeln!(f, "{} {}:", if i == 0 { "if" } else { "else if" }, operand)?;
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
                    write!(f, "{}", s)?;
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
