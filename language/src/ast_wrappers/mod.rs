use std::{collections::HashMap, rc::Rc};

use ast_wrapper::{ConstructorReference, WrappedFunction, WrappedProgram};
use scope_wrapper::{get_new_internal_id, scope_expression, Scope, ScopedProgram, VariableDefinition};
use type_wrapper::{type_expression, TypedProgram};

use crate::{ast::{ConstructorSignature, Definition, FunctionSignature, Program, Type, UTuple, FID}, error::CompileError};

pub mod ast_wrapper;
pub mod type_wrapper;
pub mod scope_wrapper;

pub type FullyWrappedProgram<'a> = TypedProgram<'a>;
pub type BaseWrappedProgram<'a> = ScopedProgram<'a>;

impl<'a> BaseWrappedProgram<'a> {
    // Creates a new program with scope information
    // Performs minimum required validation, such as no top level symbol collisions
    pub fn new(program: &'a Program) -> Result<BaseWrappedProgram<'a>, CompileError<'a>> {
        program.validate_top_level_ids();

        let mut all_function_signatures: HashMap<FID, FunctionSignature> = HashMap::new();
        for op in "+-/*".chars() {
            all_function_signatures.insert(op.to_string(), FunctionSignature { 
                argument_type: UTuple(vec![Type::Int, Type::Int]),
                result_type: UTuple(vec![Type::Int]),
                is_fip: true
            });
        }

        let mut constructor_signatures: HashMap<FID, &ConstructorSignature> = HashMap::new();
        for def in &program.0 {
            match def {
                Definition::ADTDefinition(def) => {
                    constructor_signatures.extend(def.constructors.iter().map(|cons| (cons.id.clone(), &cons.arguments)));

                    all_function_signatures.extend(def.constructors.iter().map(
                        |cons| {
                            (cons.id.clone(),
                                FunctionSignature {
                                    argument_type: cons.arguments.clone(),
                                    result_type: UTuple(vec! [Type::ADT(def.id.clone())]),
                                    is_fip: true
                                }
                            )
                        }
                    ));
                },
                Definition::FunctionDefinition(def) => {
                    all_function_signatures.insert(def.id.clone(), def.signature.clone());
                }
            }
        }

        let mut adts = HashMap::new();
        let mut constructors = HashMap::new();
        let mut functions = HashMap::new();
        for def in &program.0 {
            match def {
                Definition::ADTDefinition(def) => {
                    adts.insert(def.id.clone(), def);
    
                    for (internal_id, cons) in def.constructors.iter().enumerate() {    
                        constructors.insert(cons.id.clone(), ConstructorReference { adt: &def, constructor: &cons, internal_id });
                    }
                },
                Definition::FunctionDefinition(def) => {
                    if def.variables.0.len() != def.signature.argument_type.0.len() {
                        return Err(CompileError::InconsistentVariableCountInFunctionDefinition(def))
                    }
    
                    let base_scope = def.variables.0.iter().map(
                        |vid| {
                            (vid.clone(), Rc::new(VariableDefinition { id: vid.clone(), internal_id: get_new_internal_id() }))
                        }
                    ).collect::<Scope>();

                    let scoped_expression = scope_expression(&def.body, base_scope)?;
    
                    functions.insert(
                        def.id.clone(), 
                        WrappedFunction { 
                            def,
                            body: scoped_expression
                        }
                    );
                }
            }
        }

        Ok(BaseWrappedProgram {
            adts,
            constructors,
            functions,
            program,
            all_signatures: all_function_signatures
        })
    }

    /*pub fn validate(&self) -> CompileResult {
        self.validate_all_types()?;

        for (_, func) in &self.functions {
            func.body.validate_recursively(self)?;
            
            let return_type = match &func.body.data.0 {
                ExpressionType::UTuple(utuple) => utuple.clone(),
                ExpressionType::Type(tp) => UTuple(vec![tp.clone()]),
            };

            if return_type != func.def.signature.result_type {
                return Err(CompileError::WrongReturnType)
            }

            if func.def.signature.is_fip {
                let used_vars = func.body.recursively_validate_fip_expression(self)?;
                // Used can't contain any other variables than those defined for the function
                // since all variables are guaranteed to have a definition. All variables declared in expressions will already have been checked.

                let func_vars = func.body.data.1.values().map(|x| &**x).collect::<HashSet<_>>();
                let mut unused_vars = func_vars.difference(&used_vars);

                if let Some(unused_var) = unused_vars.next() {
                    return Err(CompileError::FIPFunctionHasUnusedVar(unused_var.id.clone()))
                }
            }
        }

        Ok(())
    }

    // Checks so that all types use defined ADT names
    fn validate_all_types(&self) -> CompileResult {
        for (_, cons) in &self.constructors {
            cons.constructor.arguments.validate_in(self)?;
        }

        for (_, func) in &self.functions {
            func.def.signature.validate_in(self)?;
        }

        Ok(())
    }*/

    pub fn get_constructor(&self, fid: &'a FID) -> Result<&ConstructorReference<'a>, CompileError<'a>> {
        self.constructors.get(fid).ok_or_else(|| CompileError::UnknownConstructor(fid))
    }
}

impl<'a> TypedProgram<'a> {
    pub fn new(program: ScopedProgram<'a>) -> Result<Self, CompileError<'a>> {
        let functions = program.functions.into_iter().map(|(fid, func)| {
            let base_var_types = func.def.variables.0.iter().zip(func.def.signature.argument_type.0.iter()).map(
                |(vid, tp)| {
                    (func.body.data.get(vid).unwrap().internal_id, tp.clone())
                }
            ).collect::<HashMap<_, _>>();

            type_expression(func.body, base_var_types, &program.all_signatures).map(|body|
                (fid.clone(), WrappedFunction {
                    def: func.def,
                    body: body
                })
            )
        }).collect::<Result<HashMap<_, _>, _>>()?;

        Ok(WrappedProgram {
            adts: program.adts,
            constructors: program.constructors,
            functions,
            all_signatures: program.all_signatures,
            program: program.program
        })
    }
}