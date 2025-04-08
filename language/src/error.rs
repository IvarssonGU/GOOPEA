use crate::ast::ast::{Type, UTuple, AID, FID, VID};
use color_eyre::{eyre, Section, SectionExt};
use thiserror::Error;

impl CompileError {
    pub fn make_report(self, snippet: &str) -> eyre::Report {
        eyre::Report::new(self).with_section(|| snippet.to_string().header("Code snippet:"))
    }
}

#[derive(Debug, Clone, Error)]
pub enum CompileError {
    #[error("THIS A TEMPORARY ERROR DESCRIPTION. TODO: Write better")]
    UnknownFunction,
    #[error("THIS A TEMPORARY ERROR DESCRIPTION. TODO: Write better")]
    UnknownVariable(VID),
    #[error("THIS A TEMPORARY ERROR DESCRIPTION. TODO: Write better")]
    UnknownConstructor,
    #[error("THIS A TEMPORARY ERROR DESCRIPTION. TODO: Write better")]
    MultipleFunctionDefinitions(FID),
    #[error("THIS A TEMPORARY ERROR DESCRIPTION. TODO: Write better")]
    MultipleADTDefinitions(AID),
    #[error("THIS A TEMPORARY ERROR DESCRIPTION. TODO: Write better")]
    InconsistentVariableCountInFunctionDefinition,
    #[error("THIS A TEMPORARY ERROR DESCRIPTION. TODO: Write better")]
    WrongVariableCountInLetStatement,
    #[error("THIS A TEMPORARY ERROR DESCRIPTION. TODO: Write better")]
    WrongVariableCountInMatchCase,
    #[error("THIS A TEMPORARY ERROR DESCRIPTION. TODO: Write better")]
    WrongVariableCountInFunctionCall,
    #[error("THIS A TEMPORARY ERROR DESCRIPTION. TODO: Write better")]
    UnknownADTInType,

    #[error("THIS A TEMPORARY ERROR DESCRIPTION. TODO: Write better")]
    MissmatchedTypes,
    #[error("THIS A TEMPORARY ERROR DESCRIPTION. TODO: Write better")]
    UnexpectedUTuple,
    #[error("THIS A TEMPORARY ERROR DESCRIPTION. TODO: Write better")]
    WrongArgumentType(FID, UTuple<Type>, UTuple<Type>),
    #[error("THIS A TEMPORARY ERROR DESCRIPTION. TODO: Write better")]
    InvalidPatternInMatchCase,
    #[error("THIS A TEMPORARY ERROR DESCRIPTION. TODO: Write better")]
    MultipleOccurencesOfConstructorInMatch,
    #[error("THIS A TEMPORARY ERROR DESCRIPTION. TODO: Write better")]
    MultipleOccurencesOfIntInMatch,
    #[error("THIS A TEMPORARY ERROR DESCRIPTION. TODO: Write better")]
    NonExhaustiveMatch,
    #[error("THIS A TEMPORARY ERROR DESCRIPTION. TODO: Write better")]
    WrongReturnType,
    #[error("THIS A TEMPORARY ERROR DESCRIPTION. TODO: Write better")]
    MatchHasMultipleWildcards,
    #[error("THIS A TEMPORARY ERROR DESCRIPTION. TODO: Write better")]
    MatchHasCaseAfterWildcard,
    #[error("THIS A TEMPORARY ERROR DESCRIPTION. TODO: Write better")]
    MatchingOnTuple,
}