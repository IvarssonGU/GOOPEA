#![feature(formatting_options)]
#![feature(btree_cursors)]
#![feature(mixed_integer_ops_unsigned_sub)]

#[cfg(not(target_arch = "wasm32"))]
use std::fs;

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

use ast::base::BaseSliceProgram;
use ast::{scoped::ScopedProgram, typed::TypedProgram};
use error::Result;
use lalrpop_util::lalrpop_mod;

pub mod ast;
mod error;
mod lexer;

mod core;
mod score;
mod stir;

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
    use core::Def;

    let code = fs::read_to_string(Path::new("examples/bools.goo")).unwrap();
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

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use crate::grammar;
    use crate::lexer::{Lexer, Token, lexer};
    use std::fs;
    use std::path::Path;

    fn parse_example(path: &Path) -> () {
        let code = fs::read_to_string(path).unwrap();
        println!(
            "{:?}",
            grammar::ProgramParser::new()
                .parse(Lexer::new(&code))
                .unwrap()
        );
    }

    #[test]
    fn parse_reverse() {
        parse_example(Path::new("examples/reverse.goo"));
    }

    #[test]
    fn parse_zipper_tree() {
        parse_example(Path::new("examples/zipper_tree.goo"));
    }

    #[test]
    fn parse_integer() {
        parse_example(Path::new("examples/integer.goo"));
    }

    fn lexer_test(file: &Path) -> Vec<Token> {
        let src = std::fs::read_to_string(file).unwrap();
        let tokens = lexer(src.as_str());

        tokens.iter().for_each(|token| println!("{:#?}", token));

        tokens
    }

    #[test]
    fn lexer_test_reverse() {
        assert_eq!(lexer_test(Path::new("examples/reverse.goo")).len(), 68)
    }

    #[test]
    fn lexer_test_zipper_tree() {
        assert_eq!(lexer_test(Path::new("examples/zipper_tree.goo")).len(), 160)
    }
}
