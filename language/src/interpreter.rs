use crate::lexer::Lexer;
use crate::scoped_ast::ScopedProgram;
use crate::simple_ast::{Operator, add_refcounts, from_scoped};
use crate::{code, grammar, ir::*};
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::iter::Peekable;
use std::path::Path;
use std::{fmt, fs};

fn extract_ifs<'a, T: Iterator<Item = &'a Statement>>(
    statements: &mut Peekable<T>,
) -> Vec<(IOperand, Vec<IStatement>)> {
    let mut chain = Vec::new();

    loop {
        let condition = match statements.next().unwrap() {
            Statement::If(operand) | Statement::ElseIf(operand) => {
                let body = extract_body(statements);
                (IOperand::from_op(operand), body)
            }
            Statement::Else => {
                let body = extract_body(statements);
                (IOperand::Int(1), body)
            }
            _ => panic!("yolo"),
        };

        chain.push(condition);

        match statements.peek() {
            Some(Statement::ElseIf(_)) | Some(Statement::Else) => {}
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
            Statement::ElseIf(_) => panic!("this should not happen"),
            Statement::Else => todo!(),
            Statement::EndIf => {
                statements.next();
                break;
            }
            Statement::Decl(id) => IStatement::Decl(id.clone()),
            Statement::InitConstructor(id, i) => IStatement::InitConstructor(id.clone(), *i),
            Statement::Return(operand) => IStatement::Return(IOperand::from_op(operand)),
            Statement::Print(operand) => IStatement::Print(IOperand::from_op(operand)),
            Statement::Inc(operand) => IStatement::Inc(IOperand::from_op(operand)),
            Statement::Dec(operand) => IStatement::Dec(IOperand::from_op(operand)),
            Statement::Assign(_, id, operand) => {
                IStatement::Assign(id.clone(), IOperand::from_op(operand))
            }
            Statement::AssignToField(id, i, operand) => {
                IStatement::AssignToField(id.clone(), *i, IOperand::from_op(operand))
            }
            Statement::AssignFromField(id, i, operand) => {
                IStatement::AssignFromField(id.clone(), *i, IOperand::from_op(operand))
            }
            Statement::AssignBinaryOperation(id, operator, operand, operand1) => {
                IStatement::AssignBinaryOperation(
                    id.clone(),
                    operator.clone(),
                    IOperand::from_op(operand),
                    IOperand::from_op(operand1),
                )
            }
            Statement::AssignConditional(id, b, operand, i) => {
                IStatement::AssignConditional(id.clone(), *b, IOperand::from_op(operand), *i)
            }
            Statement::AssignFunctionCall(id, f, operands) => {
                // first add a function call that puts the returned value in a register
                istatements.push(IStatement::FunctionCall(
                    f.clone(),
                    operands.iter().map(IOperand::from_op).collect(),
                ));
                // then assign the value to the identifier
                IStatement::AssignReturnvalue(id.clone())
            }
        };
        // consume if not if
        statements.next_if(|x| !matches!(x, Statement::If(_) | Statement::EndIf));
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
        let mut iter = def
            .body
            .iter()
            .filter(|&s| {
                if matches!(s, Statement::Inc(Operand::Int(_)) | Statement::Inc(Operand::NonShifted(_))) {
                    false
                } else {
                    true
                }
            })
            .peekable();
        let body = extract_body(&mut iter);
        IDef {
            id: def.id.clone(),
            args: def.args.clone(),
            body: body,
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

#[derive(Debug, Clone)]
pub enum IOperand {
    Ident(String),
    Int(i64),
    // Atom(i64),
    // Pointer(usize),
}
impl IOperand {
    fn from_op(operand: &Operand) -> Self {
        match operand {
            Operand::Ident(id) => Self::Ident(id.clone()),
            Operand::Int(i) => Self::Int(*i),
            Operand::NonShifted(i) => Self::Int(*i),
        }
    }
}

pub struct Interpreter {
    functions: HashMap<String, IDef>,
    heap: Vec<Vec<i64>>,
    current_function: String,
    statements: VecDeque<IStatement>,
    statement_stack: Vec<VecDeque<IStatement>>,
    local_variables: HashMap<String, i64>,
    variable_stack: Vec<HashMap<String, i64>>,
    return_value: IOperand,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            functions: HashMap::new(),
            heap: Vec::new(),
            current_function: "".to_string(),
            statements: VecDeque::new(),
            statement_stack: Vec::new(),
            local_variables: HashMap::new(),
            variable_stack: Vec::new(),
            return_value: IOperand::Int(0),
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

    pub fn step(&mut self) -> Result<(), ()> {
        let s = self.statements.pop_front();
        if let Some(statement) = s {
            match statement {
                IStatement::Decl(_) => todo!(), // does nothing
                IStatement::IfExpr(items) => {
                    for (operand, statements) in items {
                        if self.eval_op(&operand, &self.local_variables) == 1 {
                            // beautiful code🦀
                            // inside_if ++ old_code
                            let mut new_list = statements.clone();
                            new_list.extend(self.statements.clone().into_iter());
                            self.statements = new_list.into();
                            break;
                        }
                    }
                }
                IStatement::InitConstructor(id, w) => {
                    let ptr = self.malloc(w as usize);
                    self.local_variables.insert(id.clone(), ptr as i64);
                }
                IStatement::Return(ioperand) => {
                    self.return_value = ioperand;
                    self.statements = self.statement_stack.pop().expect("this should not happen");
                    self.local_variables =
                        self.variable_stack.pop().expect("this should not happen");
                }
                IStatement::Print(ioperand) => println!("> {:?}", ioperand),
                IStatement::Inc(ioperand) => {
                    if let IOperand::Ident(o) = ioperand {
                        let ptr = self.eval_op(&IOperand::Ident(o), &self.local_variables);
                        self.heap[ptr as usize][2] += 1;
                    }
                }
                IStatement::Dec(ioperand) => {
                    let ptr = self.eval_op(&ioperand, &self.local_variables);
                    let block = &mut self.heap[ptr as usize];
                    block[2] -= 1;
                    if block[2] == 0 {
                        for shit in &block[3..] {
                            // if shit == pointer
                        }
                    }
                }
                IStatement::Assign(id, ioperand) => {
                    self.local_variables
                        .insert(id, self.eval_op(&ioperand, &self.local_variables));
                }
                IStatement::AssignToField(id, i, ioperand) => {
                    let ptr = *self
                        .local_variables
                        .get(&id)
                        .expect("expected variable to be in scope");
                    let val = self.eval_op(&ioperand, &self.local_variables);
                    self.heap[ptr as usize][i as usize] = val;
                }
                IStatement::AssignFromField(id, i, ioperand) => {
                    let ptr = self.eval_op(&ioperand, &self.local_variables);
                    let val = self.heap[ptr as usize][i as usize];
                    self.local_variables.insert(id, val);
                }
                IStatement::AssignBinaryOperation(id, operator, ioperand, ioperand1) => {
                    let lhs = self.eval_op(&ioperand, &self.local_variables);
                    let rhs = self.eval_op(&ioperand1, &self.local_variables);
                    let val = match operator {
                        Operator::Equal => (lhs == rhs) as i64,
                        Operator::NotEqual => (lhs != rhs) as i64,
                        Operator::Less => (lhs < rhs) as i64,
                        Operator::LessOrEq => (lhs <= rhs) as i64,
                        Operator::Greater => (lhs > rhs) as i64,
                        Operator::GreaterOrEqual => (lhs >= rhs) as i64,
                        Operator::Add => lhs + rhs,
                        Operator::Sub => lhs - rhs,
                        Operator::Mul => lhs * rhs,
                        Operator::Div => lhs / rhs,
                    };
                    self.local_variables.insert(id, val);
                }
                IStatement::AssignConditional(_, _, ioperand, _) => todo!(),
                IStatement::FunctionCall(fid, ioperands) => {
                    self.enter_fn(
                        &fid,
                        ioperands
                            .iter()
                            .map(|x| self.eval_op(x, &self.local_variables))
                            .collect(),
                    );
                }
                IStatement::AssignReturnvalue(id) => {
                    self.local_variables
                        .insert(id, self.eval_op(&self.return_value, &self.local_variables));
                }
            }

            Ok(())
        } else {
            Err(())
        }
    }

    pub fn enter_fn(&mut self, name: &str, passed_args: Vec<i64>) {
        let f = self.functions.get(name).expect(&format!(
            "Function '{}' should be in functions but is not",
            name
        ));
        self.current_function = f.id.clone();
        // std::mem::take could make it faster
        self.statement_stack.push(self.statements.clone());
        self.variable_stack.push(self.local_variables.clone());

        self.statements = f.body.clone().into();
        // beautiful code🦀
        self.local_variables.clear();
        self.local_variables
            .extend(f.args.clone().into_iter().zip(passed_args));

        ()
    }

    pub fn run_fn(&mut self, name: &str, passed_args: Vec<i64>) {
        while let Ok(_) = self.step() {}
    }

    fn eval_op(&self, op: &IOperand, local_variables: &HashMap<String, i64>) -> i64 {
        match op {
            IOperand::Ident(id) => *local_variables
                .get(id)
                .expect("Expected variable to be in scope"),
            IOperand::Int(i) => *i,
        }
    }
}

impl Debug for Interpreter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Interpreter Debug Print!")?;

        writeln!(f, "Heap:")?;
        for m in self.heap.clone() {
            writeln!(f, "  {:?}", m)?;
        }
        writeln!(f, "Local Variables:")?;
        for (k, v) in self.local_variables.clone().iter() {
            writeln!(f, "  {} = {:?}", k, v)?;
        }
        writeln!(f, "Inside Function '{}'", self.current_function)?;
        writeln!(f, "Current Statements:")?;
        for s in self.statements.clone() {
            writeln!(f, "  {:?}", s)?;
        }

        writeln!(f, "Statement stack:")?;
        let sizes: Vec<_> = self.statement_stack.iter().map(|d| d.len()).collect();
        writeln!(f, "{:?}", sizes)?;

        Ok(())
    }
}

pub fn interpreter_test() {
    let code = fs::read_to_string(Path::new("examples/tree_flip.goo")).unwrap();

    let program = grammar::ProgramParser::new()
        .parse(Lexer::new(&code))
        .unwrap();

    let scoped_program = ScopedProgram::new(&program).unwrap();
    scoped_program.validate().unwrap();
    let simple_program = from_scoped(&scoped_program);
    let with_ref_count = add_refcounts(&simple_program);
    let code = code::Compiler::new().compile(&with_ref_count);

    let mut interpreter = Interpreter::new();
    for def in code.0.clone() {
        println!("{:?}", def.id);
        for s in def.body.clone() {
            println!("{:?}", s);
        }
        interpreter = interpreter.with_fn(IDef::from_def(&def));
    }

    interpreter.enter_fn("Main", Vec::new());
    interpreter.step();

    println!("{:?}", interpreter);

    for _ in 0..47 {
        interpreter.step();
    }

    println!("{:?}", interpreter);
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
