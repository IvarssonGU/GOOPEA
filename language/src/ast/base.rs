use std::collections::{HashMap, HashSet};

use crate::{error::{CompileError, CompileResult}, grammar, lexer::Lexer};

use super::ast::{Constructor, ExpressionNode, FullExpression, FunctionData, Operator, Pattern, Program, Type, UTuple, AID, FID, VID};

pub type BaseNode = ExpressionNode<(), SyntaxExpression<()>>;
pub type BaseProgram = Program<(), SyntaxExpression<()>>;

#[derive(Debug)]
pub enum Definition {
    ADT(AID, Vec<(FID, UTuple<Type>)>),
    Function(FID, (FunctionData, BaseNode))
}

impl BaseProgram {
    pub fn new(code: &str) -> Result<BaseProgram, CompileError> {
        //program.validate_top_level_ids();

        let program = grammar::ProgramParser::new().parse(Lexer::new(&code)).unwrap();

        let mut adts = HashMap::new();
        let mut all_constructors = HashMap::new();
        let mut function_datas = HashMap::new();
        let mut function_bodies = HashMap::new();
        for def in program {
            match def {
                Definition::ADT(aid, constructors) => {
                    adts.insert(aid.clone(), constructors.iter().map(|(fid, _)| fid.clone()).collect());
    
                    for (sibling_index, (fid, args)) in constructors.into_iter().enumerate() {    
                        all_constructors.insert(fid, Constructor { sibling_index, adt: aid.clone(), args });
                    }
                },
                Definition::Function(fid, (data, body)) => {    
                    function_datas.insert(fid.clone(), data);
                    function_bodies.insert(fid, body);
                }
            }
        }

        let program = BaseProgram { adts, constructors: all_constructors, function_datas, function_bodies };
        program.validate_top_level_ids()?;
        program.validate_all_types()?;

        Ok(program)
    }

    // Checks that there are no top level id conflicts
    fn validate_top_level_ids(&self) -> CompileResult {
        let mut top_level_fids = HashSet::new();
        for fid in self.function_datas.keys().chain(self.constructors.keys()) {
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

        for (_, func) in &self.function_datas {
            func.signature.argument_type.validate_in(self)?;
            func.signature.result_type.validate_in(self)?;
        }

        Ok(())
    }
}

impl BaseNode {
    pub fn integer(x: i64) -> Self { Self::new((),SyntaxExpression::Integer(x)) }

    pub fn variable(vid: VID) -> Self { Self::new((), SyntaxExpression::Variable(vid)) }

    pub fn function_call(fid: FID, args: UTuple<Self>) -> Self {
        Self::new((), SyntaxExpression::FunctionCall(fid, args))
    }

    pub fn operation(op: Operator, l: Self, r: Self) -> Self {
        Self::new((), SyntaxExpression::Operation(Box::new(l), op, Box::new(r)))
    }

    pub fn utuple(args: UTuple<Self>) -> Self {
        Self::new((), SyntaxExpression::UTuple(args))
    }

    pub fn mtch(match_on: Self, cases: Vec<(Pattern, Self)>) -> Self {
        Self::new((), SyntaxExpression::Match(Box::new(match_on), cases))
    }

    pub fn let_equal_in(pattern: Pattern, e1: Self, e2: Self) -> Self {
        Self::new((), SyntaxExpression::LetEqualIn(pattern, Box::new(e1), Box::new(e2)))
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

#[derive(Debug)]
pub enum SyntaxExpression<D> {
    UTuple(UTuple<ExpressionNode<D, Self>>),
    FunctionCall(FID, UTuple<ExpressionNode<D, Self>>),
    Integer(i64),
    Variable(VID),
    Match(Box<ExpressionNode<D, Self>>, Vec<(Pattern, ExpressionNode<D, Self>)>),
    LetEqualIn(Pattern, Box<ExpressionNode<D, Self>>, Box<ExpressionNode<D, Self>>),
    Operation(Box<ExpressionNode<D, Self>>, Operator, Box<ExpressionNode<D, Self>>)
}

impl<'a, D> From<&'a SyntaxExpression<D>> for FullExpression<'a, D, SyntaxExpression<D>> {
    fn from(value: &'a SyntaxExpression<D>) -> Self {
        match value {
            SyntaxExpression::UTuple(x) => FullExpression::UTuple(x),
            SyntaxExpression::FunctionCall(x, y) => FullExpression::FunctionCall(x, y),
            SyntaxExpression::Integer(x) => FullExpression::Integer(x),
            SyntaxExpression::Variable(x) => FullExpression::Variable(x),
            SyntaxExpression::Match(x, y) => FullExpression::Match(x, y),
            SyntaxExpression::LetEqualIn(x, y, z) => FullExpression::LetEqualIn(x, y, z),
            SyntaxExpression::Operation(x, y, z) => FullExpression::Operation(x, y, z)
        }
    }
}