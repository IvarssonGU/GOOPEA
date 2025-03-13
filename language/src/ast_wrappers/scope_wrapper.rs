use std::{collections::HashMap, hash::Hash, rc::Rc, sync::atomic::AtomicUsize};

use crate::{ast::{Definition, Expression, Pattern, Program, Type, UTuple, FID, VID}, error::CompileError};

use super::{ast_wrapper::{ChainedData, ConstructorReference, ExprChildren, ExprWrapper, WrappedFunction, WrappedProgram}, base_wrapper::{BaseProgram, BaseWrapper, BaseWrapperData}, type_wrapper::ExpressionType};

pub type ScopeWrapperData = ChainedData<Scope, BaseWrapperData>;
pub type Scope = HashMap<VID, Rc<VariableDefinition>>;
pub type ScopeWrapper = ExprWrapper<ScopeWrapperData>;

pub type ScopedProgram = WrappedProgram<ScopeWrapperData>;

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
pub fn scope_expression<'a>(
    expr: BaseWrapper, 
    scope: Scope
) -> Result<ScopeWrapper, CompileError> 
{
    let children = match &expr.data {
        Expression::UTuple | Expression::FunctionCall(_) => {
            let ExprChildren::Many(children) = expr.children else { panic!() };
            ExprChildren::Many(children.into_iter().map(|expr| scope_expression(expr, scope.clone())).collect::<Result<_, _>>()?)
        },
        Expression::Integer(_) | Expression::Variable(_) => ExprChildren::Zero,
        Expression::Match(patterns) => {
            let ExprChildren::Match(base_match_child, base_case_children) = expr.children else { panic!() };

            let scoped_match_child = scope_expression(*base_match_child, scope.clone())?;

            let case_scopes: Vec<ScopeWrapper> = patterns.iter().zip(base_case_children).map(|(pattern, child)| {
                match pattern {
                    Pattern::Integer(_) => scope_expression(child, scope.clone()),
                    Pattern::UTuple(vars) | Pattern::Constructor(_, vars) => {
                        scope_expression(
                            child,
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
                Box::new(scoped_match_child),
                case_scopes
            )
        }
    };

    Ok(ExprWrapper {
        children,
        data: ChainedData { data: scope, prev: expr.data }
    })
}

static COUNTER: AtomicUsize = AtomicUsize::new(0);
fn reset_internal_id_counter() {
    COUNTER.store(0, std::sync::atomic::Ordering::Relaxed);
}

pub fn get_new_internal_id() -> usize {
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

impl ScopedProgram {
    // Creates a new program with scope information
    // Performs minimum required validation, such as no top level symbol collisions
    pub fn new(program: BaseProgram) -> Result<ScopedProgram, CompileError> {
        let mut functions = HashMap::new();
        for (fid, func) in program.functions {
            let base_scope = func.vars.0.iter().map(
                |vid| {
                    (vid.clone(), Rc::new(VariableDefinition { id: vid.clone(), internal_id: get_new_internal_id() }))
                }
            ).collect::<Scope>();

            let scoped_expression = scope_expression(func.body, base_scope)?;

            functions.insert(
                fid, 
                WrappedFunction { 
                    signature: func.signature,
                    vars: func.vars,
                    body: scoped_expression
                }
            );
        }

        Ok(ScopedProgram {
            adts: program.adts,
            constructors: program.constructors,
            functions
        })
    }

    /*pub fn validate(&self) -> CompileResult {
        self.validate_all_types()?;

        for (_, func) in &self.functions {
            func.body.validate_recursively(self)?;
            
            let return_type = match &func.body.data.0 {
                ExpressionType::UTuple(utuple) => utuple.clone(),
                ExpressionType::Type(tp) => UTuple(vec![tp.clone()]),
            };

            if return_type != func.def.signature.result_type {
                return Err(CompileError::WrongReturnType)
            }

            if func.def.signature.is_fip {
                let used_vars = func.body.recursively_validate_fip_expression(self)?;
                // Used can't contain any other variables than those defined for the function
                // since all variables are guaranteed to have a definition. All variables declared in expressions will already have been checked.

                let func_vars = func.body.data.1.values().map(|x| &**x).collect::<HashSet<_>>();
                let mut unused_vars = func_vars.difference(&used_vars);

                if let Some(unused_var) = unused_vars.next() {
                    return Err(CompileError::FIPFunctionHasUnusedVar(unused_var.id.clone()))
                }
            }
        }

        Ok(())
    }

    // Checks so that all types use defined ADT names
    fn validate_all_types(&self) -> CompileResult {
        for (_, cons) in &self.constructors {
            cons.constructor.arguments.validate_in(self)?;
        }

        for (_, func) in &self.functions {
            func.def.signature.validate_in(self)?;
        }

        Ok(())
    }*/

    pub fn get_constructor<'a>(&self, fid: &'a FID) -> Result<&ConstructorReference, CompileError> {
        self.constructors.get(fid).ok_or_else(|| CompileError::UnknownConstructor)
    }
}