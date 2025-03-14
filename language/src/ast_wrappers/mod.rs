use scope_wrapper::ScopedProgram;
use type_wrapper::TypedProgram;

pub mod ast_wrapper;
pub mod type_wrapper;
pub mod scope_wrapper;
pub mod base_wrapper;

pub type FullyWrappedProgram = TypedProgram;
pub type BaseWrappedProgram = ScopedProgram;