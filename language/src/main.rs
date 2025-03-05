mod code;
mod ir;
mod typed_ast;

use typed_ast::*;
fn main() {
    // Create a function expression with the right pattern matching logic.
    let build_exp = Expression::Match(
        Box::from(Expression::Identifier("n".to_string())),
        vec![
            MatchCase {
                pattern: Pattern::Integer(0),
                body: Expression::Constructor(0, vec![]),
            },
            MatchCase {
                pattern: Pattern::Identifier("x".to_string()),
                body: Expression::Constructor(
                    1,
                    vec![
                        Expression::Identifier("x".to_string()),
                        Expression::FunctionCall(
                            "build".to_string(),
                            vec![Expression::Operation(
                                Operator::Sub,
                                Box::from(Expression::Identifier("x".to_string())),
                                Box::from(Expression::Integer(1)),
                            )],
                        ),
                    ],
                ),
            },
        ],
    );
    let sum_exp = Expression::Match(
        Box::from(Expression::Identifier("xs".to_string())),
        vec![
            MatchCase {
                pattern: Pattern::Atom(0),
                body: Expression::Integer(1),
            },
            MatchCase {
                pattern: Pattern::Constructor(
                    1,
                    vec![Some("x".to_string()), Some("rest".to_string())],
                ),
                body: Expression::Operation(
                    Operator::Mul,
                    Box::from(Expression::Identifier("x".to_string())),
                    Box::from(Expression::FunctionCall(
                        "sum".to_string(),
                        vec![Expression::Identifier("rest".to_string())],
                    )),
                ),
            },
        ],
    );

    let reverse_exp = Expression::FunctionCall(
        "revh".to_string(),
        vec![
            Expression::Identifier("xs".to_string()),
            Expression::Constructor(0, vec![]),
        ],
    );

    let revh_exp = Expression::Match(
        Box::from(Expression::Identifier("xs".to_string())),
        vec![
            MatchCase {
                pattern: Pattern::Atom(0),
                body: Expression::Identifier("acc".to_string()),
            },
            MatchCase {
                pattern: Pattern::Constructor(
                    1,
                    vec![Some("y".to_string()), Some("ys".to_string())],
                ),
                body: Expression::FunctionCall(
                    "revh".to_string(),
                    vec![
                        Expression::Identifier("ys".to_string()),
                        Expression::Constructor(
                            1,
                            vec![
                                Expression::Identifier("y".to_string()),
                                Expression::Identifier("acc".to_string()),
                            ],
                        ),
                    ],
                ),
            },
        ],
    );

    let main_exp = Expression::FunctionCall(
        "sum".to_string(),
        vec![Expression::FunctionCall(
            "rev".to_string(),
            vec![Expression::FunctionCall(
                "build".to_string(),
                vec![Expression::Integer(10)],
            )],
        )],
    );
    let fun_build = FunctionDefinition {
        id: "build".to_string(),
        args: vec!["n".to_string()],
        body: build_exp,
    };

    let fun_sum = FunctionDefinition {
        id: "sum".to_string(),
        args: vec!["xs".to_string()],
        body: sum_exp,
    };

    let fun_main = FunctionDefinition {
        id: "main".to_string(),
        args: vec![],
        body: main_exp,
    };

    let fun_reverse = FunctionDefinition {
        id: "rev".to_string(),
        args: vec!["xs".to_string()],
        body: reverse_exp,
    };

    let fun_revh = FunctionDefinition {
        id: "revh".to_string(),
        args: vec!["xs".to_string(), "acc".to_string()],
        body: revh_exp,
    };
    let prog = vec![fun_build, fun_sum, fun_revh, fun_reverse, fun_main];

    let mut compiler: code::Compiler = code::Compiler::new();
    let result = compiler.compile(&prog);

    let pretty_output = ir::output(result);
    for line in pretty_output {
        println!("{}", line);
    }
}
