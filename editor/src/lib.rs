extern crate console_error_panic_hook;
// use std::panic;

use wasm_bindgen::prelude::wasm_bindgen;
use language;

#[wasm_bindgen]
pub fn rust_function(arg: &str) -> String {
    console_error_panic_hook::set_once();

    language::compile_and_run(arg)
    // format!("HELLO, {}", arg.to_uppercase())
}