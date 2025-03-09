use crate::simple_ast::Operator;

pub struct Prog(pub Vec<Def>, pub Vec<u8>);

#[derive(Debug, Clone)]
pub struct Def {
    pub type_len: Option<u8>,
    pub id: String,
    pub args: Vec<String>,
    pub body: Vec<Statement>,
}

pub type Type = Option<u8>;

#[derive(Debug, Clone)]
pub enum Operand {
    Identifier(String),
    BinOp(Operator, Box<Operand>, Box<Operand>),
    Integer(i64),
    Application(String, Vec<Operand>),
    DerefField(String, i64),
    AccessField(String, i64),
    Condition(bool, String, Box<Operand>, Box<Operand>),
    UTuple(Vec<Operand>),
}

#[derive(Debug, Clone)]
pub enum Statement {
    Decl(String),
    If(Operand),
    ElseIf(Operand),
    EndIf,
    InitConstructor(String, i64),
    AssignField(String, i64, Operand),
    Assign(Option<Type>, String, Operand),
    Return(Operand),
    Print(Operand),
}

pub fn output(prog: &Prog) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push("#include <stdio.h>".to_string());
    lines.push("#include <stdlib.h>".to_string());
    lines.push(String::new());
    lines.push("typedef __int64_t Value;".to_string());
    lines.push(String::new());

    for amount in &prog.1 {
        lines.push(format!("typedef struct Value{0} Value{0};", amount));
        lines.push(format!("struct Value{} {{;", amount));
        for i in 0..*amount {
            lines.push(format!("\tValue elem{};", i));
        }
        lines.push(format!("}};"));
    }
    lines.push(String::new());
    for def in &prog.0 {
        lines.push(output_function_decls(def));
    }
    lines.push(String::new());
    for def in &prog.0 {
        let args_str = def
            .args
            .iter()
            .map(|arg| format!("Value {}", arg))
            .collect::<Vec<_>>()
            .join(", ");
        let utuple = match def.type_len {
            Some(n) => n.to_string(),
            None => String::new(),
        };
        lines.push(format!("Value{} {}({}) {{", utuple, def.id.clone(), args_str));
        let mut depth = 1;
        for stmt in &def.body {
            match stmt {
                Statement::If(_) => {
                    lines.push(format!(
                        "{}{}",
                        "\t".repeat(depth),
                        statement_to_string(&stmt)
                    ));
                    depth += 1;
                }
                Statement::ElseIf(_) => {
                    lines.push(format!(
                        "{}{}",
                        "\t".repeat(depth),
                        statement_to_string(&stmt)
                    ));
                    depth += 1;
                }
                Statement::EndIf => {
                    depth -= 1;
                    lines.push(format!(
                        "{}{}",
                        "\t".repeat(depth),
                        statement_to_string(&stmt)
                    ));
                }
                _ => {
                    lines.push(format!(
                        "{}{}",
                        "\t".repeat(depth),
                        statement_to_string(&stmt)
                    ));
                }
            }
        }
        lines.push("}".to_string());
        lines.push(String::new());
    }
    lines
}

fn output_function_decls(def: &Def) -> String {
    let utuple = match def.type_len {
        Some(n) => n.to_string(),
        None => String::new(),
    };
    let args_str = def
        .args
        .iter()
        .map(|arg| format!("Value {}", arg))
        .collect::<Vec<_>>()
        .join(", ");
    format!("Value{} {}({});", utuple, def.id.clone(), args_str)
}

fn statement_to_string(stmt: &Statement) -> String {
    match stmt {
        Statement::Assign(has_type, id, op) => format!(
            "{}{} = {};",
            match has_type {
                Some(n) => match n {
                    Some(n) => format!("Value{} ", n),
                    None => "Value ".to_string(),
                },
                None => "".to_string(),
            },
            id,
            operand_to_string(op)
        ),
        Statement::AssignField(id, index, op) => {
            format!("{}[{}] = {};", id, index, operand_to_string(op))
        }
        Statement::Decl(id) => format!("Value {};", id),
        Statement::InitConstructor(id, size) => {
            format!("void** {} = malloc({} * sizeof(Value));", id, size)
        }
        Statement::If(op) => format!("if ({}) {{", operand_to_string(op)),
        Statement::ElseIf(op) => format!("else if ({}) {{", operand_to_string(op)),
        Statement::EndIf => "}".to_string(),
        Statement::Return(op) => format!("return {};", operand_to_string(op)),
        Statement::Print(op) => format!("printf(\"%lld\\n\", {} >> 1);", operand_to_string(op)),
    }
}

fn operand_to_string(op: &Operand) -> String {
    match op {
        Operand::Application(id, ops) => {
            let result = ops
                .iter()
                .map(|op| operand_to_string(op))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({})", id, result)
        }
        Operand::BinOp(operator, op1, op2) => {
            let left = operand_to_string(op1);
            let right = operand_to_string(op2);
            match operator {
                Operator::Add => format!("{} + {} - 1", left, right),
                Operator::Sub => format!("{} - {} | 1", left, right),
                Operator::Mul => format!("(({} - 1) * ({} >> 1) | 1)", left, right),
                Operator::Div => format!("({} / ({} - 1)) << 1 | 1", left, right),
                op => format!("({} {} {}) << 1 | 1", left, op,  right)
            }
        }
        Operand::DerefField(id, index) => format!("((void**) {})[{}]", id, index),
        Operand::Identifier(id) => id.clone(),
        Operand::Integer(i) => (i << 1 | 1).to_string(),
        Operand::Condition(b, pointer_var, left, right) => {
            let not = if *b {""} else {"!"};
            format!("({}({} & 1)) && ({} == {})", not, pointer_var, operand_to_string(left), operand_to_string(right))
        },
        Operand::UTuple(operands) => {
            let result = operands
                .iter()
                .map(|op| operand_to_string(op))
                .collect::<Vec<_>>()
                .join(", ");
            format!("(Value{}){{{}}}", operands.len(), result)

        },
        Operand::AccessField(var, index) => format!("{}.elem{}", var, index),
    }
}
