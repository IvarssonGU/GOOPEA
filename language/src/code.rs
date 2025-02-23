use crate::ast::*;

pub fn compile_fun (fun : FunctionDefinition) -> String {
    let mut emit = String::new();
    emit.push_str(from_type(&fun.signature.result_type));
    emit.push(' ');
    emit.push_str(fun.id.0.as_str());
    emit.push('(');
    let vec = fun.signature.argument_type.0;
    for i in 0..vec.len() {
        emit.push_str(from_type(&vec[i]));
        emit.push(' ');
        emit.push_str(format!("v{}", i).as_str());
        if (i != vec.len() - 1) {
            emit.push_str(", ");
        }
    }
    emit.push_str(") {\n\treturn ");
    emit.push_str(compile_exp(fun.body).to_string().as_str());
    emit.push_str(";\n}");

    return emit;
}

fn from_type (t : &Type) -> &str {
    match t {
        Type::ADT(_t) => "void*",
        Type::Tuple(_t) => "void*",
        Type::Int => "int"
    }
}

pub fn compile_exp (exp : Expression) -> i32 {
    match exp {
        Expression::Integer(n) => n,
        _ => 0,
    }
}
