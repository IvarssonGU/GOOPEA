use wasm_bindgen::prelude::wasm_bindgen;
use language;

#[wasm_bindgen]
pub fn rust_function(arg: &str) -> String {
    println!("hello");
    language::run(arg.to_string())
    // format!("HELLO, {}", arg.to_uppercase())
}