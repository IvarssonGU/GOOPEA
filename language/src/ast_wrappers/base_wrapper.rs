use std::collections::HashMap;

use crate::{ast::{Definition, Expression, Pattern, Program, UTuple, FID, VID}, error::CompileError};

use super::ast_wrapper::{ConstructorReference, ExprWrapper, WrappedFunction, WrappedProgram};


pub type BaseWrapper = ExprWrapper<()>;
pub type BaseProgram = WrappedProgram<()>;

impl BaseProgram {
    pub fn new(program: Program) -> Result<BaseProgram, CompileError> {
        program.validate_top_level_ids();

        let mut adts = HashMap::new();
        let mut constructors = HashMap::new();
        let mut functions = HashMap::new();
        for def in program.0 {
            match def {
                Definition::ADTDefinition(def) => {
                    adts.insert(def.id.clone(), def.clone());
    
                    for (internal_id, cons) in def.constructors.iter().enumerate() {    
                        constructors.insert(cons.id.clone(), ConstructorReference { adt: def.id.clone(), constructor: cons.clone(), internal_id });
                    }
                },
                Definition::FunctionDefinition(def) => {    
                    functions.insert(
                        def.id.clone(), 
                        WrappedFunction { 
                            signature: def.signature.clone(),
                            vars: def.variables.clone(),
                            body: def.body
                        }
                    );
                }
            }
        }

        Ok(BaseProgram {
            adts,
            constructors,
            functions
        })
    }

}

impl BaseWrapper {
    pub fn integer(x: i64) -> Self { Self::new((),Expression::Integer(x)) }

    pub fn variable(vid: VID) -> Self { Self::new((), Expression::Variable(vid)) }

    pub fn function_call(fid: FID, args: UTuple<Self>) -> Self {
        Self::new((), Expression::FunctionCall(fid, args))
    }

    pub fn operation(operation: String, l: Self, r: Self) -> Self {
        Self::function_call(operation, UTuple(vec![l, r]))
    }

    pub fn utuple(args: UTuple<Self>) -> Self {
        Self::new((), Expression::UTuple(args))
    }

    pub fn mtch(match_on: Self, cases: Vec<(Pattern, Self)>) -> Self {
        Self::new((), Expression::Match(Box::new(match_on), cases))
    }
}