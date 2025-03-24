use std::collections::HashMap;

use crate::error::{CompileError, CompileResult};

use super::{ast::{ExpressionNode, FullExpression, FunctionData, Pattern, Program, UTuple, FID, VID}, typed::{TypedData, TypedNode, TypedProgram}};

pub type FIPData = TypedData;

/*pub type FIPNode = ExpressionNode<FIPData, FIPExpression<FIPData>>;
pub type FIPProgram = Program<FIPData, FIPExpression<FIPData>>;

impl FIPProgram {
    pub fn new<'a>(program: TypedProgram) -> Result<Self, CompileError> {
        let functions = program.function_datas.into_iter().map(|(fid, func)| {
            func.body.fippify()
                .map(|node| (fid.clone(), FunctionData { body: node, signature: func.signature, vars: func.vars }))
        }).collect::<Result<HashMap<_, _>, _>>()?;

        let program = FIPProgram { adts: program.adts, constructors: program.constructors, function_datas: functions };
        //program.validate_fip()?;

        Ok(program)
    }
}

impl TypedNode {
    fn fippify(self) -> Result<FIPNode, CompileError>  {

    }
}

#[derive(Debug)]
pub enum FIPExpression<D> {
    UTuple(UTuple<ExpressionNode<D, Self>>),
    FunctionCall(FID, UTuple<ExpressionNode<D, Self>>),
    Integer(i64),
    Variable(VID),
    Match(Box<ExpressionNode<D, Self>>, Vec<(Pattern, ExpressionNode<D, Self>)>),
    FreshConstructor(FID, UTuple<ExpressionNode<D, Self>>),
    ReusedConstructor(FID, UTuple<ExpressionNode<D, Self>>, usize)
}

#[derive(Debug)]
enum ExplicitConstructorExpression<D> {
    UTuple(UTuple<ExpressionNode<D, Self>>),
    FunctionCall(FID, UTuple<ExpressionNode<D, Self>>),
    Integer(i64),
    Variable(VID),
    Match(Box<ExpressionNode<D, Self>>, Vec<(Pattern, ExpressionNode<D, Self>)>),
    FreshConstructor(FID, UTuple<ExpressionNode<D, Self>>),
    ReusedConstructor(FID, UTuple<ExpressionNode<D, Self>>, usize)
}

impl<'a, D> From<&'a FIPExpression<D>> for FullExpression<'a, D, FIPExpression<D>> {
    fn from(value: &'a FIPExpression<D>) -> Self {
        match value {
            FIPExpression::UTuple(x) => FullExpression::UTuple(x),
            FIPExpression::FunctionCall(x, y) |
            FIPExpression::FreshConstructor(x, y) |
            FIPExpression::ReusedConstructor(x, y, _) => FullExpression::FunctionCall(x, y),
            FIPExpression::Integer(x) => FullExpression::Integer(x),
            FIPExpression::Variable(x) => FullExpression::Variable(x),
            FIPExpression::Match(x, y) => FullExpression::Match(x, y),
        }
    }
}*/