#![feature(formatting_options)]

use std::{fs, path::Path};

use lalrpop_util::lalrpop_mod;
use lexer::Lexer;
use scoped_ast::ScopedProgram;

mod code;
mod ir;
mod simple_ast;
mod scoped_ast;
mod ast;
mod lexer;
mod error;
lalrpop_mod!(pub grammar);
use simple_ast::*;
fn main() {
    let code = fs::read_to_string(Path::new("examples/tree_flip.goo")).unwrap();

    let program = grammar::ProgramParser::new().parse(Lexer::new(&code)).unwrap();

    let scoped_program = ScopedProgram::new(&program).unwrap();
    scoped_program.validate().unwrap();
    let simple_program = from_scoped(&scoped_program);
    let with_ref_count = add_refcounts(&simple_program);
    let code = code::Compiler::new().compile(&with_ref_count);
    println!("{}", ir::output(&code, true).join("\n"));
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::fs;
    use crate::grammar;
    use crate::lexer::{Lexer, Token, lexer};

    fn parse_example(path: &Path) -> () {
        let code = fs::read_to_string(path).unwrap();
        println!("{}", grammar::ProgramParser::new().parse(Lexer::new(&code)).unwrap());
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
