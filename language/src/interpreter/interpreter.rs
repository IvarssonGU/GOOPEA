use super::historymagic::{HMT, HistoryMagic};
use super::iast::*;
use super::mempeek::MemObj;
use crate::ast::base::BaseSliceProgram;
use crate::ast::scoped::ScopedProgram;
use crate::ast::typed::TypedProgram;
use crate::core::{Def, Operand, Prog, Statement};
use crate::score;
use crate::stir::{self, Operator};
use input::*;
use itertools::Itertools;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::time::Instant;

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
use std::{fmt, vec};

#[cfg(not(target_arch = "wasm32"))]
use std::fs;

#[derive(Clone, Copy)]
pub enum Data {
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
            heap: Vec::new(),
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
        for def in program.clone() {
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
            IOperand::Negate(id) => {
                if self.get_local_var(id)._unwrap_raw() == 0 {
                    1
                } else {
                    0
                }
            }
        }
    }

    fn op_to_data(&self, op: &IOperand) -> Data {
        match op {
            IOperand::Ident(id) => self.get_local_var(id),
            IOperand::Int(i) => Data::Value(*i),
            IOperand::Negate(id) => panic!("Hoppsan"),
        }
    }

    fn get_local_var(&self, id: &str) -> Data {
        self.local_variables
            .get(id)
            .expect("Variable not in scope")
            .clone()
    }

    fn malloc(&mut self, width: usize) -> Data {
        for n in 0..self.heap.len() {
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
                IStatement::AssignMalloc(id, w) => {
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
                IStatement::AssignTagCheck(id, b, iop, i) => {
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
                IStatement::AssignDropReuse(id, id1) => {
                    let reff = self.get_local_var(&id1);
                    let ptr = reff.unwrap_ptr();
                    if self.heap[ptr][2].unwrap_val() == 1 {
                        for i in 3..self.heap[ptr].len() {
                            if self.heap[ptr][i].is_ptr() {
                                self.dec(self.heap[ptr][i].unwrap_ptr());
                            }
                        }
                        self.local_variables.insert(id, Data::Pointer(ptr));
                    } else {
                        self.heap[ptr][2].dec();
                        self.local_variables.insert(id, Data::Value(0));
                    }
                }
            }
        }
        s
    }
}
// running until
impl Interpreter {
    pub fn run_until_next_mem(&mut self) {
        self.step();
        while let Some(s) = self.statements.get(0) {
            match s {
                IStatement::AssignMalloc(..)
                | IStatement::Inc(_)
                | IStatement::Dec(_)
                | IStatement::AssignToField(..) => {
                    break;
                }
                _ => {
                    self.step();
                }
            }
        }
    }

    pub fn run_until_next_ptr(&mut self) {
        self.step();
        while let Some(s) = self.statements.get(0) {
            if let IStatement::AssignMalloc(_, _) = s {
                break;
            } else if let IStatement::Dec(op) = s {
                match *op {
                    IOperand::Int(i) if i == 1 => {
                        break;
                    }
                    _ => (),
                }
            } else if let IStatement::AssignToField(_, _, op) = s {
                if self.op_to_data(&op).is_ptr() {
                    break;
                }
            }
            self.step();
        }
    }

    pub fn run_until_done(&mut self) {
        while let Some(_) = self.step() {}
    }

    pub fn run_until_return(&mut self) {
        let s = self.function_names_stack.len();

        while self.function_names_stack.len() >= s && !self.statements.is_empty() {
            self.step();
        }
    }

    pub fn run_step_over(&mut self) {
        let s = self.function_names_stack.len();
        self.step();
        while self.function_names_stack.len() > s && !self.statements.is_empty() {
            self.step();
        }
    }
}
// website interaction
impl Interpreter {
    pub fn get_memory_raw(&self) -> Vec<Vec<Data>> {
        self.heap.clone()
    }

    pub fn get_variables_raw(&self) -> Vec<(String, Data)> {
        let mut list = self.local_variables.clone().into_iter().collect_vec();
        list.sort_by(|(a, _), (b, _)| a.cmp(b));
        list
    }

    pub fn get_variable_json(&self, id: &str) -> String {
        if !self.local_variables.contains_key(id) {
            return "{}".to_string();
        }
        MemObj::from_data(&self.get_local_var(id), &self.heap).as_json()
    }
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
        .collect()
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
                self.get_variables_raw()
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
fn _compile(path: &str, fip: bool) -> Prog {
    let code = fs::read_to_string(Path::new(path)).unwrap();
    let base_program = BaseSliceProgram::new(&code).unwrap();
    let scoped_program = ScopedProgram::new(base_program).unwrap();
    let typed_program = TypedProgram::new(scoped_program).unwrap();
    let mut program = stir::from_typed(&typed_program);
    if fip {
        program = stir::add_reuse(&program);
    }
    let program = stir::add_rc(&program, true);
    let core_ir = score::translate(&program);
    core_ir
}

#[cfg(not(target_arch = "wasm32"))]
pub fn interpreter_test(src: &str) {
    println!("fip?");
    let core_ir = match input::<String>("").as_str() {
        "fip" => {
            println!("fip!");
            _compile(src, true)
        }
        _ => {
            println!("not fip =(");
            _compile(src, false)
        }
    };

    let mut interpreter = Interpreter::from_program(&core_ir);

    let mut history = Vec::new();
    loop {
        print!("{}[2J", 27 as char);
        println!("{:?}", interpreter);
        println!("m, r, s, b, p, enter");
        let input: String = input("");

        match input.as_str() {
            "m" => {
                history.push(interpreter.clone());
                interpreter.run_until_next_mem();
            }
            "r" => {
                history.push(interpreter.clone());
                interpreter.run_until_return();
            }
            "s" => {
                history.push(interpreter.clone());
                interpreter.run_step_over();
            }
            "b" => {
                interpreter = history.pop().unwrap();
            }
            "p" => {
                history.push(interpreter.clone());
                interpreter.run_until_next_ptr();
            }
            x if x.parse::<usize>().is_ok() => {
                let shit = MemObj::from_data(
                    &Data::Pointer(x.parse::<usize>().unwrap()),
                    &interpreter.heap,
                );
                println!("{}", shit.as_json());
            }
            x if interpreter.local_variables.contains_key(x) => {
                let json = interpreter.get_variable_json(x);
                println!("{}", json);
            }
            _ => {
                history.push(interpreter.clone());
                interpreter.step();
            }
        }
    }
}

impl HMT for Interpreter {
    fn next(&self) -> Self {
        let mut next = self.clone();
        next.step();
        next
    }
}

fn _run_until_next_mem(h: &mut HistoryMagic<Interpreter>) {
    h.next();
    while let Some(s) = h.get().statements.get(0) {
        match s {
            IStatement::AssignMalloc(..)
            | IStatement::Inc(_)
            | IStatement::Dec(_)
            | IStatement::AssignToField(..) => {
                break;
            }
            _ => {
                h.next();
            }
        }
    }
}

fn _run_until_next_ptr(h: &mut HistoryMagic<Interpreter>) {
    h.next();
    while let Some(s) = h.get().statements.get(0) {
        if let IStatement::AssignMalloc(_, _) = s {
            break;
        } else if let IStatement::Dec(op) = s {
            match *op {
                IOperand::Int(i) if i == 1 => {
                    break;
                }
                _ => (),
            }
        } else if let IStatement::AssignToField(_, _, op) = s {
            if h.get().op_to_data(&op).is_ptr() {
                break;
            }
        }
        h.next();
    }
}

fn _run_until_done(h: &mut HistoryMagic<Interpreter>) {
    while !h.get().statements.is_empty() {
        h.next();
    }
}

fn _run_until_return(h: &mut HistoryMagic<Interpreter>) {
    let s = h.get().function_names_stack.len();

    while h.get().function_names_stack.len() >= s && !h.get().statements.is_empty() {
        h.next();
    }
}

fn _run_step_over(h: &mut HistoryMagic<Interpreter>) {
    let s = h.get().function_names_stack.len();
    h.next();
    while h.get().function_names_stack.len() > s && !h.get().statements.is_empty() {
        h.next();
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn interpreter_test_brutal(src: &str) {
    let core_ir = _compile(src, true);
    let mut interpreter = Interpreter::from_program(&core_ir);

    let mut history = Vec::new();
    let now = Instant::now();
    while let Some(_) = interpreter.step() {
        history.push(interpreter.clone());
    }
    let elapsed = now.elapsed().as_millis();
    let _: String = input("Check mem use now..");
    println!("brutal: {} ms ({})", elapsed, history.len());
}

#[cfg(not(target_arch = "wasm32"))]
pub fn interpreter_test_magic(src: &str) {
    let core_ir = _compile(src, true);
    let interpreter = Interpreter::from_program(&core_ir);

    let mut history = HistoryMagic::from_init(100, interpreter);

    let now = Instant::now();
    _run_until_return(&mut history);
    let elapsed = now.elapsed().as_millis();
    let _: String = input("Check mem use now..");
    println!("magic: {} ms ({})", elapsed, history.history.len());
}

#[cfg(not(target_arch = "wasm32"))]
pub fn interpreter_test_nosave(src: &str) {
    let core_ir = _compile(src, true);
    let mut interpreter = Interpreter::from_program(&core_ir);

    let now = Instant::now();
    while let Some(_) = interpreter.step() {}
    let elapsed = now.elapsed().as_millis();
    let _: String = input("Check mem use now..");
    println!("no save: {} ms ({})", elapsed, 0);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn interpreter_test_save1000(src: &str) {
    let core_ir = _compile(src, true);
    let mut interpreter = Interpreter::from_program(&core_ir);

    let n = 1000;
    let mut c = 0;
    let mut history = vec![Interpreter::new(); n];
    let now = Instant::now();
    while let Some(_) = interpreter.step() {
        history[c % n] = interpreter.clone();
        c += 1;
    }
    let elapsed = now.elapsed().as_millis();
    let _: String = input("Check mem use now..");
    println!("no save: {} ms ({})", elapsed, 0);
}
