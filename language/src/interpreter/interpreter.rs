use super::iast::*;
use crate::ast::base::BaseSliceProgram;
use crate::ast::scoped::ScopedProgram;
use crate::ast::typed::TypedProgram;
use crate::ir::Prog;
use crate::simple_ast::{Operator, add_refcounts, from_scoped};
use crate::{code, ir};
use input::*;
use itertools::Itertools;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
use std::{fmt, vec};

#[cfg(not(target_arch = "wasm32"))]
use std::fs;

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

#[derive(Clone)]
pub struct Interpreter {
    functions: HashMap<String, IDef>,
    heap: Vec<Vec<Data>>,
    function_names_stack: Vec<String>,
    statements: VecDeque<IStatement>,
    statement_stack: Vec<VecDeque<IStatement>>,
    local_variables: HashMap<String, Data>,
    variable_stack: Vec<HashMap<String, Data>>,
    return_value: Option<Data>,
    steps: u64,
}
// init
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
            steps: 0,
        }
    }

    pub fn from_program(program: &Prog) -> Self {
        let mut interpreter = Interpreter::new();
        for def in program.0.clone() {
            interpreter = interpreter.with_fn(IDef::from_def(&def));
        }
        interpreter = interpreter.with_entry_point("main");
        interpreter
    }

    pub fn with_fn(mut self, function: IDef) -> Self {
        self.functions.insert(function.id.clone(), function);
        self
    }

    pub fn with_entry_point(mut self, function_name: &str) -> Self {
        self.enter_fn(function_name, vec![]);
        self
    }
}
// running
impl Interpreter {
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

    fn inc(&mut self, ptr: usize) {
        self.heap[ptr][2].inc();
    }

    fn clean_memory(&mut self) {
        // could make much more clean
        let i = self.heap.iter().rposition(|x| !x.is_empty()).unwrap_or(0);
        self.heap.truncate(i + 1);
    }

    fn dec(&mut self, ptr: usize) {
        self.heap[ptr][2].dec();
        if self.heap[ptr][2].unwrap_val() == 0 {
            for i in 3..self.heap[ptr].len() {
                if let Data::Pointer(ptr) = self.heap[ptr][i] {
                    self.dec(ptr);
                }
            }

            self.heap[ptr] = Vec::new();
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

    pub fn step(&mut self) -> Option<IStatement> {
        let s = self.statements.pop_front();
        if let Some(statement) = s.clone() {
            self.steps += 1;
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
                IStatement::Print(ioperand) => println!("> {}", self.eval_op(&ioperand)),
                IStatement::Inc(ioperand) => {
                    let id = ioperand.unwrap_id();
                    let data = self.get_local_var(&id);
                    if let Data::Pointer(ptr) = data {
                        self.inc(ptr);
                    }
                }
                IStatement::Dec(ioperand) => {
                    let id = ioperand.unwrap_id();
                    let data = self.get_local_var(&id);
                    if let Data::Pointer(ptr) = data {
                        self.dec(ptr);
                    }
                    self.clean_memory();
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
        }
        s
    }
}
// running until
impl Interpreter {
    pub fn run_until_next_mem(&mut self) {
        while let Some(s) = self.statements.get(0) {
            match s {
                IStatement::InitConstructor(_, _)
                | IStatement::Inc(_)
                | IStatement::Dec(_)
                | IStatement::AssignToField(_, _, _) => {
                    break;
                }
                _ => {
                    self.step();
                }
            }
        }
    }

    pub fn run_until_done(&mut self) {
        while let Some(_) = self.step() {}
    }

    pub fn run_until_return(&mut self) {
        let s = self.function_names_stack.len();

        while self.function_names_stack.len() >= s {
            self.step();
        }
    }
}
// website interactions
impl Interpreter {

}

fn concat_columns(left: &Vec<String>, right: &Vec<String>, sep: &str) -> Vec<String> {
    let wleft = left.iter().map(|s| s.len()).max().unwrap_or(0);
    let wright = right.iter().map(|s| s.len()).max().unwrap_or(0);

    left.iter()
        .zip_longest(right.iter())
        .map(|e| match e {
            itertools::EitherOrBoth::Both(a, b) => format!("{:<wleft$}{sep}{:<wright$}", a, b),
            itertools::EitherOrBoth::Left(a) => format!("{:<wleft$}{sep}{:<wright$}", a, ""),
            itertools::EitherOrBoth::Right(b) => format!("{:<wleft$}{sep}{:<wright$}", "", b),
        })
        .collect_vec()
}

impl Debug for Interpreter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{:=^50}",
            format!(
                " Interpreter Debug Print | Inside '{}' ",
                self.function_names_stack.last().unwrap_or(&"".to_string())
            )
        )?;

        let bruh = format!("{}", self.heap.len()).len();
        let heap_lines = vec!["Heap data:".to_string()]
            .into_iter()
            .chain(
                self.heap
                    .iter()
                    .enumerate()
                    .map(|(i, m)| format!("{:>bruh$}  {:?}", i, m)),
            )
            .collect_vec();

        let mut vars_lines = vec!["Local variables:".to_string()]
            .into_iter()
            .chain(
                self.local_variables
                    .iter()
                    .map(|(k, v)| format!("{} = {:?}", k, v)),
            )
            .collect_vec();
        if let Some(v) = self.return_value {
            vars_lines.push(format!("Return value: {:?}", v));
        }

        let combined = concat_columns(&heap_lines, &vars_lines, " | ");

        let statements_lines = self
            .statements
            .iter()
            .map(|s| format!("{}", s))
            .chain(vec!["...".to_string()].into_iter().cycle())
            .take(15)
            .collect_vec();

        let combined = concat_columns(&combined, &statements_lines, " | ");

        for line in &combined {
            writeln!(f, "{}", line).unwrap();
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
pub fn interpreter_test_time(src: &str) {
    let code = fs::read_to_string(Path::new(src)).unwrap();

    let base_program = BaseSliceProgram::new(&code).unwrap();
    let scoped_program = ScopedProgram::new(base_program).unwrap();
    let typed_program = TypedProgram::new(scoped_program).unwrap();

    let simple_program = from_scoped(&typed_program);
    let with_ref_count = add_refcounts(&simple_program);
    let code = code::Compiler::new().compile(&with_ref_count);

    let mut interpreter = Interpreter::from_program(&code);

    println!("Starting!");
    let now = std::time::Instant::now();

    interpreter.run_until_done();

    let elapsed = now.elapsed().as_micros();
    let steps = interpreter.steps;
    println!(
        "Done!\n{} steps in {} us\n{} sps",
        steps,
        elapsed,
        1_000_000 * steps / elapsed as u64
    );
    println!("{:?}", interpreter);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn interpreter_test(src: &str) {
    let code = fs::read_to_string(Path::new(src)).unwrap();
    let base_program = BaseSliceProgram::new(&code).unwrap();
    let scoped_program = ScopedProgram::new(base_program).unwrap();
    let typed_program = TypedProgram::new(scoped_program).unwrap();
    let simple_program = from_scoped(&typed_program);
    let with_ref_count = add_refcounts(&simple_program);
    let code = code::Compiler::new().compile(&with_ref_count);

    let c_code = ir::output(&code).join("\n");
    fs::write(Path::new(".interpreter_out/c_code.c"), c_code).unwrap();

    let mut c_ast = code.0.clone();
    c_ast.sort_by(|a, b| a.id.cmp(&b.id));
    let c_ast = c_ast
        .iter()
        .map(|def| {
            let statements = def
                .body
                .iter()
                .map(|s| format!("    {:?}", s))
                .collect_vec()
                .join("\n");
            format!("function {}{:?}:\n{}\n", def.id, def.args, statements)
        })
        .collect_vec()
        .join("\n");
    fs::write(Path::new(".interpreter_out/c_ast.txt"), c_ast).unwrap();

    let mut interpreter = Interpreter::from_program(&code);

    let mut i_ast = interpreter.functions.values().collect_vec().clone();
    i_ast.sort_by(|a, b| a.id.cmp(&b.id));
    let i_ast = i_ast
        .iter()
        .map(|idef| format!("{}\n", idef))
        .collect_vec()
        .join("\n");

    fs::write(Path::new(".interpreter_out/i_ast.txt"), i_ast).unwrap();

    loop {
        if let Some(_) = interpreter.statements.get(0) {
            println!("{:?}", interpreter);
            println!("m, r, enter");
            let input: String = input("");
            match input.as_str() {
                "m" => {
                    interpreter.run_until_next_mem();
                }
                "r" => {
                    interpreter.run_until_return();
                }
                _ => {
                    interpreter.step();
                }
            }
        } else {
            break;
        }
    }
}
