use std::fmt::Display;

use crate::{ast::{ast::{Pattern, Type, UTuple, AID, FID, VID}, base::{SourceLocation, SourceReference}}, lexer::{LexicalError, Token}};
use itertools::Itertools;
use lalrpop_util::ParseError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct ErrorSource {
    pub start: SourceLocation,
    pub end: SourceLocation,
    pub snippet: String,
    pub lines: String
}

#[derive(Debug, thiserror::Error)]
pub struct Error {
    #[source]
    pub reason: ErrorReason,
    pub source: Option<ErrorSource>
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ErrorReason {
    #[error("Syntax Error: {0:?}")]
    SyntaxError(ParseError<usize, Token, LexicalError>),

    #[error("Unknown function '{0}'")]
    UnknownFunction(FID),
    #[error("Unknown variable '{0}'")]
    UnknownVariable(VID),
    #[error("Unknown constructor '{0}'")]
    UnknownConstructor(FID),
    #[error("Multiple definitions for function '{0}'")]
    MultipleFunctionDefinitions(FID),
    #[error("Multiple definition for ADT '{0}'")]
    MultipleADTDefinitions(AID),
    #[error("Inconsistent variable count in function '{fid}'. Signature suggests {signature}, and definition suggests {definition}")]
    InconsistentVariableCountInFunctionDefinition { fid: FID, signature: usize, definition: usize },
    #[error("Wrong variable count in let statement. Expected {expected}, but got {actual}")]
    WrongVariableCountInLetStatement { expected: usize, actual: usize },
    #[error("Wrong variable count for constructor '{fid}' in match case. Expected {expected}, but got {actual}")]
    WrongVariableCountInMatchCase { fid: String, expected: usize, actual: usize },
    #[error("Wrong variable count for function call of '{fid}'. Expected {expected}, but got {actual}")]
    WrongVariableCountInFunctionCall { fid: FID, expected: usize, actual: usize },
    #[error("Use of undeclared ADT '{0}'")]
    UnknownADTInType(AID),
    #[error("The program is missing a main function")]
    MissingMainFunction,
    //#[error("Function marked with FIP ")]
    //MissingMainFunction,

    #[error("Missmatched return types of match statement")]
    MissmatchedTypesInMatchCases,
    #[error("Unexpected tuple expression")]
    UnexpectedUTuple,
    #[error("Wrong argument type for function call of '{fid}'. Expected {expected}, but got {actual}")]
    WrongArgumentType{ fid: FID, expected: UTuple<Type>, actual: UTuple<Type> },
    #[error("Invalid pattern in match statement. Matching on a {match_on_type}, and invalid pattern is {pattern}")]
    InvalidPatternInMatchCase { match_on_type: Type, pattern: Pattern },
    #[error("Constructor '{0}' is matched on in multiple cases in match statement")]
    MultipleOccurencesOfConstructorInMatch(FID),
    #[error("Integer {0} is matched on in multiple cases in match statement")]
    MultipleOccurencesOfIntInMatch(i64),
    #[error("Match statement is non exhaustive")]
    NonExhaustiveMatch,
    #[error("Wrong return type for function '{fid}'. Expected {expected}, but got {actual}")]
    WrongReturnType { fid: FID, expected: UTuple<Type>, actual: UTuple<Type> },
    #[error("Match statement has multiple wild cards")]
    MatchHasMultipleWildcards,
    #[error("Match statement has pattern {0} after a wildcard")]
    MatchHasCaseAfterWildcard(Pattern),
    #[error("Matching on a tuple is not supported")]
    MatchingOnTuple,
}

impl Into<Error> for ErrorReason {
    fn into(self) -> Error{
        Error { reason: self, source: None }
    }
}

impl<'i> Error {
    pub fn attach_source<'s>(self, src: &SourceReference<'s>) -> Error{
        Error { source: Some(ErrorSource { start: src.start.clone(), end: src.end.clone(), snippet: src.snippet.to_string(), lines: src.lines.to_string() }), ..self }
    }
}

impl<'i> Error {
    pub fn new(reason: ErrorReason) -> Self {
        Error { reason: reason, source: None }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ERROR: {}", self.reason)?;

        if let Some(source) = &self.source {
            writeln!(f)?;
            writeln!(f, "Occured at {}-{}", source.start, source.end)?;
            writeln!(f)?;

            write!(f, "{}", source.lines.to_string().lines().enumerate().map(|(i, line)| format!("{}. {line}", source.start.line+i)).join("\n"))?;
        }

        Ok(())
    }
}