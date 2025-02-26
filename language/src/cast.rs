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
}

#[derive(Debug, Clone)]
pub enum Expression {
    Integer(i32),
    Ident(String),
    MallocAdt,
    MallocInt,
    Malloc(u32),
    DerefInt(Box<Expression>, u32),
    AccessData(Box<Expression>, u32),
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
    for def in program {
        // Convert the function signature.
        let args_str = def.args
            .iter()
            .map(|(arg_type, arg_id)| format!("{} {}", type_to_string(arg_type.clone()), arg_id))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("{} {}({}) {{", type_to_string(def.t), def.id, args_str));

        // Convert each statement with some indentation.
        for stmt in def.statements {
            lines.push(format!("    {}", statement_to_string(&stmt)));
        }
        lines.push("}".to_string());
    }
    lines
}

// Helper: Converts a cast::Type into a string.
fn type_to_string(t: Type) -> String {
    match t {
        Type::Int => "int".to_string(),
        Type::Adt => "adt".to_string(),
    }
}

// Helper: Converts a cast::Statement into a string.
fn statement_to_string(stmt: &Statement) -> String {
    match stmt {
        Statement::Decl(t, id) =>
            format!("{} {};", type_to_string(t.clone()), id),
        Statement::Init(t, id, expr) =>
            format!("{} {} = {};", type_to_string(t.clone()), id, expression_to_string(expr)),
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
    }
}

// Helper: Converts a cast::Expression into a string.
fn expression_to_string(expr: &Expression) -> String {
    match expr {
        Expression::Integer(n) => format!("{}", n),
        Expression::Ident(s) => s.clone(),
        Expression::MallocAdt => "mallocAdt()".to_string(),
        Expression::MallocInt => "mallocInt()".to_string(),
        Expression::Malloc(n) => format!("malloc({})", n),
        Expression::DerefInt(e, offset) =>
            format!("*({} + {})", expression_to_string(e), offset),
        Expression::AccessData(e, idx) =>
            format!("{}->data[{}]", expression_to_string(e), idx),
        Expression::AccessTag(e) =>
            format!("{}->tag", expression_to_string(e)),
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