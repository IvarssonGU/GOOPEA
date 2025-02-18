use lalrpop_util::lalrpop_mod;

mod ast;

lalrpop_mod!(pub grammar);

fn main() {
    let program = grammar::ProgramParser::new().parse("enum Hej = A (Int, ((Int), A, ()))").unwrap();
    println!("{:#?}\n{}", program, program);
}
