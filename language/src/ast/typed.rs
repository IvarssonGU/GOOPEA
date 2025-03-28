use std::collections::{HashMap, HashSet};

use crate::error::{CompileError, CompileResult};

use super::{ast::{ChainedData, ExpressionNode, FunctionSignature, Operator, Pattern, Program, Type, UTuple, FID}, scoped::{Scope, ScopedNode, ScopedProgram, SimplifiedExpression, VariableDefinition}};

pub type TypedData = ChainedData<ExpressionType, Scope>;

pub type TypedNode = ExpressionNode<TypedData, SimplifiedExpression<TypedData>>;
pub type TypedProgram = Program<TypedData, SimplifiedExpression<TypedData>>;

fn get_children_same_type<'a>(mut iter: impl Iterator<Item = &'a TypedNode>) -> Option<ExpressionType> {
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

    pub fn expect_tp(&self) -> Result<&Type, CompileError> {
        self.tp().ok_or_else(|| CompileError::UnexpectedUTuple)
    }
}

impl TypedProgram {
    pub fn new<'a>(program: ScopedProgram) -> Result<Self, CompileError> {
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

        let program = program.transform_functions(|body, func, _| {
            let func_vars = &func.vars.0;
            let func_types = &func.signature.argument_type.0;

            if func_vars.len() != func_types.len() {
                return Err(CompileError::InconsistentVariableCountInFunctionDefinition);
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

    fn validate_return_types(&self) -> CompileResult {
        for (_, func, body) in self.function_iter() {
            let return_type = match &body.data.data {
                ExpressionType::UTuple(utuple) => utuple.clone(),
                ExpressionType::Type(tp) => UTuple(vec![tp.clone()]),
            };

            if return_type != func.signature.result_type {
                return Err(CompileError::WrongReturnType)
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

    fn validate_function_call(&self, node: &TypedNode, all_signatures: &HashMap<FID, FunctionSignature>) -> CompileResult {
        let SimplifiedExpression::FunctionCall(fid, args) = &node.expr else { return Ok(()) };

        let expected_arg_type = &all_signatures.get(fid).ok_or_else(|| CompileError::UnknownFunction)?.argument_type;

        if args.0.len() != expected_arg_type.0.len() {
            return Err(CompileError::WrongVariableCountInFunctionCall);
        }

        let arg_type: UTuple<Type> = UTuple(node.children().map(|child| child.data.expect_tp().map(|x| x.clone())).collect::<Result<_, _>>()?);
        if &arg_type != expected_arg_type {
            return Err(CompileError::WrongArgumentType(fid.clone(), arg_type, expected_arg_type.clone()))
        }

        Ok(())
    }
    
    fn validate_match_pattern(&self, node: &TypedNode) -> CompileResult {
        let SimplifiedExpression::Match(match_on, cases) = &node.expr else { return Ok(()) };

        get_children_same_type(cases.iter().map(|t| &t.1)).ok_or_else(|| CompileError::MissmatchedTypes)?;
    
        let mut has_wildcard = false;
        for (pattern, _) in cases {
            let mut case_is_wildcard = false;
            if let Pattern::Variable(_) = &pattern { case_is_wildcard = true }

            if has_wildcard {
                if !case_is_wildcard {
                    return Err(CompileError::MatchHasCaseAfterWildcard)
                } else {
                    return Err(CompileError::MatchHasMultipleWildcards)
                }
            }

            has_wildcard = has_wildcard || case_is_wildcard;
        }

        let ExpressionType::Type(tp) = &*match_on.data else {
            return Err(CompileError::MatchingOnTuple);
        };

        match tp {
            Type::Int => {
                let mut used_ints = HashSet::new();
                for (pattern, _) in cases {
                    match pattern {
                        Pattern::Integer(i) => {
                            if !used_ints.insert(i) {
                                return Err(CompileError::MultipleOccurencesOfIntInMatch);
                            }
                        },
                        Pattern::Constructor(_, _) => return Err(CompileError::InvalidPatternInMatchCase),
                        Pattern::Variable(_) => (),
                    }
                }

                if !has_wildcard {
                    return Err(CompileError::NonExhaustiveMatch)
                }
            },
            Type::ADT(aid) => {
                let mut used_constructors = HashSet::new();
                for (pattern, _) in cases {
                    match pattern {
                        Pattern::Integer(_) => return Err(CompileError::InvalidPatternInMatchCase),
                        Pattern::Variable(_) => (),
                        Pattern::Constructor(fid, vars) => {
                            let cons = self.constructors.get(fid).ok_or_else(|| CompileError::UnknownConstructor)?;
                            if &cons.adt != aid {
                                return Err(CompileError::InvalidPatternInMatchCase);
                            }
        
                            if vars.0.len() != cons.args.0.len() {
                                return Err(CompileError::WrongVariableCountInMatchCase)
                            }
        
                            if !used_constructors.insert(fid) {
                                return Err(CompileError::MultipleOccurencesOfConstructorInMatch);
                            }
                        }
                    }
                }

                let adt_constructors = self.adts.get(aid).unwrap();
                if !has_wildcard && used_constructors.len() < adt_constructors.len() {
                    return Err(CompileError::NonExhaustiveMatch)
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
pub fn type_expression(
    expr: ScopedNode,
    var_types: HashMap<usize, Type>,
    function_signatures: &HashMap<FID, FunctionSignature>
) -> Result<TypedNode, CompileError> 
{
    let (new_expr, tp) = match expr.expr {
        SimplifiedExpression::UTuple(args) => {
                        let typed_args: Vec<_> = args.0.into_iter().map(|expr| type_expression(expr, var_types.clone(), function_signatures)).collect::<Result<_, _>>()?;
                    
                        let tp = ExpressionType::UTuple(UTuple(
                            typed_args.iter().map(|s| s.data.tp().ok_or_else(|| CompileError::UnexpectedUTuple).map(|t| t.clone())).collect::<Result<_, _>>()?
                        ));
                        (SimplifiedExpression::UTuple(UTuple(typed_args)), tp)
            },
        SimplifiedExpression::FunctionCall(fid, args) => {
                let typed_args = args.0.into_iter().map(|expr| type_expression(expr, var_types.clone(), function_signatures)).collect::<Result<_, _>>()?;
            
                let return_type = &function_signatures.get(&fid).ok_or_else(|| CompileError::UnknownFunction)?.result_type;
                let tp = if return_type.0.len() == 1 { ExpressionType::Type(return_type.0[0].clone()) } else { ExpressionType::UTuple(return_type.clone()) };
                (SimplifiedExpression::FunctionCall(fid, UTuple(typed_args)), tp)
            },
        SimplifiedExpression::Integer(x) => (SimplifiedExpression::Integer(x), ExpressionType::Type(Type::Int)),
        SimplifiedExpression::Variable(vid) => {
                let tp = ExpressionType::Type(var_types.get(&expr.data.get(&vid).unwrap().internal_id).ok_or_else(|| CompileError::UnknownVariable(vid.clone()))?.clone());

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
                            new_var_types.insert(expr.data[var].internal_id, var_node.data.data.expect_tp().unwrap().clone());

                            type_expression( child, new_var_types, function_signatures)
                        },
                        Pattern::Constructor(fid, vars) => {
                            let cons_sig = &function_signatures.get(fid).ok_or(CompileError::UnknownConstructor)?.argument_type;
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

                let tp = get_children_same_type(new_cases.iter().map(|(_, e)| e)).ok_or_else(|| CompileError::MissmatchedTypes)?;

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
                return Err(CompileError::WrongVariableCountInLetStatement)
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