#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests_parse_lex {
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

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests_interpreter {
    use crate::interpreter;
    #[test]
    fn test1() {
        interpreter::Interpreter::new();
    }
}
