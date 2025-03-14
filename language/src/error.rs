use std::{error::Error, fmt::Display};

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
    InvalidPatternInMatchCase,
    MultipleOccurencesOfConstructorInMatch,
    MultipleOccurencesOfIntInMatch,
    NonExhaustiveMatch,
    WrongReturnType,
    InvalidPattern,
    MatchHasMultipleWildcards,
    MatchHasCaseAfterWildcard,
    MatchHasMultipleTupleCases,
    InternalError,

    FIPFunctionHasUnusedVar(VID),
    FIPFunctionHasMultipleUsedVar(VID),
    FIPFunctionAllocatesMemory
}

impl<'a> Display for CompileError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TODO: Better compile errors")
    }
}

impl<'b> Error for CompileError<'b> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}