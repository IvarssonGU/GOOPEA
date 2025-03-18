use std::collections::{HashMap, HashSet};

use crate::{error::{CompileError, CompileResult}, grammar, lexer::Lexer};

use super::ast::{Constructor, Expression, ExpressionNode, Function, Pattern, Program, Type, UTuple, AID, FID, VID};

pub type BaseNode = ExpressionNode<()>;
pub type BaseProgram = Program<()>;

#[derive(Debug)]
pub enum Definition {
    ADT(AID, Vec<(FID, UTuple<Type>)>),
    Function(FID, Function<()>)
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

        let program = BaseProgram { adts, constructors: all_constructors, functions };
        program.validate_top_level_ids()?;
        program.validate_all_types()?;

        Ok(program)
    }

    // Checks that there are no top level id conflicts
    fn validate_top_level_ids(&self) -> CompileResult {
        let mut top_level_fids = HashSet::new();
        for fid in self.functions.keys().chain(self.constructors.keys()) {
            if !top_level_fids.insert(fid.clone()) {
                return Err(CompileError::MultipleFunctionDefinitions(fid.clone()))
            }
        }

        let mut top_level_aids = HashSet::new();
        for aid in self.adts.keys() {
            if !top_level_aids.insert(aid.clone()) {
                return Err(CompileError::MultipleADTDefinitions(aid.clone()))
            }
        }

        Ok(())
    }

    // Checks so that all types use defined ADT names
    fn validate_all_types(&self) -> CompileResult {
        for cons in self.constructors.values() {
            cons.args.validate_in(self)?;
        }

        for (_, func) in &self.functions {
            func.signature.argument_type.validate_in(self)?;
            func.signature.result_type.validate_in(self)?;
        }

        Ok(())
    }
}

impl BaseNode {
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

impl Type {
    fn validate_in(&self, program: &BaseProgram) -> CompileResult {
        match self {
            Type::Int => Ok(()),
            Type::ADT(aid) => {
                if !program.adts.contains_key(aid) { 
                    Err(CompileError::UnknownADTInType) 
                } else { 
                    Ok(()) 
                }
            }
        }
    }
}

impl UTuple<Type> {
    fn validate_in(&self, program: &BaseProgram) -> CompileResult {
        for tp in &self.0 { tp.validate_in(program)?; }
        Ok(())
    }
}