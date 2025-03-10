#![feature(formatting_options)]

use std::{fs, path::Path};

use lalrpop_util::lalrpop_mod;
use lexer::Lexer;
use scoped_ast::ScopedProgram;

mod ast;
mod scoped_ast;
mod error;

lalrpop_mod!(pub grammar);

mod lexer;

fn main() {
    let code = fs::read_to_string(Path::new("examples/arithmetic.goo")).unwrap();

    let program = grammar::ProgramParser::new().parse(Lexer::new(&code)).unwrap();
    println!("{:#?}\n{}", program, program);

    let scoped_program = ScopedProgram::new(&program).unwrap();
    println!("{scoped_program}");

    scoped_program.validate().unwrap();
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
