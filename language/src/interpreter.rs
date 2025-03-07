use crate::ir::*;
use crate::typed_ast::*;
use std::collections::HashMap;
use std::iter::Peekable;

fn extract_ifs<'a, T: Iterator<Item = &'a Statement>>(statements: &mut Peekable<T>) -> IStatement {
    let mut chain = Vec::new();

    loop {
        let condition = match statements.next().unwrap() {
            Statement::If(operand) | Statement::ElseIf(operand) => {
                let body = extract_body(statements);
                (operand.clone(), body)
            },
            _ => panic!("yolo"),
        };

        chain.push(condition);

        match statements.peek() {
            Some(Statement::ElseIf(_)) => {}
            _ => {
                break;
            }
        }
    }

    IStatement::IfExpr(chain)
}

fn extract_body<'a, T: Iterator<Item = &'a Statement>>(
    statements: &mut Peekable<T>,
) -> Vec<IStatement> {
    let mut istatements = Vec::new();
    while let Some(statement) = statements.peek() {
        let x = match statement {
            Statement::If(_) => extract_ifs(statements),
            Statement::ElseIf(_) => todo!(),
            Statement::EndIf => {
                statements.next();
                break;
            }
            Statement::Decl(s) => {
                statements.next();
                IStatement::Decl(s.clone())
            }
            Statement::InitConstructor(s, i) => {
                statements.next();
                IStatement::InitConstructor(s.clone(), *i)
            }
            Statement::AssignField(s, i, o) => {
                statements.next();
                IStatement::AssignField(s.clone(), *i, o.clone())
            }
            Statement::Assign(b, s, operand) => {
                statements.next();
                IStatement::Assign(*b, s.clone(), operand.clone())
            }
            Statement::Return(o) => {
                statements.next();
                IStatement::Return(o.clone())
            }
            Statement::Print(o) => {
                statements.next();
                IStatement::Print(o.clone())
            }
        };
        istatements.push(x);
    }

    istatements
}

#[derive(Debug, Clone)]
pub struct IDef {
    pub id: String,
    pub args: Vec<String>,
    pub body: Vec<IStatement>,
}

impl IDef {
    pub fn from_def(def: &Def) -> Self {
        let mut iter = def.body.iter().peekable();
        let body = extract_body(&mut iter);
        IDef {
            id: def.id.clone(),
            args: def.args.clone(),
            body: body,
        }
    }
}

#[derive(Debug, Clone)]
pub enum IStatement {
    Decl(String),
    IfExpr(Vec<(Operand, Vec<IStatement>)>),
    InitConstructor(String, i64),
    AssignField(String, i64, Operand),
    Assign(bool, String, Operand),
    Return(Operand),
    Print(Operand),
}

pub struct Interpreter {
    functions: HashMap<String, IDef>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            functions: HashMap::new(),
        }
    }

    pub fn with_fn(mut self, function: IDef) -> Self {
        self.functions.insert(function.id.clone(), function);
        self
    }

    pub fn run_fn(&self, name: &str, passed_args: Vec<i64>) -> i64 {
        let function = self.functions.get(name).unwrap();

        // fill local vars with passed arguments
        let mut local_vars: HashMap<String, i64> = HashMap::new();
        for (name, val) in function.args.iter().zip(passed_args) {
            local_vars.insert(name.clone(), val);
        }

        let mut statements = function.body.iter().peekable();
        while let Some(shit) = statements.next() {
            match shit {
                IStatement::Decl(_) => (), // does nothing
                IStatement::InitConstructor(_, _) => todo!(),
                IStatement::AssignField(name, val, operand) => todo!(),
                IStatement::Assign(_, name, operand) => {
                    local_vars.insert(name.clone(), self.eval_op(operand.clone(), &local_vars));
                }
                IStatement::IfExpr(bruh) => {
                    for (op, code) in bruh {

                    }
                },
                IStatement::Return(operand) => return self.eval_op(operand.clone(), &local_vars),
                IStatement::Print(operand) => todo!(),
            }
        }

        0
    }

    fn eval_op(&self, op: Operand, local_variables: &HashMap<String, i64>) -> i64 {
        match op {
            Operand::Identifier(name) => *local_variables.get(&name).expect(&format!("Cant find identifier {}", name)),
            Operand::BinOp(operator, operand, operand1) => {
                let left = self.eval_op(*operand, local_variables);
                let right = self.eval_op(*operand1, local_variables);
                match operator {
                    Operator::Equal => (left == right) as i64,
                    Operator::NotEqual => (left != right) as i64,
                    Operator::Less => (left < right) as i64,
                    Operator::LessOrEq => (left <= right) as i64,
                    Operator::Greater => (left > right) as i64,
                    Operator::GreaterOrEqual => (left >= right) as i64,
                    Operator::Add => left + right,
                    Operator::Sub => left - right,
                    Operator::Mul => left * right,
                    Operator::Div => left / right,
                }
            }
            Operand::Integer(i) => i,
            Operand::Application(name, operands) => {
                let arguments: Vec<i64> = operands
                    .iter()
                    .map(|op| self.eval_op(op.clone(), local_variables))
                    .collect();

                self.run_fn(&name, arguments)
            }
            Operand::DerefField(_, _) => todo!(),
        }
    }
}
