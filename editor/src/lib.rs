use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn rust_function(arg: &str) -> String {
    format!("HELLO, {}", arg.to_uppercase())
}