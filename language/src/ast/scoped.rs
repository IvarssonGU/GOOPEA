use std::{cell::RefCell, collections::HashMap, hash::Hash, rc::Rc};

use crate::error::{CompileError, CompileResult};

use super::{ast::{SimplifiedExpression, ExpressionNode, Function, Pattern, Program, SyntaxExpression, UTuple, VID}, base::{BaseNode, BaseProgram}};

pub type Scope = HashMap<VID, Rc<VariableDefinition>>;
pub type ScopedNode = ExpressionNode<Scope, SimplifiedExpression<Scope>>;

pub type ScopedProgram = Program<Scope, SimplifiedExpression<Scope>>;

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

impl ScopedProgram {
    // Creates a new program with scope information
    // Performs minimum required validation, such as no top level symbol collisions
    pub fn new(program: BaseProgram) -> Result<ScopedProgram, CompileError> {
        let counter = RefCell::new(0);

        let mut functions = HashMap::new();
        for (fid, func) in program.functions {
            let base_scope = func.vars.0.iter().map(
                |vid| {
                    (vid.clone(), Rc::new(VariableDefinition { id: vid.clone(), internal_id: counter.replace_with(|&mut x| x + 1) }))
                }
            ).collect::<Scope>();

            let scoped_expression = scope_expression(func.body.map(), base_scope, &counter)?;

            functions.insert(
                fid, 
                Function { 
                    signature: func.signature,
                    vars: func.vars,
                    body: scoped_expression
                }
            );
        }

        let program =  ScopedProgram { adts: program.adts, constructors: program.constructors, functions };
        program.validate_variable_occurences()?;

        Ok(program)
    }

    fn validate_variable_occurences(&self) -> CompileResult {
        self.validate_expressions_by(
            |node| {
                let SimplifiedExpression::Variable(vid) = &node.expr else { return Ok(()) };
                if !node.data.contains_key(vid) { return Err(CompileError::UnknownVariable(vid.clone())) }

                Ok(())
            }
        )
    }
}

// Creates a ScopeExpressionNode recursively for the expression
// Each node contains a mapping from VID to VariableDefinition and the resulting type of the expression
// A variable definition contains type information 
// Checks that each case in match has correct number of arguments for the constructor
// Does type inference on variables and expression, with minimum type checking
pub fn scope_expression<'a>(
    expr: ExpressionNode<(), SimplifiedExpression<()>>, 
    scope: Scope,
    counter: &RefCell<usize>
) -> Result<ScopedNode, CompileError> 
{
    let expr = match expr.expr {
        SimplifiedExpression::UTuple(children) => {
            SimplifiedExpression::UTuple(UTuple(children.0.into_iter().map(|expr| scope_expression(expr, scope.clone(), counter)).collect::<Result<_, _>>()?))
        },
        SimplifiedExpression::FunctionCall(fid, children) => {
            SimplifiedExpression::FunctionCall(fid, UTuple(children.0.into_iter().map(|expr| scope_expression(expr, scope.clone(), counter)).collect::<Result<_, _>>()?))
        },
        SimplifiedExpression::Integer(x) => SimplifiedExpression::Integer(x),
        SimplifiedExpression::Variable(vid) => SimplifiedExpression::Variable(vid),
        SimplifiedExpression::Match(match_expr, cases) => {
            let scoped_match_child = scope_expression(*match_expr, scope.clone(), counter)?;

            let case_scopes = cases.into_iter().map(|(pattern, child)| {
                match &pattern {
                    Pattern::Integer(_) => scope_expression(child, scope.clone(), counter),
                    Pattern::UTuple(vars) | Pattern::Constructor(_, vars) => {
                        scope_expression(
                            child,
                            extended_scope(
                                &scope, 
                                vars.0.iter().map(|new_vid| {
                                    VariableDefinition {
                                        id: new_vid.clone(),
                                        internal_id: counter.replace_with(|&mut x| x + 1)
                                    }
                                })
                            ),
                            counter
                        )
                    }
                }.map(move |new_expr| (pattern, new_expr))
            }).collect::<Result<Vec<_>, _>>()?;

            SimplifiedExpression::Match(
                Box::new(scoped_match_child),
                case_scopes
            )
        }
    };

    Ok(ExpressionNode {
        expr,
        data: scope
    })
}