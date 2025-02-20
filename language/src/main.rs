use lalrpop_util::lalrpop_mod;

mod ast;

lalrpop_mod!(pub grammar);

mod lexer;

fn main() {
    let code = "enum Hej = A (Int, ((Int), A, ()))\nfip (Int, Int): ()\nTest vars = match a (x, y): x";

    let program = grammar::ProgramParser::new().parse(code).unwrap();
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
        grammar::ProgramParser::new().parse(&code).unwrap()
    }

    #[test]
    fn parse_reverse() {
        parse_example(Path::new("examples/reverse.goo"));
    }

    #[test]
    fn parse_zipper_tree() {
        parse_example(Path::new("examples/zipper_tree.goo"));
    }

    use crate::lexer::lexer;
    
    #[test]
    fn lexer_test() {
        let tokens = lexer("1()");
        println!("{:?}", tokens);
    }
}
