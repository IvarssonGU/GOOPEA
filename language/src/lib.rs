#![feature(formatting_options)]

pub mod ast;
mod code;
mod error;
mod interpreter;
mod ir;
mod lexer;
mod simple_ast;
lalrpop_mod!(pub grammar);

use ast::{base::BaseSliceProgram, scoped::ScopedProgram, typed::TypedProgram};
use interpreter::Interpreter;
use ir::Prog;
use lalrpop_util::lalrpop_mod;
use simple_ast::{add_refcounts, from_scoped};
use std::cell::RefCell;

thread_local! {
    static INTERPRETER: RefCell<Interpreter> = RefCell::new(Interpreter::new());
    static INT_HISTORY: RefCell<Vec<Interpreter>> = RefCell::new(Vec::new());
}

pub fn compile(code: &str) -> Prog {
    let base_program = BaseSliceProgram::new(&code).unwrap();
    let scoped_program = ScopedProgram::new(base_program).unwrap();
    let typed_program = TypedProgram::new(scoped_program).unwrap();

    let simple_program = from_scoped(&typed_program);
    let with_ref_count = add_refcounts(&simple_program);
    code::Compiler::new().compile(&with_ref_count)
}

pub fn c_code(program: &Prog) -> String {
    ir::output(program).join("\n")
}

// Interpreter stuff

pub fn load_interpreter(program: &Prog) {
    let interpreter = Interpreter::from_program(program);
    INTERPRETER.set(interpreter);
}

pub fn step_interpreter() {
    INTERPRETER.with_borrow_mut(|interpreter| {
        interpreter.step();
    });
}

pub fn run_interpreter() {
    INTERPRETER.with_borrow_mut(|interpreter| {
        interpreter.run_until_done();
    });
}

pub fn get_interpreter_state() -> String {
    INTERPRETER.with_borrow_mut(|interpreter| format!("{:?}", interpreter))
}

pub fn save_interpreter() {
    INT_HISTORY.with_borrow_mut(|history| {
        history.push(INTERPRETER.with(|x| x.clone()).borrow().clone());
    });
}

pub fn restore_interpreter() {
    INT_HISTORY.with_borrow_mut(|history| {
        INTERPRETER.set(history.pop().unwrap()); // Skulle kunna felhantera här
    });
}
