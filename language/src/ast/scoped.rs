use std::{cell::RefCell, collections::{HashMap, HashSet}, hash::Hash, rc::Rc};

use crate::error::{CompileError, CompileResult};

use super::{ast::{map_expr_box, ExpressionNode, FullExpression, Pattern, Program, UTuple, FID, VID}, base::{BaseProgram, SyntaxExpression}};

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
        let program = program.map();

        let counter = RefCell::new(0);

        let atom_constructors = program.constructors.iter().filter_map(|(fid, cons)| (cons.args.0.len() == 0).then_some(fid.clone())).collect();
        let zero_argument_functions = program.function_datas.iter().filter_map(|(fid, sig)| (sig.vars.0.len() == 0).then_some(fid.clone())).collect();

        let program = program.transform_functions(|body, func, _| {
            let base_scope = func.vars.0.iter().map(
                |vid| {
                    (vid.clone(), Rc::new(VariableDefinition { id: vid.clone(), internal_id: counter.replace_with(|&mut x| x + 1) }))
                }
            ).collect::<Scope>();

            scope_expression(body, base_scope, &counter, &atom_constructors, &zero_argument_functions)
        })?;

        program.validate_variable_occurences()?;

        Ok(program)
    }

    // This should be redundant
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

pub fn scope_expression(
    expr: ExpressionNode<(), SimplifiedExpression<()>>, 
    scope: Scope,
    counter: &RefCell<usize>,
    atom_constructors: &HashSet<FID>,
    zero_argument_functions: &HashSet<FID>
) -> Result<ScopedNode, CompileError> 
{
    let expr = match expr.expr {
        SimplifiedExpression::UTuple(children) => {
                        SimplifiedExpression::UTuple(UTuple(children.0.into_iter().map(|expr| scope_expression(expr, scope.clone(), counter, atom_constructors, zero_argument_functions)).collect::<Result<_, _>>()?))
            },
        SimplifiedExpression::FunctionCall(fid, children) => {
                SimplifiedExpression::FunctionCall(fid, UTuple(children.0.into_iter().map(|expr| scope_expression(expr, scope.clone(), counter, atom_constructors, zero_argument_functions)).collect::<Result<_, _>>()?))
            },
        SimplifiedExpression::Integer(x) => SimplifiedExpression::Integer(x),
        SimplifiedExpression::Variable(vid) => {
                if scope.contains_key(&vid) { SimplifiedExpression::Variable(vid) }
                // This should never happen
                else if atom_constructors.contains(&vid) { SimplifiedExpression::FunctionCall(vid, UTuple(vec![])) }
                else if zero_argument_functions.contains(&vid) { SimplifiedExpression::FunctionCall(vid, UTuple(vec![])) }
                else { SimplifiedExpression::Variable(vid) }
            },
        SimplifiedExpression::Match(var_node, cases) => {
                let var_node = ExpressionNode { expr: var_node.expr, data: scope.clone() };

                let case_scopes = cases.into_iter().map(|(pattern, child)| {
                    match &pattern {
                        Pattern::Integer(_) => scope_expression(child, scope.clone(), counter, atom_constructors, zero_argument_functions),
                        Pattern::Variable(_) | Pattern::Constructor(_, _) => {
                            let vars = match &pattern {
                                Pattern::Constructor(_, vars) => vars.0.iter().collect(),
                                Pattern::Variable(vid) => vec![vid],
                                Pattern::Integer(_) => unreachable!(),
                            };

                            scope_expression(
                                child,
                                extended_scope(
                                    &scope, 
                                    vars.iter().map(|new_vid| {
                                        VariableDefinition {
                                            id: (*new_vid).clone(),
                                            internal_id: counter.replace_with(|&mut x| x + 1)
                                        }
                                    })
                                ),
                                counter,
                                atom_constructors,
                                zero_argument_functions
                            )
                        }
                    }.map(move |new_expr| (pattern, new_expr))
                }).collect::<Result<Vec<_>, _>>()?;

                SimplifiedExpression::Match(
                    var_node,
                    case_scopes
                )
            }
        SimplifiedExpression::LetEqualIn(vars, e1, e2) => {
            let e1 = scope_expression(*e1, scope.clone(), counter, atom_constructors, zero_argument_functions)?;

            let e2 = scope_expression(
                *e2,
                extended_scope(
                    &scope, 
                    vars.0.iter().map(|new_vid| {
                        VariableDefinition {
                            id: new_vid.clone(),
                            internal_id: counter.replace_with(|&mut x| x + 1)
                        }
                    })
                ),
                counter,
                atom_constructors,
                zero_argument_functions
            )?;

            SimplifiedExpression::LetEqualIn(vars, Box::new(e1), Box::new(e2))
        },
    };

    Ok(ExpressionNode {
        expr,
        data: scope
    })
}

#[derive(Debug)]
pub enum SimplifiedExpression<D> {
    UTuple(UTuple<ExpressionNode<D, Self>>),
    FunctionCall(FID, UTuple<ExpressionNode<D, Self>>),
    Integer(i64),
    Variable(VID),
    Match(ExpressionNode<D, VID>, Vec<(Pattern, ExpressionNode<D, Self>)>),
    LetEqualIn(UTuple<VID>, Box<ExpressionNode<D, Self>>, Box<ExpressionNode<D, Self>>),
}

impl<'a, D> From<&'a SimplifiedExpression<D>> for FullExpression<'a, D, SimplifiedExpression<D>> {
    fn from(value: &'a SimplifiedExpression<D>) -> Self {
        match value {
            SimplifiedExpression::UTuple(x) => FullExpression::UTuple(x),
            SimplifiedExpression::FunctionCall(x, y) => FullExpression::FunctionCall(x, y),
            SimplifiedExpression::Integer(x) => FullExpression::Integer(x),
            SimplifiedExpression::Variable(x) => FullExpression::Variable(x),
            SimplifiedExpression::Match(x, y) => FullExpression::MatchOnVariable(x, y),
            SimplifiedExpression::LetEqualIn(x, y, z) => FullExpression::LetEqualIn(x, y, z),
        }
    }
}

impl From<SyntaxExpression<()>> for SimplifiedExpression<()> {
    fn from(value: SyntaxExpression<()>) -> Self {
        match value {
            SyntaxExpression::UTuple(x) => SimplifiedExpression::UTuple(x.map()),
            SyntaxExpression::FunctionCall(x, y) => SimplifiedExpression::FunctionCall(x, y.map()),
            SyntaxExpression::Integer(x) => SimplifiedExpression::Integer(x),
            SyntaxExpression::Variable(x) => SimplifiedExpression::Variable(x),
            SyntaxExpression::Match(expr, cases) => {
                let new_cases = cases.into_iter().map(|(a, b)| (a, b.map())).collect();

                match &expr.expr {
                    SyntaxExpression::Variable(vid) => SimplifiedExpression::Match(ExpressionNode { expr: vid.clone(), data: () }, new_cases),
                    _ => SimplifiedExpression::LetEqualIn(
                        UTuple(vec!["<temp>".to_string()]), 
                        map_expr_box(expr),
                        Box::new(ExpressionNode { 
                            data: (), 
                            expr: SimplifiedExpression::Match(
                                ExpressionNode { expr: "<temp>".to_string(), data: () },
                                new_cases
                            ) 
                        })
                    )
                }
            },
            SyntaxExpression::LetEqualIn(x, y, z) => SimplifiedExpression::LetEqualIn(x, map_expr_box(y), map_expr_box(z)),
            SyntaxExpression::Operation(e1, op, e2) => {
                SimplifiedExpression::FunctionCall(op.to_string(), UTuple(vec![e1.map(), e2.map()]))
            }
        }
    }
}