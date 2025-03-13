use std::{collections::HashMap, hash::Hash, rc::Rc, sync::atomic::AtomicUsize};

use crate::{ast::{Expression, Pattern, Program, Type, UTuple, VID}, error::CompileError};

use super::{ast_wrapper::{ExprChildren, ExprWrapper, WrappedProgram}, type_wrapper::ExpressionType};

pub type ScopeWrapperData = Scope;
pub type Scope = HashMap<VID, Rc<VariableDefinition>>;
pub type ScopeWrapper<'a> = ExprWrapper<'a, ScopeWrapperData>;

pub type ScopedProgram<'a> = WrappedProgram<'a, ScopeWrapperData>;

#[derive(Clone, Debug, Eq)]
pub struct VariableDefinition {
    pub id: VID,
    pub internal_id: usize // Each definition is given a definitively different internal_id
}

impl PartialEq for VariableDefinition {
    fn eq(&self, other: &Self) -> bool {
       self.internal_id == other.internal_id
    }
}

impl Hash for VariableDefinition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.internal_id.hash(state);
    }
}

fn extended_scope(base: &Scope, new_vars: impl Iterator<Item = VariableDefinition>) -> Scope {
    let mut new_scope = base.clone();
    new_scope.extend(new_vars.map(|x| (x.id.clone(), Rc::new(x))));
    new_scope
}

// Creates a ScopeExpressionNode recursively for the expression
// Each node contains a mapping from VID to VariableDefinition and the resulting type of the expression
// A variable definition contains type information 
// Checks that each case in match has correct number of arguments for the constructor
// Does type inference on variables and expression, with minimum type checking
pub fn scope_expression<'a, 'b>(
    expr: &'a Expression, 
    scope: Scope
) -> Result<ScopeWrapper<'a>, CompileError<'a>> 
{
    let children = match expr {
        Expression::UTuple(tup) | Expression::FunctionCall(_, tup) => {
            ExprChildren::Many(tup.0.iter().map(|expr| scope_expression(expr, scope.clone())).collect::<Result<_, _>>()?)
        },
        Expression::Integer(_) | Expression::Variable(_) => ExprChildren::Zero,
        Expression::Match(match_expr) => {
            let match_child = scope_expression(&match_expr.expr, scope.clone())?;

            let case_scopes: Vec<ScopeWrapper<'_>> = match_expr.cases.iter().map(|case| {
                match &case.pattern {
                    Pattern::Integer(_) => scope_expression(&case.body, scope.clone()),
                    Pattern::UTuple(vars) | Pattern::Constructor(_, vars) => {
                        scope_expression(
                            &case.body,
                            extended_scope(
                                &scope, 
                                vars.0.iter().map(|new_vid| {
                                    VariableDefinition {
                                        id: new_vid.clone(),
                                        internal_id: get_new_internal_id()
                                    }
                                })
                            )
                        )
                    }
                }
            }).collect::<Result<_, _>>()?;

            ExprChildren::Match(
                Box::new(match_child),
                case_scopes
            )
        }
    };

    Ok(ExprWrapper {
        expr,
        children,
        data: scope
    })
}

static COUNTER: AtomicUsize = AtomicUsize::new(0);
fn reset_internal_id_counter() {
    COUNTER.store(0, std::sync::atomic::Ordering::Relaxed);
}

pub fn get_new_internal_id() -> usize {
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}