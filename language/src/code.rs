use crate::ir::*;
use crate::simple_ast::*;

pub struct Compiler {
    var_counter: u32,
    generated_statements: Vec<Statement>,
    unboxed_tuple_structs: Vec<u8>,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            var_counter: 0,
            generated_statements: Vec::new(),
            unboxed_tuple_structs: Vec::new(),
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
        Prog(programs, self.unboxed_tuple_structs.clone())
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
            type_len: match def.return_type_len {
                0  => None,
                1 => None,
                n => Some(n),
            },
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
                    Some(None),
                    match_var.clone(),
                    result.clone(),
                ));
                for (i, case) in cases.iter().enumerate() {
                    let bool_exp = match &case.pattern {
                        Pattern::Identifier(_) => Operand::Integer(1),
                        Pattern::Integer(n) => Operand::Condition(
                            true,
                            match_var.clone(), 
                            Box::from(Operand::Identifier(match_var.clone())), 
                            Box::from(Operand::Integer(*n))
                        ), 
                        Pattern::Wildcard => Operand::Integer(1),
                        Pattern::Atom(tag) => Operand::Condition(
                            true,
                            match_var.clone(),
                            Box::from(Operand::Identifier(match_var.clone())),
                            Box::from(Operand::Integer(*tag))
                        ),
                        Pattern::Constructor(tag, _) => Operand::Condition(
                            false, 
                            match_var.clone(),
                            Box::from(Operand::DerefField(match_var.clone(), 0)),
                            Box::from(Operand::Integer(*tag))
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
                                Some(None),
                                id.clone(),
                                Operand::Identifier(match_var.clone()),
                            ));
                        }
                        Pattern::Constructor(_, options) => {
                            for i in 0..options.len() {
                                match &options[i] {
                                    None => (),
                                    Some(id) => self.generated_statements.push(Statement::Assign(
                                        Some(None),
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
                        None,
                        var.clone(),
                        match_exp,
                    ));
                    self.generated_statements.push(Statement::EndIf);
                }
                Operand::Identifier(var)
            },
            Expression::Let(vars, bind_exp, exp) => {
                let result_bind = self.compile_exp(bind_exp);
                let result_var = self.next_var();
                self.generated_statements.push(Statement::Assign(Some(Some(vars.len() as u8)), result_var.clone(), result_bind));
                for (i, var) in vars.iter().enumerate() {
                    self.generated_statements.push(
                        Statement::Assign(Some(None), var.clone(), Operand::AccessField(result_var.clone(), i as i64)),
                    );
                }
                self.compile_exp(exp)
            },
            Expression::UTuple(exps) => {
                self.unboxed_tuple_structs.push(exps.len() as u8);
                let results = exps.iter().map(|exp| self.compile_exp(exp)).collect();
                Operand::UTuple(results)
            }
        }
    }
}
