use std::collections::HashMap;

use crate::{ast_wrappers::ast_wrapper::{Expression, Pattern, UTuple, FID, VID}, error::CompileError, grammar, lexer::Lexer};

use super::ast_wrapper::{Constructor, ExpressionNode, Type, WrappedFunction, WrappedProgram, AID};

pub type BaseWrapper = ExpressionNode<()>;
pub type BaseProgram = WrappedProgram<()>;

#[derive(Debug)]
pub enum Definition {
    ADT(AID, Vec<(FID, UTuple<Type>)>),
    Function(FID, WrappedFunction<()>)
}

impl BaseProgram {
    pub fn new(code: &str) -> Result<BaseProgram, CompileError> {
        //program.validate_top_level_ids();

        let program = grammar::ProgramParser::new().parse(Lexer::new(&code)).unwrap();

        let mut adts = HashMap::new();
        let mut all_constructors = HashMap::new();
        let mut functions = HashMap::new();
        for def in program {
            match def {
                Definition::ADT(aid, constructors) => {
                    adts.insert(aid.clone(), constructors.iter().map(|(fid, _)| fid.clone()).collect());
    
                    for (sibling_index, (fid, args)) in constructors.into_iter().enumerate() {    
                        all_constructors.insert(fid, Constructor { sibling_index, adt: aid.clone(), args });
                    }
                },
                Definition::Function(fid, def) => {    
                    functions.insert(fid, def);
                }
            }
        }

        Ok(BaseProgram {
            adts,
            constructors: all_constructors,
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