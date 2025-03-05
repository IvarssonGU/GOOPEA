use crate::ir::*;
use crate::typed_ast::*;

pub struct Compiler {
    var_counter: u32,
    generated_statements: Vec<Statement>,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            var_counter: 0,
            generated_statements: Vec::new(),
        }
    }

    fn next_var(&mut self) -> String {
        let ctr = self.var_counter;
        self.var_counter += 1;
        format!("var{}", ctr)
    }

    pub fn compile(&mut self, prog: &Program) -> Prog {
        let mut programs = Vec::new();
        for def in prog {
            programs.push(self.compile_def(def));
        }
        programs
    } 
    fn compile_def(&mut self, def: &FunctionDefinition) -> Def {
        let result = self.compile_exp(&def.body);
        let mut statements = self.generated_statements.clone();
        if &def.id == "main" {
            statements.push(Statement::Print(result));
        } else {
            statements.push(Statement::Return(result));
        }
        self.generated_statements.clear();
        Def {
            id: def.id.clone(),
            args: def.args.clone(),
            body: statements,
        }
    }

    fn compile_exp(&mut self, exp: &Expression) -> Operand {
        match exp {
            Expression::Integer(i) => Operand::Integer(*i),
            Expression::Identifier(id) => Operand::Identifier(id.clone()),
            Expression::Operation(op, exp1, exp2) => {
                let left = self.compile_exp(&exp1);
                let right = self.compile_exp(&exp2);
                Operand::BinOp(op.clone(), Box::from(left), Box::from(right))
            }
            Expression::FunctionCall(id, exps) => {
                let arguments = exps.iter().map(|exp| self.compile_exp(exp)).collect();
                Operand::Application(id.clone(), arguments)
            }
            Expression::Constructor(tag, exps) => {
                let len = exps.len() + 1;
                let arguments: Vec<Operand> =
                    exps.iter().map(|exp| self.compile_exp(exp)).collect();
                let var = self.next_var();
                if len == 1 {
                    Operand::Integer(*tag)
                } else {
                    self.generated_statements
                        .push(Statement::InitConstructor(var.clone(), len as i64));
                    self.generated_statements.push(Statement::AssignField(
                        var.clone(),
                        0,
                        Operand::Integer(*tag),
                    ));
                    for i in 1..len {
                        self.generated_statements.push(Statement::AssignField(
                            var.clone(),
                            i as i64,
                            arguments[i - 1].clone(),
                        ));
                    }
                    Operand::Identifier(var)
                }
            }
            Expression::Match(exp, cases) => {
                let var = self.next_var();
                let match_var = self.next_var();
                self.generated_statements.push(Statement::Decl(var.clone()));
                let result = self.compile_exp(exp);
                self.generated_statements.push(Statement::Assign(
                    true,
                    match_var.clone(),
                    result.clone(),
                ));
                for (i, case) in cases.iter().enumerate() {
                    let bool_exp = match &case.pattern {
                        Pattern::Identifier(_) => Operand::Integer(1),
                        Pattern::Integer(n) => Operand::BinOp(
                            Operator::Equal,
                            Box::from(Operand::Integer(*n)),
                            Box::from(Operand::Identifier(match_var.clone())),
                        ),
                        Pattern::Wildcard => Operand::Integer(1),
                        Pattern::Atom(tag) => Operand::Condition(true,
                            Box::from(Operand::Identifier(match_var.clone())),
                            Box::from(Operand::BinOp(
                                Operator::Equal,
                                Box::from(Operand::Integer(*tag)),
                                Box::from(Operand::Identifier(match_var.clone())),
                            ))
                        ),
                        Pattern::Constructor(tag, _) => Operand::Condition(false, 
                            Box::from(Operand::Identifier(match_var.clone())),
                            Box::from(Operand::BinOp(
                                Operator::Equal,
                                Box::from(Operand::Integer(*tag)),
                                Box::from(Operand::DerefField(match_var.clone(), 0)),
                            ))
                        ),
                    };
                    if i == 0 {
                        self.generated_statements.push(Statement::If(bool_exp));
                    } else {
                        self.generated_statements.push(Statement::ElseIf(bool_exp));
                    }

                    match &case.pattern {
                        Pattern::Identifier(id) => {
                            self.generated_statements.push(Statement::Assign(
                                true,
                                id.clone(),
                                Operand::Identifier(match_var.clone()),
                            ));
                        }
                        Pattern::Constructor(tag, options) => {
                            for i in 0..options.len() {
                                match &options[i] {
                                    None => (),
                                    Some(id) => self.generated_statements.push(Statement::Assign(
                                        true,
                                        id.to_string(),
                                        Operand::DerefField(match_var.clone(), i as i64 + 1),
                                    )),
                                }
                            }
                        }
                        _ => (),
                    }

                    let match_exp = self.compile_exp(&case.body);
                    self.generated_statements.push(Statement::Assign(
                        false,
                        var.clone(),
                        match_exp,
                    ));
                    self.generated_statements.push(Statement::EndIf);
                }
                Operand::Identifier(var)
            }
        }
    }
}
