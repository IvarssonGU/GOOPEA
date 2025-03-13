use crate::ast::{Expression, FunctionDefinition, MatchCase, Type, UTuple, AID, FID, VID};

pub type CompileResult<'a> = Result<(), CompileError<'a>>;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum CompileError<'a> {
    UnknownFunction(&'a VID),
    UnknownVariable(&'a FID),
    UnknownConstructor(&'a FID),
    MultipleDefinitions(&'a str),
    InconsistentVariableCountInFunctionDefinition(&'a FunctionDefinition),
    WrongVariableCountInLetStatement(&'a Expression),
    WrongVariableCountInMatchCase(&'a MatchCase),
    WrongVariableCountInFunctionCall(&'a Expression),
    UnknownADTInType(&'a AID),
    LetHasNoFunctionCall(&'a Expression),

    MissmatchedTypes(&'a Expression),
    UnexpectedUTuple,
    WrongArgumentType(FID, UTuple<Type>, UTuple<Type>),
    InvalidOperationTypes,
    WrongVariableTypeInMatch,
    InvalidConstructorInMatchCase,
    MultipleOccurencesOfConstructorInMatch,
    NonExhaustiveMatch,
    WrongReturnType
}