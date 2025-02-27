use crate::ast::Operator;


pub type Program = Vec<Definition>;

#[derive(Debug, Clone)]
pub struct Definition {
    pub t: Type,
    pub id: String,
    pub args: Vec<(Type, String)>,
    pub statements: Vec<Statement>
}

#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Adt
}

#[derive(Debug, Clone)]
pub enum Statement {
    Decl(Type, String),
    Init(Type, String, Expression),
    Return(Expression),
    If(Expression),
    ElseIf(Expression),
    EndIf,
    Assign(Expression, Expression),
    AssignField(Expression, i32, Type, Expression),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Integer(i32),
    Ident(String),
    InitStruct(i32, i32),
    Deref(Type, Box<Expression>, i32),
    AccessTag(Box<Expression>),
    Application(String, Vec<Expression>),
    Operation(Box<Expression>, Operator, Box<Expression>)
}

//------------------------------------------------------------------------------------------
//                                   Pretty Print Below
//------------------------------------------------------------------------------------------
// Converts a cast::Program (a Vec<Definition>) into a Vec<String>
// where each string is a line of human-readable output.
pub fn output(program: Program) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push("#include <stdio.h>".to_string());
    lines.push("#include <stdlib.h>".to_string());
    lines.push(String::new());
    lines.push("typedef struct Adt Adt;".to_string());
    lines.push("struct Adt {".to_string());
    lines.push("\tint tag;".to_string());
    lines.push("\tvoid** data;".to_string());
    lines.push("};".to_string());
    lines.push(String::new());

    for def in program {
        // Convert the function signature.
        let args_str = def.args
            .iter()
            .map(|(arg_type, arg_id)| format!("{} {}", type_to_string(&arg_type), arg_id))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("{} {}({}) {{", type_to_string(&def.t), def.id, args_str));

        // Convert each statement with some indentation.
        for stmt in def.statements {
            lines.push(format!("    {}", statement_to_string(&stmt)));
        }
        lines.push("}".to_string());
    }
    lines
}

// Helper: Converts a cast::Type into a string.
fn type_to_string(t: &Type) -> String {
    match t {
        Type::Int => "int".to_string(),
        Type::Adt => "Adt".to_string(),
    }
}

// Helper: Converts a cast::Statement into a string.
fn statement_to_string(stmt: &Statement) -> String {
    match stmt {
        Statement::Decl(t, id) =>
            format!("{} {};", type_to_string(t), id),
        Statement::Init(t, id, expr) =>
            format!("{} {} = {};", type_to_string(t), id, expression_to_string(expr)),
        Statement::Return(expr) =>
            format!("return {};", expression_to_string(expr)),
        Statement::If(expr) =>
            format!("if ({}) {{", expression_to_string(expr)),
        Statement::ElseIf(expr) =>
            format!("else if ({}) {{", expression_to_string(expr)),
        Statement::EndIf =>
            "}".to_string(),
        Statement::Assign(lhs, rhs) =>
            format!("{} = {};", expression_to_string(lhs), expression_to_string(rhs)),
        Statement::AssignField(exp, index, t, assigned_exp) => format!("{0}.data[{1}] = malloc(sizeof({2}));\n*({2}*) {0}.data[1] = {3};", expression_to_string(exp), index, type_to_string(t), expression_to_string(assigned_exp)),
    }
}

// Helper: Converts a cast::Expression into a string.
fn expression_to_string(expr: &Expression) -> String {
    match expr {
        Expression::Integer(n) => format!("{}", n),
        Expression::Ident(s) => s.clone(),
        Expression::InitStruct(n1, n2) => format!("{{{}, {}}};", n1, if *n2 == 0 { "NULL".to_string() } else { format!("malloc({} * sizeof(void*))", n2)}),
        Expression::Deref(t, exp, index) =>
            format!("*(({}*) {}->data[{}])", type_to_string(t), expression_to_string(exp), index),
        Expression::AccessTag(e) =>
            format!("{}.tag", expression_to_string(e)),
        Expression::Application(id, args) => {
            let args_str = args.iter()
                .map(|arg| expression_to_string(arg))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({})", id, args_str)
        },
        Expression::Operation(lhs, op, rhs) => {
            format!("({} {} {})", expression_to_string(lhs), operator_to_string(op), expression_to_string(rhs))
        },
    }
}

// Helper: Converts an Operator into a string.
// (Adjust the match arms based on your Operator enum variants.)
fn operator_to_string(op: &Operator) -> String {
    match op {
        Operator::Add    => "+".to_string(),
        Operator::Sub    => "-".to_string(),
        Operator::Equal  => "==".to_string(),
        // Add other operators as needed…
        _ => format!("{:?}", op),
    }
}