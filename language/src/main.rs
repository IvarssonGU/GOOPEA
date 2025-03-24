#![feature(formatting_options)]

use std::{fs, path::Path};

use lalrpop_util::lalrpop_mod;
use ast::{ast::{ChainedData, Type}, base::BaseProgram, scoped::ScopedProgram, typed::TypedProgram};
use simple_ast::{add_refcounts, from_scoped};

mod code;
mod ir;
mod simple_ast;
mod lexer;
mod error;
pub mod ast;

lalrpop_mod!(pub grammar);

pub fn run(code: String) -> String {
    let base_program = BaseProgram::new(&code).unwrap();
    let scoped_program = ScopedProgram::new(base_program).unwrap();
    let typed_program = TypedProgram::new(scoped_program).unwrap();
    let simple_program = from_scoped(&typed_program);
    let with_ref_count = add_refcounts(&simple_program);
    let code = code::Compiler::new().compile(&with_ref_count);

    ir::output(&code).join("\n")
}

fn main() {
    let code = fs::read_to_string(Path::new("examples/arithmetic.goo")).unwrap();

    let base_program = BaseProgram::new(&code).unwrap();
    println!("{base_program}");

    let scoped_program = ScopedProgram::new(base_program).unwrap();
    println!("{scoped_program}");

    let typed_program = TypedProgram::new(scoped_program).unwrap();
    println!("{typed_program}");

    let simple_program = from_scoped(&typed_program);
    let with_ref_count = add_refcounts(&simple_program);
    let code = code::Compiler::new().compile(&with_ref_count);
    println!("{}", ir::output(&code).join("\n"));
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::fs;
    use crate::grammar;
    use crate::lexer::{Lexer, Token, lexer};

    fn parse_example(path: &Path) -> () {
        let code = fs::read_to_string(path).unwrap();
        grammar::ProgramParser::new().parse(Lexer::new(&code)).unwrap();
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
