#![feature(formatting_options)]
#![feature(btree_cursors)]
#![feature(mixed_integer_ops_unsigned_sub)]

use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[cfg(not(target_arch = "wasm32"))]
use clap::Parser;

use ast::base::BaseSliceProgram;
use ast::{scoped::ScopedProgram, typed::TypedProgram};
use compiler::compile::compile_typed;
use error::Result;
use lalrpop_util::lalrpop_mod;
use preprocessor::preprocess;

pub mod ast;
pub mod compiler;
mod error;
mod interpreter;
mod lexer;
pub mod preprocessor;

lalrpop_mod!(pub grammar);

#[cfg(target_arch = "wasm32")]
fn main() {}

fn parse_and_validate(code: &str) -> Result<TypedProgram<'_>> {
    let base_program = BaseSliceProgram::new(&code)?;
    let scoped_program = ScopedProgram::new(base_program)?;
    TypedProgram::new(scoped_program)
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    file: PathBuf,
    #[arg(short, long)]
    interpret: bool,
    #[arg(short, long)]
    preprocess: bool,
    #[arg(short, long)]
    benchmark: bool,
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // "--"" för att separera cargos argument med våra argument
    // cargo run -- -f examples/test.goo

    // -i / --interpret för interpreter
    // cargo run (--release) -- -f examples/test.goo -i

    let args = Args::parse();
    let file = args.file;
    match (args.interpret, args.preprocess) {
        (false, false) => {
            let code = preprocess(file);
            let typed_program = parse_and_validate(&code)
                .map_err(|e| e.to_string())
                .unwrap();
            let compiled_program = compile_typed(&typed_program);
            let result = compiler::core::output(&compiled_program.core);
            println!("{}", result.join("\n"));
        }
        (false, true) => {
            println!("{}", preprocess(file));
        }
        (true, false) => {
            if args.benchmark {
                if file.is_dir() {
                    println!("file, fip, malloc_time_micros, exec_time_ms, steps, steps/s, max_mem_words");
                    interpreter::interpreter_bench_fip(&file, Duration::from_micros(0));
                    interpreter::interpreter_bench_fip(&file, Duration::from_micros(5));
                    interpreter::interpreter_bench_fip(&file, Duration::from_micros(25));
                    interpreter::interpreter_bench_fip(&file, Duration::from_micros(50));
                } else {
                    interpreter::interpreter_bench(&file);
                    interpreter::interpreter_bench_peak_mem(file);
                }
            } else {
                interpreter::interpreter_test(file);
            }
        }
        (true, true) => panic!("cant interpret and preprocess"),
    }
}
