use crate::ast::base::BaseProgram;
use crate::ast::scoped::ScopedProgram;
use crate::ast::typed::TypedProgram;
use crate::lexer::Lexer;
use crate::simple_ast::{Operator, add_refcounts, from_scoped};
use crate::{code, grammar, ir};
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
use std::{fmt, vec};

#[cfg(not(target_arch = "wasm32"))]
use std::fs;

use super::iast::*;

#[derive(Clone, Copy)]
enum Data {
    Value(i64),
    Pointer(usize),
}

#[allow(unused)]
impl Data {
    fn is_val(&self) -> bool {
        match self {
            Data::Value(_) => true,
            Data::Pointer(_) => false,
        }
    }

    fn is_ptr(&self) -> bool {
        match self {
            Data::Value(_) => false,
            Data::Pointer(_) => true,
        }
    }

    fn unwrap_val(&self) -> i64 {
        match self {
            Data::Value(i) => *i,
            Data::Pointer(_) => panic!("Not a value"),
        }
    }

    fn unwrap_ptr(&self) -> usize {
        match self {
            Data::Value(_) => panic!("Not a pointer"),
            Data::Pointer(p) => *p,
        }
    }

    fn _unwrap_raw(&self) -> i64 {
        match self {
            Data::Value(i) => *i,
            Data::Pointer(p) => *p as i64,
        }
    }

    fn inc(&mut self) -> i64 {
        match self {
            Data::Value(i) => {
                *i += 1;
                self.unwrap_val()
            }
            Data::Pointer(_) => panic!("Not a value"),
        }
    }

    fn dec(&mut self) -> i64 {
        match self {
            Data::Value(i) => {
                *i -= 1;
                self.unwrap_val()
            }
            Data::Pointer(_) => panic!("Not a value"),
        }
    }
}

pub struct Interpreter {
    functions: HashMap<String, IDef>,
    heap: Vec<Vec<Data>>,
    function_names_stack: Vec<String>,
    statements: VecDeque<IStatement>,
    statement_stack: Vec<VecDeque<IStatement>>,
    local_variables: HashMap<String, Data>,
    variable_stack: Vec<HashMap<String, Data>>,
    return_value: Option<Data>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            functions: HashMap::new(),
            heap: vec![vec![]],
            function_names_stack: Vec::new(),
            statements: VecDeque::new(),
            statement_stack: Vec::new(),
            local_variables: HashMap::new(),
            variable_stack: Vec::new(),
            return_value: None,
        }
    }

    pub fn with_fn(mut self, function: IDef) -> Self {
        self.functions.insert(function.id.clone(), function);
        self
    }

    pub fn with_entry_point(mut self, function_name: &str) -> Self {
        self.enter_fn(function_name, vec![]);
        self
    }

    fn eval_op(&self, op: &IOperand) -> i64 {
        match op {
            IOperand::Ident(id) => self.get_local_var(id).unwrap_val(),
            IOperand::Int(i) => *i,
        }
    }

    fn op_to_data(&self, op: &IOperand) -> Data {
        match op {
            IOperand::Ident(id) => self.get_local_var(id),
            IOperand::Int(i) => Data::Value(*i),
        }
    }

    fn get_local_var(&self, id: &str) -> Data {
        self.local_variables
            .get(id)
            .expect("Variable not in scope")
            .clone()
    }

    fn malloc(&mut self, width: usize) -> Data {
        for n in 1..self.heap.len() {
            if self.heap[n].is_empty() {
                self.heap[n] = vec![Data::Value(0); width];
                return Data::Pointer(n);
            }
        }

        self.heap.push(vec![Data::Value(0); width]);
        Data::Pointer(self.heap.len() - 1)
    }

    pub fn step(&mut self) -> Result<(), ()> {
        let s = self.statements.pop_front();
        if let Some(statement) = s {
            match statement {
                IStatement::Decl(_) => (), // does nothing
                IStatement::IfExpr(items) => {
                    for (operand, statements) in items {
                        if self.eval_op(&operand) == 1 {
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
                    self.local_variables.insert(id.clone(), ptr);
                }
                IStatement::Return(ioperand) => {
                    self.return_value = Some(self.op_to_data(&ioperand));
                    self.statements = self.statement_stack.pop().expect("this should not happen");
                    self.local_variables =
                        self.variable_stack.pop().expect("this should not happen");
                    self.function_names_stack.pop();
                }
                IStatement::Print(ioperand) => println!("> {:?}", ioperand),
                IStatement::Inc(ioperand) => {
                    let id = ioperand.unwrap_id();
                    let ptr = self.get_local_var(&id);
                    if ptr.is_ptr() {
                        self.heap[ptr.unwrap_ptr()][2].inc();
                    }
                }
                IStatement::Dec(ioperand) => {
                    let id = ioperand.unwrap_id();
                    let ptr = self.get_local_var(&id);
                    if ptr.is_ptr() {
                        let block = &mut self.heap[ptr.unwrap_ptr()];
                        if block[2].dec() == 0 {
                            for _ in &block[3..] {
                                // println!("SOMETHING IS GOOONE")
                            }
                        }
                    }
                }
                IStatement::Assign(id, ioperand) => {
                    let val = self.op_to_data(&ioperand);
                    self.local_variables.insert(id, val);
                }
                IStatement::AssignToField(id, i, ioperand) => {
                    let ptr = self.get_local_var(&id).unwrap_ptr();
                    let val = self.op_to_data(&ioperand);
                    self.heap[ptr][i as usize] = val;
                }
                IStatement::AssignFromField(id, i, ioperand) => {
                    let name = ioperand.unwrap_id();
                    let ptr = self.get_local_var(&name).unwrap_ptr();
                    let val = self.heap[ptr][i as usize];
                    self.local_variables.insert(id, val);
                }
                IStatement::AssignBinaryOperation(id, operator, ioperand, ioperand1) => {
                    let lhs = self.eval_op(&ioperand);
                    let rhs = self.eval_op(&ioperand1);
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
                    self.local_variables.insert(id, Data::Value(val));
                }
                /*
                Statement::AssignConditional(id, b, op, tag) => {
                    let result = operand_to_string(op);
                    if *b {
                        format!("Value {} = !(1 & {}) && {} == ((void** {})[0];", id, result, tag, result)
                    } else {
                        format!("Value {} = {} == {};", id, tag, result)
                    }
                }
                */
                IStatement::AssignConditional(id, b, iop, i) => {
                    let i = i >> 1; // bruh
                    let val = if b {
                        let name = iop.unwrap_id();
                        let shit = self.get_local_var(&name);
                        shit.is_ptr() && i == self.heap[shit.unwrap_ptr()][0].unwrap_val()
                    } else {
                        i == self.op_to_data(&iop)._unwrap_raw()
                    } as i64;
                    self.local_variables.insert(id, Data::Value(val));
                }
                IStatement::FunctionCall(fid, ioperands) => {
                    self.enter_fn(&fid, ioperands.iter().map(|x| self.op_to_data(x)).collect());
                }
                IStatement::AssignReturnvalue(id) => {
                    self.local_variables.insert(id, self.return_value.unwrap());
                    self.return_value = None;
                }
            }

            Ok(())
        } else {
            Err(())
        }
    }

    fn enter_fn(&mut self, name: &str, passed_args: Vec<Data>) {
        let f = self.functions.get(name).expect(&format!(
            "Function '{}' should be in functions but is not",
            name
        ));
        self.function_names_stack.push(f.id.clone());
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

    fn _yolo(&mut self) {
        loop {
            println!("{:?}", self);
            use std::io::{Write, stdin, stdout};
            let mut s = String::new();
            let _ = stdout().flush();
            stdin()
                .read_line(&mut s)
                .expect("Did not enter a correct string");
            self.step().unwrap();
        }
    }
}

impl Debug for Interpreter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:=^50}", " Interpreter Debug Print ")?;

        writeln!(f, "Heap:")?;
        let bruh = format!("{}", self.heap.len()).len();
        for (i, m) in self.heap.clone().iter().enumerate() {
            writeln!(f, "{:>bruh$}  {:?}", i, m)?;
        }
        writeln!(f, "")?;

        writeln!(f, "Local Variables:")?;
        for (k, v) in self.local_variables.clone().iter() {
            writeln!(f, "  {} = {:?}", k, v)?;
        }
        writeln!(f, "")?;
        writeln!(f, "Return value: {:?}\n", self.return_value)?;

        if !self.function_names_stack.is_empty() {
            writeln!(
                f,
                "Inside Function '{}'",
                self.function_names_stack
                    .last()
                    .unwrap()
            )?;
            writeln!(f, "Current Statements:")?;
            for s in self.statements.clone() {
                writeln!(f, "  {}", s)?;
            }
            writeln!(f, "")?;
        }

        writeln!(f, "Statement stack:")?;
        let sizes: Vec<_> = self.statement_stack.iter().map(|d| d.len()).collect();
        writeln!(f, "{:?}", sizes)?;

        Ok(())
    }
}

impl Debug for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Data::Value(i) => write!(f, "{}", i),
            Data::Pointer(p) => write!(f, "<{}>", p),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn interpreter_test_time() {
    let code = fs::read_to_string(Path::new("examples/tree_flip.goo")).unwrap();

    let base_program = BaseProgram::new(&code).unwrap();
    let scoped_program = ScopedProgram::new(base_program).unwrap();
    let typed_program = TypedProgram::new(scoped_program).unwrap();

    let simple_program = from_scoped(&typed_program);
    let with_ref_count = add_refcounts(&simple_program);
    let code = code::Compiler::new().compile(&with_ref_count);

    ir::output(&code)
        .iter()
        .for_each(|line| println!("{}", line));

    let mut interpreter = Interpreter::new();
    for def in code.0.clone() {
        interpreter = interpreter.with_fn(IDef::from_def(&def));
        println!("{}\n", IDef::from_def(&def));
    }
    interpreter = interpreter.with_entry_point("Main");

    println!("Starting!");
    let now = std::time::Instant::now();

    while let Ok(_) = interpreter.step() {}

    let elapsed = now.elapsed().as_micros();
    println!("Done! ({} us)", elapsed);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn interpreter_test_tree_flip() {
    let code = fs::read_to_string(Path::new("examples/tree_flip.goo")).unwrap();

    let program = grammar::ProgramParser::new()
        .parse(Lexer::new(&code))
        .unwrap();

    let base_program = BaseProgram::new(&code).unwrap();
    let scoped_program = ScopedProgram::new(base_program).unwrap();
    let typed_program = TypedProgram::new(scoped_program).unwrap();

    let simple_program = from_scoped(&typed_program);
    let with_ref_count = add_refcounts(&simple_program);
    let code = code::Compiler::new().compile(&with_ref_count);

    ir::output(&code)
        .iter()
        .for_each(|line| println!("{}", line));

    let mut interpreter = Interpreter::new();
    for def in code.0.clone() {
        interpreter = interpreter.with_fn(IDef::from_def(&def));
        println!("{}\n", IDef::from_def(&def));
    }
    interpreter = interpreter.with_entry_point("Main");

    interpreter._yolo();

    interpreter.step().unwrap();

    println!("{:?}", interpreter);

    for _ in 0..50 {
        interpreter.step().unwrap();
    }
    // end of build
    println!("build end\n{:?}", interpreter);

    for _ in 0..224 {
        interpreter.step().unwrap();
    }
    // end of flip
    println!("flip end\n{:?}", interpreter);

    for _ in 0..175 {
        interpreter.step().unwrap();
    }
    // end of sum
    println!("sum end\n{:?}", interpreter);

    interpreter.step();
    println!("main end\n{:?}", interpreter);
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
