use super::interpreter::Data;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::{to_string, to_string_pretty};

#[derive(Debug, Serialize, Deserialize)]
pub struct MemPeek {
    data: Vec<MemObj>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MemObj {
    Value(i64),
    Pointer(Box<MemPeek>),
}

impl MemObj {
    pub fn from_data(x: &Data, mem: &Vec<Vec<Data>>) -> Self {
        match x {
            Data::Value(i) => MemObj::Value(*i),
            Data::Pointer(n) => MemObj::Pointer(Box::new({
                let slice = mem[*n].clone();
                MemPeek {
                    data: slice
                        .into_iter()
                        .map(|d| MemObj::from_data(&d, mem))
                        .collect_vec(),
                }
            })),
        }
    }

    pub fn as_json(&self) -> String {
        to_string_pretty(self).unwrap()
    }
}
