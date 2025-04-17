#![feature(formatting_options)]
#![feature(btree_cursors)]
#![feature(mixed_integer_ops_unsigned_sub)]

pub mod ast;
pub mod core;
pub mod error;
pub mod interpreter;
mod lexer;
mod score;
pub mod stir;

lalrpop_mod!(pub grammar);

use error::Result;
use ast::{base::BaseSliceProgram, scoped::ScopedProgram, typed::TypedProgram};
use core::Prog;
use interpreter::Interpreter;
use lalrpop_util::lalrpop_mod;
use std::cell::RefCell;

thread_local! {
    static INTERPRETER: RefCell<Interpreter> = RefCell::new(Interpreter::new());
    static INT_HISTORY: RefCell<Vec<Interpreter>> = RefCell::new(Vec::new());
}

pub fn compile(code: &str) -> Result<Prog> {
    let base_program = BaseSliceProgram::new(&code)?;
    let scoped_program = ScopedProgram::new(base_program)?;
    let typed_program = TypedProgram::new(scoped_program)?;

    let pure_ir = stir::from_typed(&typed_program);
    let pure_reuse = stir::add_reuse(&pure_ir);
    let pure_rc = stir::add_rc(&pure_reuse, true);
    let core = score::translate(&pure_rc);
    Ok(core)
}

pub fn c_code(program: &Prog) -> String {
    core::output(program).join("\n")
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

pub fn run_until_next_ptr() {
    INTERPRETER.with_borrow_mut(|interpreter| {
        interpreter.run_until_next_ptr();
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

pub fn perform_on_interpreter<T>(f: impl FnOnce(&Interpreter) -> T) -> T {
    INTERPRETER.with_borrow(f)
}