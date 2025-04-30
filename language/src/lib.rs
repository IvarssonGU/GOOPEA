#![feature(formatting_options)]
#![feature(btree_cursors)]
#![feature(mixed_integer_ops_unsigned_sub)]

pub mod ast;
pub mod compiler;
pub mod error;
mod interpreter;
mod lexer;
mod preprocessor;

lalrpop_mod!(pub grammar);

use ast::{base::BaseSliceProgram, scoped::ScopedProgram, typed::TypedProgram};
use compiler::{
    compile::{CompiledProgram, compile_typed},
    core::Prog,
};
use error::Result;
use interpreter::Interpreter;
use lalrpop_util::lalrpop_mod;
use std::cell::RefCell;

thread_local! {
    static INTERPRETER: RefCell<Interpreter> = RefCell::new(Interpreter::new());
    static INT_HISTORY: RefCell<Vec<Interpreter>> = RefCell::new(Vec::new());
}

pub fn compile(code: &str) -> Result<CompiledProgram> {
    let base_program = BaseSliceProgram::new(&code)?;
    let scoped_program = ScopedProgram::new(base_program)?;
    let typed_program = TypedProgram::new(scoped_program)?;
    let compiled_program = compile_typed(&typed_program);
    Ok(compiled_program)
}

pub fn c_code(program: &CompiledProgram) -> String {
    compiler::core::output(&program.core).join("\n")
}

pub fn stir_str(program: &CompiledProgram) -> String {
    program
        .stir
        .iter()
        .map(|def| def.to_string())
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn reuse_str(program: &CompiledProgram) -> String {
    program
        .reuse
        .iter()
        .map(|def| def.to_string())
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn rc_str(program: &CompiledProgram) -> String {
    program
        .rc
        .iter()
        .map(|def| def.to_string())
        .collect::<Vec<String>>()
        .join("\n")
}

// Interpreter stuff
//         store  store  store store
// restore return memory step1 finish
// state   state  state  state state

pub fn load_interpreter(program: &Prog) {
    let interpreter = Interpreter::from_program(program);
    INTERPRETER.set(interpreter);
}

pub fn step_interpreter() {
    INTERPRETER.with_borrow_mut(|interpreter| {
        interpreter.step();
    });
}

pub fn run_until_next_mem() {
    INTERPRETER.with_borrow_mut(|interpreter| {
        interpreter.run_until_next_mem();
    });
}

pub fn run_until_return() {
    INTERPRETER.with_borrow_mut(|interpreter| {
        interpreter.run_until_return();
    });
}

pub fn run_interpreter() {
    INTERPRETER.with_borrow_mut(|interpreter| {
        interpreter.run_until_done();
    });
}

pub fn get_interpreter_state() -> String {
    INTERPRETER.with_borrow(|interpreter| format!("{:?}", interpreter))
}

pub fn store_interpreter() {
    INT_HISTORY.with_borrow_mut(|history| {
        history.push(INTERPRETER.with(|x| x.clone()).borrow().clone());
    });
}

pub fn restore_interpreter() {
    INT_HISTORY.with_borrow_mut(|history| {
        if let Some(i) = history.pop() {
            INTERPRETER.set(i);
        }
    });
}
