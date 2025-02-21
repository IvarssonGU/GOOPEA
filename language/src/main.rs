use lalrpop_util::lalrpop_mod;

mod ast;

lalrpop_mod!(pub grammar);

mod lexer;

fn main() {
    let code = "enum Hej = A (Int, ((Int), A, ()))\n(Int, Int): ()\ntest (a, b) =";

    let program = grammar::ProgramParser::new().parse(code).unwrap();
    println!("{:#?}\n{}", program, program);
}

#[cfg(test)]
mod tests {
    use crate::lexer::lexer;
    
    #[test]
    fn lexer_test() {
        let src = std::fs::read_to_string("src/testsyntax.txt").unwrap();
        let tokens = lexer(src.as_str());
        tokens.iter().for_each(|token| println!("{:#?}", token));
        
        assert_eq!(tokens.len(), 107);
    }
}
