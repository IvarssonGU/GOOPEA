#![feature(formatting_options)]
#![feature(btree_cursors)]
#![feature(mixed_integer_ops_unsigned_sub)]

#[cfg(not(target_arch = "wasm32"))]
use std::fs;

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

mod ast;
mod core;
mod error;
mod interpreter;
mod lexer;
mod preprocessor;
mod score;
mod stir;

use ast::base::BaseSliceProgram;
use ast::{scoped::ScopedProgram, typed::TypedProgram};
use error::Result;
use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub grammar);

#[cfg(target_arch = "wasm32")]
fn main() {}

fn parse_and_validate(code: &str) -> Result<TypedProgram<'_>> {
    let base_program = BaseSliceProgram::new(&code)?;
    let scoped_program = ScopedProgram::new(base_program)?;
    TypedProgram::new(scoped_program)
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let code = fs::read_to_string(Path::new("examples/tree_flip.goo")).unwrap();
    //let code = "(): ()\nmain = ()".to_string();

    let typed_program = parse_and_validate(&code)
        .map_err(|e| e.to_string())
        .unwrap();

    let pure_ir = stir::from_typed(&typed_program);
    let pure_reuse = stir::add_reuse(&pure_ir);
    let pure_rc = stir::add_rc(&pure_reuse, true);
    let core_ir = score::translate(&pure_rc);
    let result = core::output(&core_ir);
    println!("{}", result.join("\n"));
}
