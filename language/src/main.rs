use std::{os::unix::raw::off_t, process::Termination, result};

mod ir;
mod typed_ast;
mod code;

use typed_ast::*;
fn main() {
    // Create a function expression with the right pattern matching logic.
    let build_exp = Expression::Match(
        Box::from(Expression::Identifier("n".to_string())), 
        vec![
            MatchCase{
                pattern: Pattern::Integer(0),
                body: Expression::Constructor(0, vec![])
            },
            MatchCase{
                pattern: Pattern::Identifier("x".to_string()),
                body: Expression::Constructor(1, vec![
                    Expression::Identifier("x".to_string()),
                    Expression::FunctionCall("build".to_string(), vec![
                        Expression::Operation(
                            Operator::Sub,
                            Box::from(Expression::Identifier("x".to_string())),
                            Box::from(Expression::Integer(1))
                        )
                    ])
                ])
            }
        ]
    );
    let sum_exp = Expression::Match(
        Box::from(Expression::Identifier("xs".to_string())),
        vec![
            MatchCase{
                pattern: Pattern::Atom(0),
                body: Expression::Integer(0)
            },
            MatchCase{
                pattern: Pattern::Constructor(1, vec![Some("x".to_string()), Some("rest".to_string())]),
                body: Expression::Operation(
                    Operator::Add, 
                    Box::from(Expression::Identifier("x".to_string())), 
                    Box::from(Expression::FunctionCall("sum".to_string(), vec![Expression::Identifier("rest".to_string())])))
            }
        ]

    );

    let main_exp = Expression::FunctionCall("sum".to_string(), vec![
        Expression::FunctionCall("build".to_string(), vec![Expression::Integer(100)])
    ]);
    let fun_build = FunctionDefinition {
        id: "build".to_string(),
        args: vec!["n".to_string()],
        body: build_exp
    };

    let fun_sum = FunctionDefinition {
        id: "sum".to_string(),
        args: vec!["xs".to_string()],
        body: sum_exp
    };

    let fun_main = FunctionDefinition {
        id: "main".to_string(),
        args: vec![],
        body: main_exp
    };
    let prog= vec![fun_build, fun_sum, fun_main];

    let mut compiler: code::Compiler = code::Compiler::new();
    let result = compiler.compile(&prog);


    let pretty_output = ir::output(result);
    for line in pretty_output {
        println!("{}", line);
    }
}
