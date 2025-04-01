#![feature(formatting_options)]

pub mod ast;
mod code;
mod error;
mod interpreter;
mod ir;
mod lexer;
mod simple_ast;
lalrpop_mod!(pub grammar);

use ast::{base::BaseSliceProgram, scoped::ScopedProgram, typed::TypedProgram};
use interpreter::Interpreter;
use ir::Prog;
use lalrpop_util::lalrpop_mod;
use simple_ast::{add_refcounts, from_scoped};

pub fn compile(code: &str) -> Prog {
    let base_program = BaseSliceProgram::new(&code).unwrap();
    let scoped_program = ScopedProgram::new(base_program).unwrap();
    let typed_program = TypedProgram::new(scoped_program).unwrap();

    let simple_program = from_scoped(&typed_program);
    let with_ref_count = add_refcounts(&simple_program);
    code::Compiler::new().compile(&with_ref_count)
}

pub fn c_code(program: &Prog) -> String {
    ir::output(program).join("\n")
}

pub fn run_interpreter(program: &Prog) {
    let mut interpreter = Interpreter::from_program(program);
    interpreter.run_until_done();
}
