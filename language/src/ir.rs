use crate::simple_ast::Operator;

pub struct Prog(pub Vec<Def>, pub Vec<u8>);

#[derive(Debug, Clone)]
pub struct Def {
    pub type_len: Option<u8>,
    pub id: String,
    pub args: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Type {
    Value,
    UTuple(u8),
    VoidPtrPtr,
    None
}

#[derive(Debug, Clone)]
pub enum Operand {
    Ident(String),
    NonShifted(i64),
    Int(i64)
}

#[derive(Debug, Clone)]
pub enum Statement {
    Decl(String),
    If(Operand),
    ElseIf(Operand),
    Else,
    EndIf,
    InitConstructor(String, i64),
    Return(Operand),
    Print(Operand),
    Inc(Operand),
    Dec(Operand), 
    Assign(Type, String, Operand),
    AssignToField(String, i64, Operand),
    AssignFromField(String, i64, Operand),
    AssignBinaryOperation(String, Operator, Operand, Operand),
    AssignConditional(String, bool, Operand, i64),
    AssignFunctionCall(String, String, Vec<Operand>),
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
    lines.push("Value inc(Value ref) {".to_string());
    lines.push("\tif (!(1 & ref)) {".to_string());
    lines.push("\t\tvoid** ptr = ref;".to_string());
    lines.push("\t\tptr[2]++;".to_string());
    lines.push("\t}".to_string());
    lines.push("\treturn ref;".to_string());
    lines.push("}".to_string());

    lines.push(String::new());

    lines.push("Value dec(Value ref) {".to_string());
    lines.push("\tif (!(1 & ref)) {".to_string());
    lines.push("\t\tvoid** ptr = ref;".to_string());
    lines.push("\t\tif (ptr[2] == 1) {".to_string());
    lines.push("\t\t\tfor (int i = 3; i < ptr[1] + 3; i++) {".to_string());
    lines.push("\t\t\t\tdec(ptr[i]);".to_string());
    lines.push("\t\t\t}".to_string());
    lines.push("\t\t\tfree(ref);".to_string());
    lines.push("\t\t\tprintf(\"Tag: %lld, Len: %lld, RefCount: %lld\\n\", ptr[0], ptr[1], ptr[2]);".to_string());
    lines.push("\t\t}".to_string());
    lines.push("\t\telse {".to_string());
    lines.push("\t\t\tptr[2]--;".to_string());
    lines.push("\t\t}".to_string());
    lines.push("\t}".to_string());
    lines.push("\treturn ref;".to_string());
    lines.push("}".to_string());
    

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
                Statement::Else => {
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
        Statement::Assign(t, id, op) => format!(
            "{}{} = {};",
            match t {
                Type::Value => "Value ".to_string(),
                Type::UTuple(n) => format!("Value{} ", n),
                Type::VoidPtrPtr => "void** ".to_string(),
                Type::None => "".to_string()
            },
            id,
            operand_to_string(op)
        ),
        Statement::AssignToField(id, index, op) => {
            format!("{}[{}] = {};", id, index, operand_to_string(op))
        },
        Statement::AssignFromField(id, index, op) => {
            format!("Value {} = {}[{}];", id, operand_to_string(op), index)
        },
        Statement::Decl(id) => format!("Value {};", id),
        Statement::InitConstructor(id, size) => {
            format!("void** {} = malloc({} * sizeof(Value));", id, size)
        }
        Statement::If(op) => format!("if ({}) {{", operand_to_string(op)),
        Statement::ElseIf(op) => format!("else if ({}) {{", operand_to_string(op)),
        Statement::Else => "else {".to_string(),
        Statement::EndIf => "}".to_string(),
        Statement::Return(op) => format!("return {};", operand_to_string(op)),
        Statement::Print(op) => format!("printf(\"%lld\\n\", {} >> 1);", operand_to_string(op)),
        Statement::Inc(op) => format!("inc({});", operand_to_string(op)),
        Statement::Dec(op) => format!("dec({});", operand_to_string(op)),
        Statement::AssignBinaryOperation(id, op, op1, op2) => {
            let left = operand_to_string(op1);
            let right = operand_to_string(op2);
            match op {
                Operator::Add => format!("Value {} = {} + {} - 1;", id, left, right),
                Operator::Sub => format!("Value {} = {} - {} | 1;", id, left, right),
                Operator::Mul => format!("Value {} = (({} - 1) * ({} >> 1) | 1);", id, left, right),
                Operator::Div => format!("Value {} = ({} / ({} - 1)) << 1 | 1;", id, left, right),
                op => format!("Value {} = ({} {} {}) << 1 | 1;", id, left, op, right)
            }
        },
        Statement::AssignFunctionCall(var, fun, operands ) => {
            format!("Value {} = {}({});", var, fun, operands.iter().map(|op| operand_to_string(op)).collect::<Vec<_>>().join(", "))
        },
        Statement::AssignConditional(id, b, op, tag) => {
            let result = operand_to_string(op);
            if *b {
                format!("Value {} = !(1 & {}) && {} == ((void** {})[0];", id, result, tag, result)
            } else {
                format!("Value {} = {} == {};", id, tag, result)
            }
        }
    }
}

fn operand_to_string(op: &Operand) -> String {
    match op {
        Operand::Ident(id) => id.clone(),
        Operand::Int(i) => (i << 1 | 1).to_string(),
        Operand::NonShifted(i) => i.to_string()
        /* Operand::Application(id, ops) => {
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
        Operand::Inc(op) => format!("inc({})", operand_to_string(op)),
        Operand::Dec(op) => format!("dec({})", operand_to_string(op)), */
    }
}
