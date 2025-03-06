mod code;
mod ir;
mod typed_ast;


/* 
data Tree = Empty | Node Tree Int Tree
data List = Nil | Cons Int List


insert :: Int -> Tree -> Tree
insert i tree = match tree of
    Node left v right -> match i < v of
        1 -> Node left v (insert i right)
        3 -> Node (insert i left) v right
    Empty -> Node Empty i Empty

sumTree :: Tree -> Int
sumTree tree = match tree of 
    Node left v right -> v + sumTree left + sumTree right
    Empty -> 0

buildTree :: List -> Tree
buildTree xs = match xs of 
    Nil -> Empty
    Cons y ys -> insert y (buildTree ys) 
    
fromTree :: Tree -> List
fromTree tree = match tree of 
    Node left v right -> concat(fromTree left) (Cons v (fromTree right))
    Empty -> Nil

concat :: List -> List 
concat xs ys = match xs of 
    Nil -> ys
    Cons z zs -> Cons z (concat zs ys)
*/

use typed_ast::*;
fn main() {
    let concat_exp = Expression::Match(
        Box::from(Expression::Identifier("xs".to_string())),
        vec![
            MatchCase {
                pattern: Pattern::Atom(0),
                body: Expression::Identifier("ys".to_string()),
            },
            MatchCase {
                pattern: Pattern::Constructor(1, vec![Some("z".to_string()), Some("zs".to_string())]),
                body: Expression::Constructor(
                    1,
                    vec![
                        Expression::Identifier("z".to_string()),
                        Expression::FunctionCall(
                            "concat".to_string(),
                            vec![
                                Expression::Identifier("zs".to_string()),
                                Expression::Identifier("ys".to_string()),
                            ],
                        ),
                    ],
                ),
            },
        ],
    );
    let from_tree_exp = Expression::Match(
        Box::from(Expression::Identifier("tree".to_string())),
        vec![
            MatchCase {
                pattern: Pattern::Constructor(3, vec![Some("left".to_string()), Some("v".to_string()), Some("right".to_string())]),
                body: Expression::FunctionCall(
                    "concat".to_string(),
                    vec![
                        Expression::FunctionCall(
                            "fromTree".to_string(),
                            vec![Expression::Identifier("left".to_string())],
                        ),
                        Expression::FunctionCall(
                            "concat".to_string(),
                            vec![
                                Expression::Constructor(
                                    1,
                                    vec![
                                        Expression::Identifier("v".to_string()),
                                        Expression::FunctionCall(
                                            "fromTree".to_string(),
                                            vec![Expression::Identifier("right".to_string())],
                                        ),
                                    ],
                                ),
                                Expression::Constructor(0, vec![]),
                            ],
                        ),
                    ],
                ),
            },
            MatchCase {
                pattern: Pattern::Atom(2),
                body: Expression::Constructor(0, vec![]),
            },
        ],
    );
    let insert_exp = Expression::Match(
        Box::from(Expression::Identifier("tree".to_string())),
        vec![
            MatchCase {
                pattern: Pattern::Constructor(3, vec![Some("left".to_string()), Some("v".to_string()), Some("right".to_string())]),
                body: Expression::Match(
                    Box::from(Expression::Operation(
                        Operator::Less,
                        Box::from(Expression::Identifier("i".to_string())),
                        Box::from(Expression::Identifier("v".to_string())),
                    )),
                    vec![
                        MatchCase {
                            pattern: Pattern::Integer(0),
                            body: Expression::Constructor(
                                3,
                                vec![
                                    Expression::Identifier("left".to_string()),
                                    Expression::Identifier("v".to_string()),
                                    Expression::FunctionCall(
                                        "insert".to_string(),
                                        vec![
                                            Expression::Identifier("i".to_string()),
                                            Expression::Identifier("right".to_string()),
                                        ],
                                    ),
                                ],
                            ),
                        },
                        MatchCase {
                            pattern: Pattern::Integer(1),
                            body: Expression::Constructor(
                                3,
                                vec![
                                    Expression::FunctionCall(
                                        "insert".to_string(),
                                        vec![
                                            Expression::Identifier("i".to_string()),
                                            Expression::Identifier("left".to_string()),
                                        ],
                                    ),
                                    Expression::Identifier("v".to_string()),
                                    Expression::Identifier("right".to_string()),
                                ],
                            ),
                        },
                    ],  
                ),
            },
            MatchCase {
                pattern: Pattern::Atom(2),
                body: Expression::Constructor(
                    3,
                    vec![
                        Expression::Constructor(2, vec![]),
                        Expression::Identifier("i".to_string()),
                        Expression::Constructor(2, vec![]),
                    ],
                ),
            }
        ]
    );

    let sum_tree_exp = Expression::Match(
        Box::from(Expression::Identifier("tree".to_string())),
        vec![
            MatchCase {
                pattern: Pattern::Constructor(3, vec![Some("left".to_string()), Some("v".to_string()), Some("right".to_string())]),
                body: Expression::Operation(
                    Operator::Add,
                    Box::from(Expression::Identifier("v".to_string())),
                    Box::from(Expression::Operation(
                        Operator::Add,
                        Box::from(Expression::FunctionCall(
                            "sumTree".to_string(),
                            vec![Expression::Identifier("left".to_string())],
                        )),
                        Box::from(Expression::FunctionCall(
                            "sumTree".to_string(),
                            vec![Expression::Identifier("right".to_string())],
                        )),
                    )),
                ),
            },
            MatchCase {
                pattern: Pattern::Atom(2),
                body: Expression::Integer(0),
            },
        ] 
    );
    let build_tree_exp = Expression::Match(
        Box::from(Expression::Identifier("xs".to_string())),
        vec![
            MatchCase {
                pattern: Pattern::Atom(0),
                body: Expression::Constructor(2, vec![]),
            },
            MatchCase {
                pattern: Pattern::Constructor(1, vec![Some("y".to_string()), Some("ys".to_string())]),
                body: Expression::FunctionCall(
                    "insert".to_string(),
                    vec![
                        Expression::Identifier("y".to_string()),
                        Expression::FunctionCall(
                            "buildTree".to_string(),
                            vec![Expression::Identifier("ys".to_string())],
                        ),
                    ],
                ),
            },
        ],
    );
    
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
        "sumTree".to_string(),
        vec![
            Expression::FunctionCall(
                "buildTree".to_string(),
                vec![
                    Expression::FunctionCall(
                        "build".to_string(),    
                        vec![Expression::Integer(100)]
                    )
                ] 
            )
        ]
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

    let fun_insert = FunctionDefinition {
        id: "insert".to_string(),
        args: vec!["i".to_string(), "tree".to_string()],
        body: insert_exp,
    };
    let fun_sum_tree = FunctionDefinition {
        id: "sumTree".to_string(),
        args: vec!["tree".to_string()],
        body: sum_tree_exp,
    };  
    let fun_build_tree = FunctionDefinition {
        id: "buildTree".to_string(),
        args: vec!["xs".to_string()],
        body: build_tree_exp,
    };
    let fun_from_tree = FunctionDefinition {
        id: "fromTree".to_string(),
        args: vec!["tree".to_string()],
        body: from_tree_exp,
    };
    let fun_concat = FunctionDefinition {
        id: "concat".to_string(),
        args: vec!["xs".to_string(), "ys".to_string()],
        body: concat_exp
    };
    let prog = vec![fun_build, fun_sum, fun_revh, fun_reverse, fun_insert, fun_sum_tree, fun_build_tree, fun_concat, fun_from_tree, fun_main];   

    let mut compiler: code::Compiler = code::Compiler::new();
    let result = compiler.compile(&prog);

    let pretty_output = ir::output(result);
    for line in pretty_output {
        println!("{}", line);
    }
}