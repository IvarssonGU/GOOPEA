use std::{error::Error, fmt::Display};

use crate::ast::ast::{Type, UTuple, AID, FID, VID};

pub type CompileResult = Result<(), CompileError>;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum CompileError {
    UnknownFunction,
    UnknownVariable(VID),
    UnknownConstructor,
    MultipleFunctionDefinitions(FID),
    MultipleADTDefinitions(AID),
    InconsistentVariableCountInFunctionDefinition,
    WrongVariableCountInLetStatement,
    WrongVariableCountInMatchCase,
    WrongVariableCountInFunctionCall,
    UnknownADTInType,
    LetHasNoFunctionCall,

    MissmatchedTypes,
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
    FIPFunctionAllocatesMemory,
    FIPFunctionDeallocatesMemory
}

impl<'a> Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TODO: Better compile errors")
    }
}

impl Error for CompileError {
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