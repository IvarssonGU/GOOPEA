use super::core::Prog;
use super::crux::{from_exp_type, from_type, from_typed_expr};
use super::stir::remove_dead_bindings;
use super::stir::{self, Stir};
use super::stir::{Body, Function, from_simple};
use crate::ast::typed::TypedProgram;

pub struct CompiledProgram {
    pub stir: Stir,
    pub reuse: Stir,
    pub rc: Stir,
    pub core: Prog,
}

fn from_typed(typed: &TypedProgram) -> Stir {
    stir::reset_var_counter();
    let mut stir = vec![];
    for (id, func, body) in typed.function_iter() {
        stir.push(Function {
            fip: func.signature.is_fip,
            id: id.clone(),
            typ: from_exp_type(&body.data.data),
            args: func
                .vars
                .0
                .iter()
                .zip(func.signature.argument_type.0.iter())
                .map(|(var, typ)| (var.clone(), from_type(typ)))
                .collect(),
            body: remove_dead_bindings(from_simple(&from_typed_expr(body, typed), &|var| {
                Body::Ret(var)
            })),
        });
        from_typed_expr(body, typed);
    }
    stir
}

pub fn compile_typed(typed: &TypedProgram) -> CompiledProgram {
    let stir = from_typed(typed);
    let reuse = crate::compiler::reuse::add_reuse(&stir);
    let rc = crate::compiler::rc::add_rc(&reuse, true);
    let core = crate::compiler::score::translate(&rc);
    CompiledProgram {
        stir,
        reuse,
        rc,
        core,
    }
}

pub fn compile_with_scoped_rc(typed: &TypedProgram) -> CompiledProgram {
    let stir = from_typed(typed);
    let rc = crate::compiler::scoped_rc::add_rc(&stir);
    let core = crate::compiler::score::translate(&rc);
    CompiledProgram {
        stir: stir.clone(),
        reuse: stir,
        rc,
        core,
    }
}
