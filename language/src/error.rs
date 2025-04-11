use crate::ast::{ast::{Pattern, Type, UTuple, AID, FID, VID}, base::SourceReference};
use color_eyre::{eyre, Section, SectionExt};
use itertools::Itertools;
use thiserror::Error;

pub trait AttachSource {
    fn attach_source(self, snippet: &SourceReference<'_>) -> Self;
}

impl AttachSource for eyre::Report {
    fn attach_source(self, source: &SourceReference<'_>) -> Self {
        self.with_section(|| source.snippet.to_string().lines().enumerate().map(|(i, line)| format!("{}. {line}", source.start.line+i)).join("\n")
            .header(format!("Source location {}:{}-{}:{}", source.start.line, source.start.char_offset, source.end.line, source.end.char_offset)))
    }
}

#[derive(Debug, Clone, Error)]
pub enum CompileError {
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