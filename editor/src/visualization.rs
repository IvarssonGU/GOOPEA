use std::collections::BTreeMap;

use language::{interpreter::Interpreter, perform_on_interpreter};
use serde::Serialize;
use serde_wasm_bindgen::preserve::serialize;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use language::interpreter::Data as InterpreterData;

#[derive(Serialize)]
pub struct MemorySnapshot {
    pub variables: BTreeMap<String, Data>,
    pub heap: Vec<Vec<Data>>,
    pub call_stack: Vec<String>
}

pub struct BigInt(i64);

impl Serialize for BigInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        serializer.collect_str(&self.0)
    }
}

#[derive(Serialize)]
pub struct Data {
    pub is_ptr: bool,
    pub val: BigInt
}

impl Data {
    pub fn new_ptr(x: i64) -> Data { Data { is_ptr: true, val: BigInt(x) } }
    pub fn new_int(x: i64) -> Data { Data { is_ptr: false, val: BigInt(x) } }
}

impl From<InterpreterData> for Data {
    fn from(value: InterpreterData) -> Self {
        match value {
            InterpreterData::Pointer(x) => Data::new_ptr(x as i64),
            InterpreterData::Value(x) => Data::new_int(x)
        }
    }
}

pub fn take_interpreter_memory_snapshot_helper(interp: &Interpreter) -> MemorySnapshot {
    let mut variables = BTreeMap::new();

    for (name, data) in interp.get_variables_raw() {
        variables.insert(name, data.into());
    }

    MemorySnapshot { 
        variables, 
        heap: interp.get_memory_raw().into_iter().map(|x| x.into_iter().map(|x| x.into()).collect()).collect(),
        call_stack: interp.get_function_names_stack()
    }
}

#[wasm_bindgen]
pub fn take_interpreter_memory_snapshot() -> JsValue {
    let snapshot = perform_on_interpreter(take_interpreter_memory_snapshot_helper);

    serde_wasm_bindgen::to_value(&snapshot).unwrap()
}