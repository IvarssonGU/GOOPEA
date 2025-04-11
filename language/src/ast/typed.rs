use std::collections::{HashMap, HashSet};

use crate::error::{AttachSource, CompileError};
use color_eyre::{eyre::Report, Result};

use super::{ast::{ChainedData, ExpressionNode, FunctionSignature, Operator, Pattern, Program, Type, UTuple, FID}, base::SourceReference, scoped::{ScopedData, ScopedNode, ScopedProgram, SimplifiedExpression}};

pub type TypedData<'i> = ChainedData<ExpressionType, ScopedData<'i>>;

pub type TypedNode<'i> = ExpressionNode<TypedData<'i>, SimplifiedExpression<TypedData<'i>>>;
pub type TypedProgram<'i> = Program<TypedData<'i>, SimplifiedExpression<TypedData<'i>>>;

fn get_children_same_type<'a>(mut iter: impl Iterator<Item = &'a TypedNode<'a>>) -> Option<ExpressionType> {
    let tp = iter.next()?.data.data.clone();

    for x in iter {
        if *x.data != tp { return None; }
    }

    Some(tp)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpressionType {
    UTuple(UTuple<Type>),
    Type(Type)
}

impl ExpressionType {
    pub fn utuple(&self) -> Option<&UTuple<Type>> {
        match self {
            ExpressionType::UTuple(tup) => Some(tup),
            ExpressionType::Type(_) => None
        }
    }

    pub fn tp(&self) -> Option<&Type> {
        match self {
            ExpressionType::Type(tp) => Some(tp),
            ExpressionType::UTuple(_) => None,
        }
    }

    pub fn expect_tp(&self, src: &SourceReference<'_>) -> Result<&Type> {
        self.tp().ok_or_else(|| Report::new(CompileError::UnexpectedUTuple).attach_source(src))
    }
}

impl<'i> TypedNode<'i> {
    pub fn snippet(&self) -> &SourceReference<'_> {
        &self.data.next.next
    }
}

impl<'i> TypedProgram<'i> {
    pub fn new<'a>(program: ScopedProgram<'i>) -> Result<Self> {
        let mut all_function_signatures: HashMap<FID, FunctionSignature> = HashMap::new();
        for op in Operator::NUMERICAL {
            all_function_signatures.insert(op.to_string(), FunctionSignature { 
                argument_type: UTuple(vec![Type::Int, Type::Int]),
                result_type: UTuple(vec![Type::Int]),
                is_fip: true
            });
        }

        for op in Operator::COMPERATORS {
            all_function_signatures.insert(op.to_string(), FunctionSignature { 
                argument_type: UTuple(vec![Type::Int, Type::Int]),
                result_type: UTuple(vec![Type::ADT("Bool".to_string())]),
                is_fip: true
            });
        }

        for (fid, cons) in &program.constructors {
            all_function_signatures.insert(
                fid.clone(), 
                FunctionSignature {
                    argument_type: cons.args.clone(),
                    result_type: UTuple(vec! [Type::ADT(cons.adt.clone())]),
                    is_fip: true
                }
            );
        }

        for (fid, func) in &program.function_datas {
            all_function_signatures.insert(fid.clone(), func.signature.clone());
        }

        let program = program.transform_functions(|fid, body, func, _| {
            let func_vars = &func.vars.0;
            let func_types = &func.signature.argument_type.0;

            if func_vars.len() != func_types.len() {
                return Err(CompileError::InconsistentVariableCountInFunctionDefinition { fid: fid.clone(), signature: func_types.len(), definition: func_vars.len() }.into());
            }

            let base_var_types = func_vars.iter().zip(func_types.iter()).map(
                |(vid, tp)| {
                    (body.data.get(vid).unwrap().internal_id, tp.clone())
                }
            ).collect::<HashMap<_, _>>();

            type_expression(body, base_var_types, &all_function_signatures)
        })?;

        program.validate_expressions_by(|node| program.validate_function_call(node, &all_function_signatures))?;
        program.validate_expressions_by(|node| program.validate_match_pattern(node))?;
        program.validate_return_types()?;
        //program.validate_fip()?;

        Ok(program)
    }

    fn validate_return_types(&self) -> Result<()> {
        for (fid, func, body) in self.function_iter() {
            let return_type = match &body.data.data {
                ExpressionType::UTuple(utuple) => utuple.clone(),
                ExpressionType::Type(tp) => UTuple(vec![tp.clone()]),
            };

            if return_type != func.signature.result_type {
                return Err(Report::new(CompileError::WrongReturnType {fid: fid.clone(), expected: func.signature.result_type.clone(), actual: return_type}).attach_source(body.snippet()))
            }
        }

        Ok(())
    }

    /*fn validate_fip(&self) -> CompileResult {
        for (_, func, body) in self.function_iter() {
            if func.signature.is_fip {
                let used_vars = self.recursively_validate_fip_expression(body)?;
                // Used can't contain any other variables than those defined for the function
                // since all variables are guaranteed to have a definition. All variables declared in expressions will already have been checked.

                let func_vars = body.data.next.values().map(|x| &**x).collect::<HashSet<_>>();
                let mut unused_vars = func_vars.difference(&used_vars);

                if let Some(unused_var) = unused_vars.next() {
                    return Err(CompileError::FIPFunctionHasUnusedVar(unused_var.id.clone()))
                }
            }
        }

        Ok(())
    }*/

    fn validate_function_call(&self, node: &TypedNode, all_signatures: &HashMap<FID, FunctionSignature>) -> Result<()> {
        let SimplifiedExpression::FunctionCall(fid, args) = &node.expr else { return Ok(()) };

        let expected_arg_type = &all_signatures.get(fid).ok_or_else(|| Report::new(CompileError::UnknownFunction(fid.clone())).attach_source(node.snippet()))?.argument_type;

        if args.0.len() != expected_arg_type.0.len() {
            return Err(Report::new(CompileError::WrongVariableCountInFunctionCall {fid: fid.clone(), expected: expected_arg_type.0.len(), actual: args.0.len()}).attach_source(node.snippet()));
        }

        let arg_type: UTuple<Type> = UTuple(node.children().map(|child| child.data.expect_tp(child.snippet()).map(|x| x.clone())).collect::<Result<_, _>>()?);
        if &arg_type != expected_arg_type {
            return Err(Report::new(CompileError::WrongArgumentType{ fid: fid.clone(), actual: arg_type, expected: expected_arg_type.clone()}).attach_source(node.snippet()))
        }

        Ok(())
    }
    
    fn validate_match_pattern(&self, node: &TypedNode) -> Result<()> {
        let SimplifiedExpression::Match(match_on, cases) = &node.expr else { return Ok(()) };

        get_children_same_type(cases.iter().map(|t| &t.1))
            .ok_or_else(|| Report::new(CompileError::MissmatchedTypesInMatchCases).attach_source(node.snippet()))?;
    
        let mut has_wildcard = false;
        for (pattern, _) in cases {
            let mut case_is_wildcard = false;
            if let Pattern::Variable(_) = &pattern { case_is_wildcard = true }

            if has_wildcard {
                if !case_is_wildcard {
                    return Err(Report::new(CompileError::MatchHasCaseAfterWildcard(pattern.clone())).attach_source(node.snippet()))
                } else {
                    return Err(Report::new(CompileError::MatchHasMultipleWildcards).attach_source(node.snippet()))
                }
            }

            has_wildcard = has_wildcard || case_is_wildcard;
        }

        let ExpressionType::Type(tp) = &*match_on.data else {
            return Err(Report::new(CompileError::MatchingOnTuple).attach_source(node.snippet()));
        };

        match tp {
            Type::Int => {
                let mut used_ints = HashSet::new();
                for (pattern, _) in cases {
                    match pattern {
                        Pattern::Integer(i) => {
                            if !used_ints.insert(i) {
                                return Err(Report::new(CompileError::MultipleOccurencesOfIntInMatch(i.clone())).attach_source(node.snippet()));
                            }
                        },
                        Pattern::Constructor(_, _) => return Err(Report::new(CompileError::InvalidPatternInMatchCase{ pattern: pattern.clone(), match_on_type: tp.clone() }).attach_source(node.snippet())),
                        Pattern::Variable(_) => (),
                    }
                }

                if !has_wildcard {
                    return Err(Report::new(CompileError::NonExhaustiveMatch).attach_source(node.snippet()))
                }
            },
            Type::ADT(aid) => {
                let mut used_constructors = HashSet::new();
                for (pattern, _) in cases {
                    match pattern {
                        Pattern::Integer(_) => return Err(Report::new(CompileError::InvalidPatternInMatchCase { match_on_type: tp.clone(), pattern: pattern.clone() }).attach_source(node.snippet())),
                        Pattern::Variable(_) => (),
                        Pattern::Constructor(fid, vars) => {
                            let cons = self.constructors.get(fid).ok_or_else(|| Report::new(CompileError::UnknownConstructor(fid.clone())).attach_source(node.snippet()))?;
                            if &cons.adt != aid {
                                return Err(Report::new(CompileError::InvalidPatternInMatchCase { match_on_type: tp.clone(), pattern: pattern.clone() }).attach_source(node.snippet()));
                            }
        
                            if vars.0.len() != cons.args.0.len() {
                                return Err(Report::new(CompileError::WrongVariableCountInMatchCase { fid: fid.clone(), actual: vars.0.len(), expected: cons.args.0.len() }).attach_source(node.snippet()))
                            }
        
                            if !used_constructors.insert(fid) {
                                return Err(Report::new(CompileError::MultipleOccurencesOfConstructorInMatch(fid.clone())).attach_source(node.snippet()));
                            }
                        }
                    }
                }

                let adt_constructors = self.adts.get(aid).unwrap();
                if !has_wildcard && used_constructors.len() < adt_constructors.len() {
                    return Err(Report::new(CompileError::NonExhaustiveMatch).attach_source(node.snippet()))
                }
            },
        };

        Ok(())
    }

    // Returns a list of variables used within all paths of execution
    // TODO: Check for reuse pairs / allocations
    /*fn recursively_validate_fip_expression<'a, 'b: 'a>(&'a self, node: &'b TypedNode) -> Result<HashSet<&'b VariableDefinition>, CompileError>
    {
        let mut used_vars = HashSet::new();

        /*if let Expression::FunctionCall(fid, vars) = &self.expr {
            if program.constructors.contains_key(fid) && vars.0.len() > 0 {
                return Err(CompileError::FIPFunctionAllocatesMemory)
            }
        }*/

        match &node.expr {
            SimplifiedExpression::FunctionCall(_, _) | SimplifiedExpression::UTuple(_) => {
                for child in node.children() {
                    let child_used_vars = self.recursively_validate_fip_expression(child)?;
                    if let Some(double_var) = used_vars.intersection(&child_used_vars).next() {
                        return Err(CompileError::FIPFunctionHasMultipleUsedVar(double_var.id.clone()))
                    }

                    used_vars.extend(child_used_vars);
                }
            },
            SimplifiedExpression::Integer(_) => (),
            SimplifiedExpression::Variable(vid) => { used_vars.insert(node.data.next.get(vid).unwrap()); },
            SimplifiedExpression::Match(match_on, cases) => {
                used_vars.extend(self.recursively_validate_fip_expression(&match_on)?);

                let mut cases_used_vars = None;

                for (pattern, child) in cases {
                    let mut child_used_vars = self.recursively_validate_fip_expression(child)?;

                    match pattern {
                        Pattern::Integer(_) => (),
                        Pattern::Constructor(_, vars) | Pattern::UTuple(vars) => {
                            for vid in &vars.0 {
                                if !child_used_vars.remove(&**child.data.next.get(vid).unwrap()) {
                                    return Err(CompileError::FIPFunctionHasUnusedVar(vid.clone()))
                                }
                            }
                        },
                    }

                    if let Some(double_used_var) = child_used_vars.intersection(&used_vars).next() {
                        return Err(CompileError::FIPFunctionHasMultipleUsedVar(double_used_var.id.clone()))
                    }

                    if let Some(cases_used_vars) = &cases_used_vars {
                        let mut diff = child_used_vars.symmetric_difference(&cases_used_vars);

                        if let Some(unused_var) = diff.next() {
                            return Err(CompileError::FIPFunctionHasUnusedVar(unused_var.id.clone()))
                        }
                    } else { 
                        cases_used_vars = Some(child_used_vars);
                    };
                }

                if let Some(cases_used_vars) = cases_used_vars { used_vars.extend(cases_used_vars); }
            },
        }

        return Ok(used_vars)
    }*/
}

// Creates a ScopeExpressionNode recursively for the expression
// Each node contains a mapping from VID to VariableDefinition and the resulting type of the expression
// A variable definition contains type information 
// Checks that each case in match has correct number of arguments for the constructor
// Does type inference on variables and expression, with minimum type checking
pub fn type_expression<'i>(
    expr: ScopedNode<'i>,
    var_types: HashMap<usize, Type>,
    function_signatures: &HashMap<FID, FunctionSignature>
) -> Result<TypedNode<'i>> 
{
    let (new_expr, tp) = match expr.expr {
        SimplifiedExpression::UTuple(args) => {
                        let typed_args: Vec<_> = args.0.into_iter().map(|expr| type_expression(expr, var_types.clone(), function_signatures)).collect::<Result<_, _>>()?;
                    
                        let tp = ExpressionType::UTuple(UTuple(
                            typed_args.iter().map(|s| s.data.tp().ok_or_else(|| Report::new(CompileError::UnexpectedUTuple).attach_source(&expr.data.next)).map(|t| t.clone())).collect::<Result<_, _>>()?
                        ));
                        (SimplifiedExpression::UTuple(UTuple(typed_args)), tp)
            },
        SimplifiedExpression::FunctionCall(fid, args) => {
                let typed_args = args.0.into_iter().map(|expr| type_expression(expr, var_types.clone(), function_signatures)).collect::<Result<_, _>>()?;
            
                let return_type = &function_signatures.get(&fid)
                    .ok_or_else(|| Report::new(CompileError::UnknownFunction(fid.clone())).attach_source(&expr.data.next))?.result_type;
                let tp = if return_type.0.len() == 1 { ExpressionType::Type(return_type.0[0].clone()) } else { ExpressionType::UTuple(return_type.clone()) };
                (SimplifiedExpression::FunctionCall(fid, UTuple(typed_args)), tp)
            },
        SimplifiedExpression::Integer(x) => (SimplifiedExpression::Integer(x), ExpressionType::Type(Type::Int)),
        SimplifiedExpression::Variable(vid) => {
                let tp = ExpressionType::Type(var_types.get(&expr.data.get(&vid).unwrap().internal_id).ok_or_else(|| Report::new(CompileError::UnknownVariable(vid.clone())).attach_source(&expr.data.next))?.clone());

                (SimplifiedExpression::Variable(vid), tp)
            },
        SimplifiedExpression::Match(var_node, cases) => {
                let var_node = ExpressionNode::new(
                    ChainedData { data: ExpressionType::Type(var_types[&var_node.data[&var_node.expr].internal_id].clone()), next: var_node.data }, 
                    var_node.expr
                );

                let new_cases: Vec<(Pattern, TypedNode)> = cases.into_iter().map(|(pattern, child)| {
                    match &pattern {
                        Pattern::Integer(_) => type_expression(child, var_types.clone(), function_signatures),
                        Pattern::Variable(var) => {
                            let mut new_var_types = var_types.clone();
                            new_var_types.insert(expr.data[var].internal_id, var_node.data.data.expect_tp(&var_node.data.next.next).unwrap().clone());

                            type_expression( child, new_var_types, function_signatures)
                        },
                        Pattern::Constructor(fid, vars) => {
                            let cons_sig = &function_signatures.get(fid).ok_or(Report::new(CompileError::UnknownConstructor(fid.clone())).attach_source(&expr.data.next))?.argument_type;
                            if cons_sig.0.len() != vars.0.len() {
                                panic!("Wrong number of arguments in match statement of case {}", fid);
                            }

                            let mut var_types = var_types.clone();
                            var_types.extend(
                                vars.0.iter().zip(cons_sig.0.iter())
                                .map(|(vid, tp)| {
                                    (child.data.get(vid).unwrap().internal_id, tp.clone())
                                })
                            );

                            type_expression(child, var_types, function_signatures)
                        },
                    }.map(move |new_expr| (pattern, new_expr))
                }).collect::<Result<_, _>>()?;

                let tp = get_children_same_type(new_cases.iter().map(|(_, e)| e)).ok_or_else(|| Report::new(CompileError::MissmatchedTypesInMatchCases).attach_source(&expr.data.next))?;

                let new_expr = SimplifiedExpression::Match(
                    var_node,
                    new_cases
                );

                (new_expr, tp)
            }
        SimplifiedExpression::LetEqualIn(vars, e1, e2) => {
            let e1 = type_expression(*e1, var_types.clone(), function_signatures)?;
            
            let vt = match &e1.data.data {
                ExpressionType::UTuple(utuple) => utuple.0.clone(),
                ExpressionType::Type(tp) => vec![tp.clone()],
            };

            if vt.len() != vars.0.len() {
                return Err(Report::new(CompileError::WrongVariableCountInLetStatement { actual: vars.0.len(), expected: vt.len() }).attach_source(&expr.data.next))
            }

            let mut new_var_types = var_types;
            new_var_types.extend(vars.0.iter().map(|vid| e2.data[vid].internal_id).zip(vt.into_iter()));

            let e2 = type_expression(*e2, new_var_types.clone(), function_signatures)?;

            let tp = e2.data.data.clone();

            (
                SimplifiedExpression::LetEqualIn(vars, Box::new(e1), Box::new(e2)),
                tp
            )
        }
    };

    Ok(ExpressionNode {
        expr: new_expr,
        data: ChainedData { data: tp, next: expr.data }
    })
}