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

    use crate::lexer;
    use crate::lexer::Token;
    
    fn lexer_test(file: &Path) -> Vec<lexer::Token> {
        let src = std::fs::read_to_string(file).unwrap();
        let tokens = lexer::lexer(src.as_str());

        tokens.iter().for_each(|token| println!("{:#?}", token));
        
        tokens
    }

    #[test]
    fn lexer_test_reverse() {
        assert_eq!(lexer_test(Path::new("examples/reverse.goo")).len(), 77)
    }

    #[test]
    fn lexer_test_zipper_tree() {
        assert_eq!(lexer_test(Path::new("examples/zipper_tree.goo")).len(), 177)
    }

    #[test]
    fn lexer_test_integer() {
        assert_eq!(
            lexer_test(Path::new("examples/integer.goo")), 
            vec![
                Token::Identifier("GetMinusFive".to_owned()), Token::Equal, Token::PlusMinus("-".to_owned()), Token::Integer(5), 
                Token::Identifier("Subtract".to_owned()), Token::Equal, Token::Integer(2), Token::PlusMinus("-".to_owned()), Token::Integer(1)
            ])
    }
}
