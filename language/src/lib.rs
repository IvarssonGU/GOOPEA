#![feature(formatting_options)]

use lalrpop_util::lalrpop_mod;
use ast::{base::BaseSliceProgram, scoped::ScopedProgram, typed::TypedProgram};
use simple_ast::{add_refcounts, from_scoped};

mod code;
mod ir;
mod simple_ast;
mod lexer;
mod error;
pub mod ast;

mod interpreter;
lalrpop_mod!(pub grammar);

pub fn compile_and_run(code: &str) -> String {
    let base_program = BaseSliceProgram::new(&code).unwrap();
    let scoped_program = ScopedProgram::new(base_program).unwrap();
    let typed_program = TypedProgram::new(scoped_program).unwrap();

    let simple_program = from_scoped(&typed_program);
    let with_ref_count = add_refcounts(&simple_program);
    let code = code::Compiler::new().compile(&with_ref_count);

    ir::output(&code).join("\n")
}