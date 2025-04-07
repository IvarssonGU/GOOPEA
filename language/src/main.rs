#![feature(formatting_options)]

use std::{collections::HashMap, fs, path::Path};

use lalrpop_util::lalrpop_mod;
use lexer::Lexer;

mod ast;
mod core;
mod error;
mod lexer;
mod scoped;
mod score;
mod simple_ast;
mod stir;

lalrpop_mod!(pub grammar);
use stir::*;
fn main() {
    let code = fs::read_to_string(Path::new("examples/reuse_different_type.goo")).unwrap();
    let program = grammar::ProgramParser::new()
        .parse(Lexer::new(&code))
        .unwrap();
    let scoped_program = scoped::ScopedProgram::new(&program).unwrap();
    scoped_program.validate().unwrap();
    let pure_ir = from_scoped(&scoped_program);
    for fun in &pure_ir {
        println!("{}", fun);
    }
    let pure_reuse = add_reuse(&pure_ir);
    let pure_rc = add_rc(&pure_ir, false);
    let core = score::translate(&pure_rc);
    let result = core::output(&core);
    for line in result {
        println!("{}", line);
    }
}

#[cfg(test)]
mod tests {
    use crate::grammar;
    use crate::lexer::{Lexer, Token, lexer};
    use std::fs;
    use std::path::Path;

    fn parse_example(path: &Path) -> () {
        let code = fs::read_to_string(path).unwrap();
        println!(
            "{}",
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
