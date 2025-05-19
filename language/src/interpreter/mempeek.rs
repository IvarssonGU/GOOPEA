use super::interpreter::Data;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

#[derive(Debug, Serialize, Deserialize)]
pub struct MemPeek {
    data: Vec<MemObj>,
}

impl MemPeek {
    fn is_list(&self) -> bool {
        match &self.data[..] {
            [_, _, _, x, p] => x.is_val() && match p {
                MemObj::Value(0) => true,
                MemObj::Value(_) => false,
                MemObj::Pointer(p) => p.is_list()
            },
            _ => false
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MemObj {
    Value(i64),
    Pointer(Box<MemPeek>),
}

impl MemObj {
    fn is_val(&self) -> bool {
        match self {
            MemObj::Value(_) => true,
            MemObj::Pointer(_) => false,
        }
    }

    fn unwrap_val(&self) -> i64 {
        match self {
            MemObj::Value(x) => *x,
            MemObj::Pointer(_) => panic!(),
        }
    }
}

impl MemObj {
    // Infinite loop on cycles
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

    pub fn is_list(&self) -> bool {
        match self {
            MemObj::Value(_) => false,
            MemObj::Pointer(mem_peek) => mem_peek.is_list()
        }
    }

    fn list(&self) -> Vec<i64> {
        let mut vec = Vec::new();
        let mut data = match self {
            MemObj::Value(_) => panic!(),
            MemObj::Pointer(mem_peek) => mem_peek,
        };
        loop {
            vec.push(data.data[3].unwrap_val()); 
            match &data.data[4] {
                MemObj::Value(_) => {break;},
                MemObj::Pointer(mem_peek) => {data = mem_peek},
            } 
        }
        vec
    } 

    pub fn list_string(&self) -> String {
        format!("[{}]", self.list().iter().map(|x| x.to_string()).join(", "))
    }
}
