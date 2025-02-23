use ast::*;

mod ast;
mod code;

fn main() {
    let data = ADTDefinition {
        id: AID("List".to_string()),
        constructors: vec![
            ConstructorDefinition {
                id: FID("Nil".to_string()),
                argument: TupleType(vec![]),
            },
            ConstructorDefinition {
                id: FID("Cons".to_string()),
                argument: TupleType(vec![Type::Int, Type::ADT(AID("List".to_string()))])
            },
        ]
    };
    let exp = Expression::Constructor(FID("Cons".to_string()), vec![Expression::Integer(2), Expression::Constructor(FID("Nil".to_string()), vec![])]);
    let exp2 = Expression::Match(MatchExpression {
        exp: Box::new(Expression::Identifier(VID("".to_string()))),
        cases: vec![
            MatchCase{
                pattern: Pattern::Integer(0),
                body: Expression::Constructor(FID("Nil".to_string()), vec![])
            },
            MatchCase{
                pattern: Pattern::Identifier(VID("n".to_string())),
                body: Expression::Constructor(FID("Cons".to_string()), vec![
                    Expression::Identifier(VID("n".to_string())),
                    Expression::FunctionCall(FID("buildList".to_string()), TupleExpression(vec![
                        Expression::Operation(Operator::
                            Sub, Box::new(Expression::Identifier(VID("n".to_string()))), 
                            Box::new(Expression::Integer(-1)))
                    ]))
                ])
            }
        ]

    });
    let fun = FunctionDefinition {
        id: FID("main".to_string()),
        args: vec!["num".to_string()],
        body: exp,
        signature: FunctionSignature {
            argument_type: TupleType(vec![Type::Int]),
            result_type: Type::ADT(AID("List".to_string())),
            is_fip: false
        }
    };
    let fun2 = FunctionDefinition {
        id: FID("buildList".to_string()),
        args: vec!["n".to_string()],
        body: exp2,
        signature: FunctionSignature {
            argument_type: TupleType(Vec::new()),
            result_type: Type::ADT(AID("List".to_string())),
            is_fip: false
        }
    };
    let prog= Program(vec![Definition::ADTDefinition(data), Definition::FunctionDefinition(fun)]);
    let mut compiler: code::Compiler = code::Compiler::new();
    compiler.compile(prog);
    println!("{}", compiler.get_output());
}
