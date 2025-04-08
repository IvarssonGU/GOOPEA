extern crate console_error_panic_hook;
// use std::panic;

use wasm_bindgen::prelude::wasm_bindgen;
use language::{self, ir::Prog};
use color_eyre::Result;
use std::cell::RefCell;

thread_local! {
    static COMPILATION_RESULT: RefCell<Option<Result<Prog>>> = RefCell::new(None);
}

#[wasm_bindgen]
pub fn compile(arg: &str) {
    console_error_panic_hook::set_once();
    COMPILATION_RESULT.set(Some(language::compile(arg)));
}

#[wasm_bindgen]
pub fn get_c_code() -> String {
    COMPILATION_RESULT.with_borrow(|x| {
        language::c_code(x.as_ref().unwrap().as_ref().unwrap())
    })
}

#[wasm_bindgen]
pub fn get_error() -> String {
    COMPILATION_RESULT.with_borrow(|x| {
        x.as_ref().unwrap().as_ref().map_err(|e| e.to_string()).err().unwrap()
    })
}

#[wasm_bindgen]
pub fn has_error() -> bool {
    COMPILATION_RESULT.with_borrow(|x| {
        x.as_ref().unwrap().is_err()
    })
}

#[wasm_bindgen]
pub fn start_interpreter() {
    COMPILATION_RESULT.with_borrow(|x| {
        language::load_interpreter(x.as_ref().unwrap().as_ref().unwrap());
    })
}

#[wasm_bindgen]
pub fn get_one_step() -> String {
    language::store_interpreter();
    language::step_interpreter();
    language::get_interpreter_state()
}

#[wasm_bindgen]
pub fn get_run() -> String {
    language::store_interpreter();
    language::run_interpreter();
    language::get_interpreter_state()
}

#[wasm_bindgen]
pub fn get_back_step() -> String {
    language::restore_interpreter();
    language::get_interpreter_state()
}

#[wasm_bindgen]
pub fn get_until_mem() -> String {
    language::store_interpreter();
    language::run_until_next_mem();
    language::get_interpreter_state()
}

#[wasm_bindgen]
pub fn get_until_return() -> String {
    language::store_interpreter();
    language::run_until_return();
    language::get_interpreter_state()
}

#[wasm_bindgen]
pub fn get_state() -> String {
    language::get_interpreter_state()
}