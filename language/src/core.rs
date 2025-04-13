use std::fmt::{Display, Formatter};

//core = C-Oriented-Representation for Execution
use crate::stir::Operator;

//Warning this is currently not correct, we will look deeper into this later.

pub type Prog = Vec<Def>;

#[derive(Debug, Clone)]
pub struct Def {
    pub id: String,
    pub args: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Type {
    Value,
    VoidPtrPtr,
    None,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Value => write!(f, "Value "),
            Type::VoidPtrPtr => write!(f, "void** "),
            Type::None => write!(f, ""),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Operand {
    Ident(String),
    NonShifted(i64),
    Int(i64),
    Negate(String),
}

#[derive(Debug, Clone)]
pub enum Statement {
    IfElse(Vec<(Operand, Vec<Statement>)>),
    Return(Operand),
    Print(Operand),
    AssignMalloc(Type, String, u8),
    Assign(Type, String, Operand),
    AssignToField(String, i64, Operand),
    AssignFromField(String, i64, Operand),
    AssignBinaryOperation(String, Operator, Operand, Operand),
    AssignTagCheck(String, bool, Operand, i64),
    AssignFunctionCall(String, String, Vec<Operand>),
    AssignDropReuse(String, String),
    AssignUTuple(u8, String, Vec<String>),
    Inc(String),
    Dec(String),
}

pub fn output(prog: &Prog) -> Vec<String> {
    let mut lines = vec![
        "#include <stdio.h>".to_string(),
        "#include <stdlib.h>".to_string(),
        String::new(),
        "typedef __int64_t Value;".to_string(),
        String::new(),
    ];

    for def in prog {
        lines.push(output_function_decls(def));
    }
    lines.extend(vec![
        String::new(),
        "Value inc(Value ref) {".to_string(),
        "\tif (!(1 & ref)) {".to_string(),
        "\t\tvoid** ptr = ref;".to_string(),
        "\t\tptr[2]++;".to_string(),
        "\t}".to_string(),
        "\treturn ref;".to_string(),
        "}".to_string(),
        String::new(),
        "Value dec(Value ref) {".to_string(),
        "\tif (!(1 & ref)) {".to_string(),
        "\t\tvoid** ptr = ref;".to_string(),
        "\t\tif (ptr[2] == 1) {".to_string(),
        "\t\t\tfor (int i = 3; i < ptr[1] + 3; i++) {".to_string(),
        "\t\t\t\tdec(ptr[i]);".to_string(),
        "\t\t\t}".to_string(),
        "\t\t\tfree(ref);".to_string(),
        "\t\t}".to_string(),
        "\t\telse {".to_string(),
        "\t\t\tptr[2]--;".to_string(),
        "\t\t}".to_string(),
        "\t}".to_string(),
        "\treturn ref;".to_string(),
        "}".to_string(),
        String::new(),
    ]);
    lines.extend(vec![
        "void** drop_reuse(Value ref) {".to_string(),
        "\tif (((void**) ref)[2] == 1) {".to_string(),
        "\t\tfor (int i = 3; i < ((void**) ref)[1] + 3; i++) {".to_string(),
        "\t\t\tdec(((void**) ref)[i]);".to_string(),
        "\t\t}".to_string(),
        "\t\treturn ref;".to_string(),
        "\t}".to_string(),
        "\telse {".to_string(),
        "\t\t((void**) ref)[2]--;".to_string(),
        "\t\treturn NULL;".to_string(),
        "\t}".to_string(),
        "}".to_string(),
        String::new(),
    ]);

    for def in prog {
        let args_str = def
            .args
            .iter()
            .map(|arg| format!("Value {}", arg))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("Value {}({}) {{", def.id.clone(), args_str));
        let stmts_as_str = def
            .body
            .iter()
            .map(|stmt| statement_to_string(stmt, 1))
            .collect::<Vec<_>>();
        lines.extend(stmts_as_str);
        lines.push("}".to_string());
        lines.push(String::new());
    }
    lines
}

fn output_function_decls(def: &Def) -> String {
    let args_str = def
        .args
        .iter()
        .map(|arg| format!("Value {}", arg))
        .collect::<Vec<_>>()
        .join(", ");
    format!("Value {}({});", def.id.clone(), args_str)
}

fn statement_to_string(stmt: &Statement, depth: usize) -> String {
    let tab = "  ".repeat(depth);
    match stmt {
        Statement::Assign(t, id, op) => format!("{}{}{} = {};", tab, t, id, operand_to_string(op)),
        Statement::AssignToField(id, index, op) => {
            format!("{}{}[{}] = {};", tab, id, index, operand_to_string(op))
        }
        Statement::AssignFromField(id, index, op) => {
            format!(
                "{}Value {} = ((void**) {})[{}];",
                tab,
                id,
                operand_to_string(op),
                index
            )
        }
        Statement::AssignMalloc(t, var, size) => {
            format!(
                "{}{}{} = malloc({} * sizeof(Value));",
                tab,
                t,
                var,
                size + 3
            )
        }
        Statement::IfElse(branches) => {
            let mut lines = vec![];

            for (i, (cond, stmts)) in branches.iter().enumerate() {
                if i == 0 {
                    lines.push(format!("{}if ({}) {{", tab, operand_to_string(cond)));
                } else if i == branches.len() - 1 {
                    lines.push(format!("{}else {{", tab));
                } else {
                    lines.push(format!("{}else if ({}) {{", tab, operand_to_string(cond)));
                }
                for stmt in stmts {
                    lines.push(statement_to_string(stmt, depth + 1));
                }
                lines.push(format!("{}}}", tab));
            }
            lines.join("\n")
        }
        Statement::Return(op) => format!("{}return {};", tab, operand_to_string(op)),
        Statement::Print(op) => format!(
            "{}printf(\"%lld\\n\", {} >> 1);",
            tab,
            operand_to_string(op)
        ),
        Statement::AssignBinaryOperation(id, op, op1, op2) => {
            let left = operand_to_string(op1);
            let right = operand_to_string(op2);
            match op {
                Operator::Add => format!("{}Value {} = {} + {} - 1;", tab, id, left, right),
                Operator::Sub => format!("{}Value {} = {} - {} | 1;", tab, id, left, right),
                Operator::Mul => format!(
                    "{}Value {} = (({} - 1) * ({} >> 1) | 1);",
                    tab, id, left, right
                ),
                Operator::Div => format!(
                    "{}Value {} = ({} / ({} - 1)) << 1 | 1;",
                    tab, id, left, right
                ),
                op => format!(
                    "{}Value {} = ({} {} {}) << 1 | 1;",
                    tab, id, left, op, right
                ),
            }
        }
        Statement::AssignFunctionCall(var, fun, operands) => {
            format!(
                "{}Value {} = {}({});",
                tab,
                var,
                fun,
                operands
                    .iter()
                    .map(operand_to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        Statement::AssignTagCheck(id, b, op, tag) => {
            let result = operand_to_string(op);
            if *b {
                format!(
                    "{}Value {} = !(1 & {}) && {} == ((void**) {})[0];",
                    tab, id, result, tag, result
                )
            } else {
                format!("{}Value {} = {} == {};", tab, id, tag, result)
            }
        }
        Statement::AssignDropReuse(var, reset_var) => {
            format!("{}void** {} = drop_reuse({});", tab, var, reset_var)
        }
        Statement::AssignUTuple(size, var, args) => {
            format!("{}Value{} {} = {{{}}};", tab, size, var, args.join(", "))
        }

        Statement::Inc(var) => format!("{}inc({});", tab, var),
        Statement::Dec(var) => format!("{}dec({});", tab, var),
    }
}

fn operand_to_string(op: &Operand) -> String {
    match op {
        Operand::Ident(var) => var.clone(),
        Operand::Int(i) => (i << 1 | 1).to_string(),
        Operand::NonShifted(i) => i.to_string(),
        Operand::Negate(var) => format!("!{}", var),
    }
}
