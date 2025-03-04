mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    // #[wasm_bindgen(raw_module = "/../www/index.js")]
    // fn to_output_box(s : &str);    
}

#[wasm_bindgen(module = "/src/jstest.js")]
extern "C" {
    fn testprint();
}

#[wasm_bindgen(raw_module = "../www/index.js")]
extern "C" {
    fn to_output_box(s : &str);
}


#[wasm_bindgen]
pub fn greet() {
    alert("Hello, editor!");
}
#[wasm_bindgen]
pub fn process_text(text : &str){
    // log(text)
    to_output_box(text);
    testprint()
}