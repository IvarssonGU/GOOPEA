use std::{collections::HashMap, rc::Rc};

use ast_wrapper::{ConstructorReference, WrappedFunction, WrappedProgram};
use scope_wrapper::{get_new_internal_id, scope_expression, Scope, ScopedProgram, VariableDefinition};
use type_wrapper::{type_expression, TypedProgram};

use crate::{ast::{ConstructorSignature, Definition, FunctionSignature, Program, Type, UTuple, FID}, error::CompileError};

pub mod ast_wrapper;
pub mod type_wrapper;
pub mod scope_wrapper;
pub mod base_wrapper;

pub type FullyWrappedProgram = TypedProgram;
pub type BaseWrappedProgram = ScopedProgram;