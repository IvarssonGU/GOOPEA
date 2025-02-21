#![feature(formatting_options)]

use lalrpop_util::lalrpop_mod;
use lexer::Lexer;

mod ast;

lalrpop_mod!(pub grammar);

mod lexer;

fn main() {
    let code = "enum Hej = A (Int, ((Int), A, ()));\nfip (Int, Int): ()\nTest vars = match a (x, y): (RunLong 1);";

    let program = grammar::ProgramParser::new().parse(Lexer::new(code)).unwrap();
    println!("{:#?}\n{}", program, program);
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::fs;
    use crate::ast::Program;
    use crate::grammar;

    fn parse_example(path: &Path) -> Program {
        let code = fs::read_to_string(path).unwrap();
        grammar::ProgramParser::new().parse(Lexer::new(&code)).unwrap()
    }

    #[test]
    fn parse_reverse() {
        parse_example(Path::new("examples/reverse.goo"));
    }

    #[test]
    fn parse_zipper_tree() {
        parse_example(Path::new("examples/zipper_tree.goo"));
    }

    use crate::lexer::{lexer, Lexer};
    
    fn lexer_test(file: &Path) -> usize {
        let src = std::fs::read_to_string(file).unwrap();
        let tokens = lexer(src.as_str());

        tokens.iter().for_each(|token| println!("{:#?}", token));
        
        tokens.len()
    }

    #[test]
    fn lexer_test_reverse() {
        assert_eq!(lexer_test(Path::new("examples/reverse.goo")), 81)
    }

    #[test]
    fn lexer_test_zipper_tree() {
        assert_eq!(lexer_test(Path::new("examples/zipper_tree.goo")), 185)
    }
}
