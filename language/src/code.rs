
use std::collections::HashMap;

use crate::{ast::*, cast::{self, Statement}};




pub struct Compiler {
    cons_map: HashMap<String, (u32, Vec<bool>)>,
    tag_counter: u32,
    generated_statements: Vec<cast::Statement>,
    var_counter: u32

}


//    data List = Nil | Cons Int List
//    data Blah = Cons klfd fk dsaj dsaji List

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            cons_map: HashMap::new(),
            tag_counter: 0,
            generated_statements: Vec::new(),
            var_counter: 0
        }
    }

    fn lookup_cons(&self, cons: &String) -> (u32, Vec<bool>) {
        self.cons_map.get(cons).unwrap().clone()
    }

    pub fn compile(&mut self, prog: Program) -> cast::Program {
        for def in prog.adt_definitions {
            for cons in def.constructors {
                self.compile_constructor(&cons);
            }
        }

        let mut defs = Vec::new();
        for fun in prog.fun_definitions {
            defs.push(self.compile_definition(&fun));
        }
        defs
    }

    fn next_var(&mut self) -> String {
        let ctr = self.var_counter;
        self.var_counter += 1;
        format!("var{}", ctr)
    }


    fn next_tag(&mut self) -> u32 {
        let tag = self.tag_counter;
        self.tag_counter += 1;
        tag
    }

    fn add_to_cons_map(&mut self, constructor: &String, types: Vec<bool>) {
        let tag = self.next_tag();
        self.cons_map.insert(constructor.to_string(), (tag, types));
    }

    fn compile_constructor(&mut self, constructor: &ConstructorDefinition) {
        let mut types = Vec::new();
        for t in &constructor.argument.0 {
            match t {
                Type::Int => types.push(true),
                _ => types.push(false)
            }
        }
        self.add_to_cons_map(&constructor.id.0, types);
    }

    fn compile_expression(&mut self, exp: &Expression) -> cast::Expression {
        match &exp {
            Expression::Integer(i) => cast::Expression::Integer(*i),
            Expression::Identifier(VID(ident)) => cast::Expression::Ident(ident.clone()),
            Expression::Operation(op, exp1, exp2) => {
               let result1 = self.compile_expression(exp1);
               let result2 = self.compile_expression(exp2);
               return cast::Expression::Operation(Box::from(result1), op.clone(), Box::from(result2));
            },
            Expression::FunctionCall(FID(id), TupleExpression(exps)) => {
                let mut results = Vec::new();
                for exp in exps {
                    results.push(self.compile_expression(exp));
                }
                return cast::Expression::Application(id.clone(), results);
            },
            Expression::Constructor(FID(id), exps) => {
                let (tag, types) = self.lookup_cons(id);
                let len = types.len();
                let new_var = self.next_var();
                self.generated_statements.push(cast::Statement::Init(cast::Type::Adt, new_var.clone(), cast::Expression::MallocAdt));
                self.generated_statements.push(cast::Statement::Assign(cast::Expression::AccessTag(Box::from(cast::Expression::Ident(new_var.clone()))), cast::Expression::Integer(tag as i32))); 
                //depending on size we need to malloc that many void pointers. 
                for i in 0..len {
                    let result = self.compile_expression(&exps[i]);
                    if types[i] {
                        //add code for int case
                    }
                    else {
                        self.generated_statements.push(Statement::Assign(cast::Expression::AccessData(Box::from(cast::Expression::Ident((new_var.clone()))), i as u32), result));
                    }
                }
                return cast::Expression::Ident(new_var);
            },
            Expression::Match(MatchExpression{exp, cases}, old_t) => {
                let new_var = self.next_var();
                let new_t = match old_t {
                    Type::Int => cast::Type::Int,
                    _ => cast::Type::Adt
                };
                self.generated_statements.push(Statement::Decl(new_t.clone(), new_var.clone()));
                let result = self.compile_expression(&exp);
                for (i, case) in cases.iter().enumerate() {
                    let bool_exp = match &case.pattern {
                        Pattern::Identifier(_) => cast::Expression::Integer(1),
                        Pattern::Integer(n) => cast::Expression::Operation(Box::from(cast::Expression::Integer(*n)), Operator::Equal, Box::from(result.clone())),
                        Pattern::Wildcard => cast::Expression::Integer(1),
                        Pattern::Constructor(FID(id), _) => {
                            let (tag, _) = self.lookup_cons(id);
                            cast::Expression::Operation(
                                Box::from(cast::Expression::Integer(tag as i32)), 
                                Operator::Equal, 
                                Box::from(cast::Expression::AccessTag(Box::from(result.clone())))
                            )
                        }
                    };
                    if i == 0 {
                        self.generated_statements.push(cast::Statement::If(bool_exp));
                    }
                    else {
                        self.generated_statements.push(cast::Statement::ElseIf(bool_exp));
                    }

                    match &case.pattern {
                        Pattern::Identifier(VID(id)) => {
                            self.generated_statements.push(cast::Statement::Init(new_t.clone(), id.to_string(), result.clone()));
                        },
                        Pattern::Constructor(FID(cons), idents) => {
                            let (_, types) = self.lookup_cons(cons);
                            for i in 0..idents.len() {
                                let t = match types[i] {
                                    true => cast::Type::Int,
                                    false => cast::Type::Adt
                                };
                                match &idents[i] {
                                    None => (),
                                    Some(VID(id)) => {
                                        self.generated_statements.push(cast::Statement::Init(
                                            t, 
                                            id.to_string(),
                                            cast::Expression::AccessData(Box::from(result.clone()), i as u32) 
                                        ))
                                    }
                                }
                            }
                        },
                        _ => ()
                    }

                    let match_exp = self.compile_expression(&case.body);
                    self.generated_statements.push(cast::Statement::Assign(cast::Expression::Ident(new_var.clone()), match_exp));
                    self.generated_statements.push(cast::Statement::EndIf);

                }
                return cast::Expression::Ident(new_var);
            }
            _ => todo!(),
        }
    }
   
    fn compile_definition(&mut self, def: &FunctionDefinition) -> cast::Definition {
        let new_t = match def.signature.result_type {
            Type::Int => cast::Type::Int,
            _ => cast::Type::Adt
        };
        let mut new_args = Vec::new();
        for i in 0..def.args.len() {
            let t = match &def.signature.argument_type.0[i] {
                Type::Int => cast::Type::Int,
                _ => cast::Type::Adt
            };
            new_args.push((t, def.args[i].clone()));
        };
        let new_id = def.id.0.clone();
        
        let operand = self.compile_expression(&def.body);
        self.generated_statements.push(cast::Statement::Return(operand));
        let def = cast::Definition {
            t: new_t,
            args: new_args,
            id: new_id,
            statements: self.generated_statements.clone()
        };
        self.generated_statements.clear();
        def
    }
}

