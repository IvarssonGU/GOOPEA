use lalrpop_util::lalrpop_mod;

mod ast;

lalrpop_mod!(pub grammar);

fn main() {
    let code = "enum Hej = A (Int, ((Int), A, ()))\n(Int, Int): ()\ntest (a, b) =";

    let program = grammar::ProgramParser::new().parse(code).unwrap();
    println!("{:#?}\n{}", program, program);
}
