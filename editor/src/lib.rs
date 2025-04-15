pub mod visualization;

extern crate console_error_panic_hook;
// use std::panic;

use wasm_bindgen::prelude::wasm_bindgen;
use language::{self, core::Prog, error::{Error, Result}};
use std::{cell::RefCell, fmt::Debug};

#[wasm_bindgen]
pub struct ResultWrapper {
    ok: Option<Prog>,
    err: Option<Error>
}

#[wasm_bindgen]
pub struct ErrorLocation {
    pub start_line: usize,
    pub start_line_char: usize,
    pub end_line: usize,
    pub end_line_char: usize
}

#[wasm_bindgen]
impl ResultWrapper {
    pub fn is_ok(&self) -> bool { self.ok.is_some() }
    pub fn is_err(&self) -> bool { self.err.is_some() }

    pub fn unwrap(self) -> ProgWrapper { ProgWrapper(self.ok.unwrap()) }
    pub fn unwrap_err(self) -> ErrorWrapper { ErrorWrapper(self.err.unwrap()) }
}

impl Into<ResultWrapper> for Result<Prog> {
    fn into(self) -> ResultWrapper {
        match self {
            Ok(ok) => ResultWrapper { ok: Some(ok), err: None },
            Err(err) => ResultWrapper { ok: None, err: Some(err) },
        }
    }
}

#[wasm_bindgen]
pub struct ErrorWrapper(Error);

#[wasm_bindgen]
pub struct ProgWrapper(Prog);

#[wasm_bindgen]
impl ErrorWrapper {
    pub fn get_error_string(&self) -> String {
        format!("{}", self.0)
    }

    pub fn has_source(&self) -> bool { self.0.source.is_some() }
    pub fn get_source(&self) -> ErrorLocation {
        let source = self.0.source.as_ref().unwrap();

        ErrorLocation { 
            start_line: source.start.line, 
            start_line_char: source.start.char_offset, 
            end_line: source.end.line, 
            end_line_char: source.end.char_offset 
        }    
    }
}

#[wasm_bindgen]
impl ProgWrapper {
    pub fn get_c_code(&self) -> String {
        language::c_code(&self.0)
    }

    pub fn start_interpreter(&self) {
        language::load_interpreter(&self.0)
    }
}

#[wasm_bindgen]
pub fn compile(arg: &str) -> ResultWrapper {
    console_error_panic_hook::set_once();

    language::compile(arg).into()
}

#[wasm_bindgen]
pub fn start_interpreter(arg: &str) {
    language::load_interpreter(&language::compile(arg).unwrap())
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