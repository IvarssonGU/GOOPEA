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
    use crate::lexer::lexer;
    
    #[test]
    fn lexer_test() {
        let tokens = lexer("1()");
        println!("{:?}", tokens);
    }
}
