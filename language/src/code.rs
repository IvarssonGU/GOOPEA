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
        if &def.id == "Main" {
            statements.push(Statement::Print(result));
        } 
        else {
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
            Expression::Integer(i) => Operand::Int(*i),
            Expression::Ident(id) => Operand::Ident(id.clone()),
            Expression::Operation(op, exp1, exp2) => {
                let left = self.compile_exp(&exp1);
                let right = self.compile_exp(&exp2);
                let var = self.next_var();
                self.generated_statements.push(Statement::AssignBinaryOperation(
                    var.clone(), 
                    op.clone(),
                    left, 
                    right
                ));
                Operand::Ident(var.clone())
            }
            Expression::App(id, exps) => {
                let var = self.next_var();
                let arguments: Vec<Operand> =
                    exps.iter().map(|exp| self.compile_exp(exp)).collect();
                self.generated_statements.push(Statement::AssignFunctionCall(
                    var.clone(),
                    id.clone(),
                    arguments,
                ));
                Operand::Ident(var)
            }
            Expression::Constructor(tag, exps) => {
                let len = exps.len() + 3;
                let arguments: Vec<Operand> =
                    exps.iter().map(|exp| self.compile_exp(exp)).collect();
                let var = self.next_var();
                if len == 3 {
                    Operand::Int(*tag)
                } 
                else {
                    self.generated_statements
                        .push(Statement::InitConstructor(var.clone(), len as i64));
                    self.generated_statements.push(Statement::AddToMemoryAllocated(len as i64));
                    self.generated_statements.push(Statement::AssignToField(
                        var.clone(),
                        0,
                        Operand::Int(*tag),
                    ));
                    self.generated_statements.push(Statement::AssignToField(
                        var.clone(),
                        1,
                        Operand::NonShifted(exps.len() as i64),
                    ));
                    self.generated_statements.push(Statement::AssignToField(
                        var.clone(),
                        2,
                        Operand::NonShifted(1),
                    ));
                    for i in 3..len {
                        self.generated_statements.push(Statement::AssignToField(
                            var.clone(),
                            i as i64,
                            arguments[i - 3].clone(),
                        ));
                    }
                    Operand::Ident(var)
                }
            }
            Expression::Match(exp, cases) => {
                let var = self.next_var();
                self.generated_statements.push(Statement::Decl(var.clone()));
                let match_var = self.compile_exp(exp);
                let bool_vars = cases.iter().map(|((tag, binds), exp)| {
                    let bool_var = self.next_var();
                    self.generated_statements.push(Statement::AssignConditional(
                        bool_var.clone(),
                        binds.len() != 0,
                        match_var.clone(),
                        *tag << 1 | 1
                    ));
                    Operand::Ident(bool_var)
                }).collect::<Vec<Operand>>();
                for (i, ((_, binds), exp)) in cases.iter().enumerate() {
                    if i == cases.len() - 1 {
                        self.generated_statements.push(Statement::Else);
                    }
                    else if i == 0 {
                            self.generated_statements.push(Statement::If(bool_vars[i].clone()));
                    } 
                    else {
                        self.generated_statements.push(Statement::ElseIf(bool_vars[i].clone()));
                    }
                    if binds.len() > 0 {
                        let binders_var = self.next_var();
                        self.generated_statements.push(Statement::Assign(
                            Type::VoidPtrPtr, 
                            binders_var.clone(),
                            match_var.clone(),
                        ));
                        for i in 0..binds.len() {
                            match &binds[i] {
                                Binder::Wildcard => (),
                                Binder::Variable(id) => self.generated_statements.push(
                                    Statement::AssignFromField(
                                        id.clone(),
                                        i as i64 + 3,
                                        Operand::Ident(binders_var.clone())
                                    ),
                                )
                            }
                        }
                    }

                    let match_exp = self.compile_exp(exp);
                    self.generated_statements.push(Statement::Assign(
                        Type::None,
                        var.clone(),
                        match_exp,
                    ));
                    self.generated_statements.push(Statement::EndIf);
                }
                Operand::Ident(var)
            },
            Expression::Inc(id, exp) => {
                self.generated_statements.push(Statement::Inc(Operand::Ident(id.clone())));
                self.compile_exp(exp)
            },
            Expression::Dec(id, exp) => {
                self.generated_statements.push(Statement::Dec(Operand::Ident(id.clone())));
                self.compile_exp(exp)
            },
            Expression::Let(id, bind_exp, exp) => {
                let result_bind = self.compile_exp(bind_exp);
                self.generated_statements.push(Statement::Assign(Type::Value, id.clone(), result_bind));
                self.compile_exp(exp)
            },
            _ => panic!("Not implemented"),
        }
    }
}
