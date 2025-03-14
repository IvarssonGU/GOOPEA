use crate::ir::*;
use crate::simple_ast::Operator;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::iter::Peekable;

fn extract_ifs<'a, T: Iterator<Item = &'a Statement>>(
    statements: &mut Peekable<T>,
) -> Vec<(Operand, Vec<IStatement>)> {
    let mut chain = Vec::new();

    loop {
        let condition = match statements.next().unwrap() {
            Statement::If(operand) | Statement::ElseIf(operand) => {
                let body = extract_body(statements);
                (operand.clone(), body)
            }
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

    chain
}

fn extract_body<'a, T: Iterator<Item = &'a Statement>>(
    statements: &mut Peekable<T>,
) -> Vec<IStatement> {
    let mut istatements = Vec::new();
    while let Some(statement) = statements.peek() {
        let x = match statement {
            Statement::If(_) => IStatement::IfExpr(extract_ifs(statements)),
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
            Statement::Assign(_, s, operand) => {
                statements.next();
                IStatement::Assign(s.clone(), operand.clone())
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

impl fmt::Display for IDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn write_indent(f: &mut fmt::Formatter, s: IStatement, indent: usize) -> fmt::Result {
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

#[derive(Debug, Clone)]
pub enum IStatement {
    Decl(String),
    IfExpr(Vec<(Operand, Vec<IStatement>)>),
    InitConstructor(String, i64),
    AssignField(String, i64, Operand),
    Assign(String, Operand),
    Return(Operand),
    Print(Operand),
}

pub struct Interpreter {
    functions: HashMap<String, IDef>,
    heap: Vec<Vec<i64>>
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            functions: HashMap::new(),
            heap: Vec::new()
        }
    }

    pub fn with_fn(mut self, function: IDef) -> Self {
        self.functions.insert(function.id.clone(), function);
        self
    }

    fn malloc(&mut self, width: usize) -> usize {
        self.heap.push(vec![0; width]);
        self.heap.len() - 1
    }

    pub fn run_fn(&mut self, name: &str, passed_args: Vec<i64>) -> i64 {
        // clone to not borrow self
        let function = self.functions.get(name).unwrap().clone();

        // fill local vars with passed arguments
        let mut local_vars: HashMap<String, i64> = HashMap::new();
        for (name, val) in function.args.iter().zip(passed_args) {
            local_vars.insert(name.clone(), val);
        }

        let mut statements: VecDeque<IStatement> = function.body.clone();
        while let Some(shit) = statements.pop_front() {
            match shit {
                IStatement::Decl(_) => (), // does nothing
                IStatement::InitConstructor(name, width) => {
                    let pointer = self.malloc(*width as usize);
                    local_vars.insert(name.clone(), pointer as i64);
                },
                IStatement::AssignField(name, index, operand) => {
                    let pointer = local_vars.get(&name).unwrap();
                    self.heap[*pointer as usize][*index as usize] = self.eval_op(&operand, &local_vars);
                },
                IStatement::Assign(name, operand) => {
                    local_vars.insert(name.clone(), self.eval_op(&operand, &local_vars));
                }
                IStatement::IfExpr(bruh) => {
                    for (op, code) in bruh {
                        if self.eval_op(&op, &local_vars) == 1 {
                            statements = code.append(statements);
                            break;
                        }
                    }
                },
                IStatement::Return(operand) => return self.eval_op(&operand, &local_vars),
                IStatement::Print(operand) => println!("{}", self.eval_op(&operand, &local_vars)),
            }
        }

        0
    }

    fn eval_op(&mut self, op: &Operand, local_variables: &HashMap<String, i64>) -> i64 {
        match op {
            Operand::Identifier(name) => *local_variables
                .get(name)
                .expect(&format!("Identifier '{}' should be in scope but is not", name)),
            Operand::BinOp(operator, operand, operand1) => {
                let left = self.eval_op(operand, local_variables);
                let right = self.eval_op(operand1, local_variables);
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
            Operand::Integer(i) => *i,
            Operand::Application(name, operands) => {
                let arguments: Vec<i64> = operands
                    .iter()
                    .map(|op| self.eval_op(op, local_variables))
                    .collect();

                self.run_fn(&name, arguments)
            }
            Operand::DerefField(_, _) => todo!(),
            Operand::Condition(_, _, __, _) => todo!(),
            Operand::AccessField(_, _) => todo!(),
            Operand::UTuple(operands) => todo!(),
        }
    }
}

pub fn interpreter_test() {
    let shit = Def {
        id: "test_function".to_string(),
        args: Vec::new(),
        body: vec![],
        type_len: None,
    };

    println!("hello test");
}

