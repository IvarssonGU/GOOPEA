extern crate console_error_panic_hook;
// use std::panic;

use wasm_bindgen::prelude::wasm_bindgen;
use language;

#[wasm_bindgen]
pub fn get_c_code(arg: &str) -> String {
    console_error_panic_hook::set_once();
    
    // let compiled = language::compile(arg);
    language::c_code(&language::compile(arg))
}

#[wasm_bindgen]
pub fn start_interpreter(arg: &str) {
    language::load_interpreter(&language::compile(arg));
}

#[wasm_bindgen]
pub fn get_one_step() -> String {
    language::step_interpreter();
    language::get_interpreter_state()
}

#[wasm_bindgen]
pub fn get_run() -> String {
    language::run_interpreter();
    language::get_interpreter_state()
}

#[wasm_bindgen]
pub fn get_state() -> String {
    language::get_interpreter_state()
}