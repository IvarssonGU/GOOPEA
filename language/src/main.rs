mod ast;
mod code;

fn main() {
    let x = ast::FunctionDefinition {
        id: ast::FID(String::from("testing")),
        body: ast::Expression::Integer(20),
        signature: ast::FunctionSignature {
            argument_type: ast::TupleType(vec![ast::Type::Int, ast::Type::Int, ast::Type::Int]),
            result_type: ast::Type::Int,
            is_fip: false
        }
    };
    println!("{}", code::compile_fun(x));
}
