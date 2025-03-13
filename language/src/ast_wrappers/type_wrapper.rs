use std::collections::HashMap;

use crate::{ast::{ConstructorSignature, Expression, FunctionSignature, Pattern, Type, UTuple, FID}, error::{CompileError, CompileResult}};

use super::{ast_wrapper::{ChainedData, ExprChildren, ExprWrapper, WrappedFunction, WrappedProgram}, scope_wrapper::{ScopeWrapper, ScopeWrapperData, ScopedProgram}};

pub type TypeWrapperData = ChainedData<ExpressionType, ScopeWrapperData>;

pub type TypeWrapper<'a> = ExprWrapper<'a, TypeWrapperData>;
pub type TypedProgram<'a> = WrappedProgram<'a, TypeWrapperData>;

fn get_children_same_type<'a, 'b: 'a>(mut iter: impl Iterator<Item = &'a TypeWrapper<'b>>) -> Option<ExpressionType> {
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

impl Type {
    fn validate_in(&self, program: &TypedProgram) -> CompileResult {
        match self {
            Type::Int => Ok(()),
            Type::ADT(aid) => {
                if !program.adts.contains_key(aid) { 
                    Err(CompileError::UnknownADTInType(aid)) 
                } else { 
                    Ok(()) 
                }
            }
        }
    }
}

impl UTuple<Type> {
    fn validate_in(&self, program: &TypedProgram) -> CompileResult {
        for tp in &self.0 { tp.validate_in(program)?; }
        Ok(())
    }
}

impl FunctionSignature {
    fn validate_in(&self, program: &TypedProgram) -> CompileResult {
        self.argument_type.validate_in(program)?;
        self.result_type.validate_in(program)
    }
}

// Creates a ScopeExpressionNode recursively for the expression
// Each node contains a mapping from VID to VariableDefinition and the resulting type of the expression
// A variable definition contains type information 
// Checks that each case in match has correct number of arguments for the constructor
// Does type inference on variables and expression, with minimum type checking
pub fn type_expression<'a, 'b>(
    expr: ScopeWrapper<'a>,
    var_types: HashMap<usize, Type>,
    function_signatures: &'b HashMap<FID, FunctionSignature>
) -> Result<TypeWrapper<'a>, CompileError<'a>> 
{
    let (children, tp) = match expr.expr {
        Expression::UTuple(tup) => {
            let ExprChildren::Many(scoped_children) = expr.children else { panic!() };
            let typed_children = ExprChildren::Many(scoped_children.into_iter().map(|expr| type_expression(expr, var_types.clone(), function_signatures)).collect::<Result<_, _>>()?);
            
            let tp = ExpressionType::UTuple(UTuple(
                typed_children.all_children().iter().map(|s| s.data.tp().ok_or_else(|| CompileError::UnexpectedUTuple).map(|t| t.clone())).collect::<Result<_, _>>()?
            ));
            (typed_children, tp)
        },
        Expression::FunctionCall(fid, _) => {
            let ExprChildren::Many(scoped_children) = expr.children else { panic!() };
            let children = ExprChildren::Many(scoped_children.into_iter().map(|expr| type_expression(expr, var_types.clone(), function_signatures)).collect::<Result<_, _>>()?);
            
            let return_type = &function_signatures.get(fid).ok_or_else(|| CompileError::UnknownFunction(fid))?.result_type;
            let tp = if return_type.0.len() == 1 { ExpressionType::Type(return_type.0[0].clone()) } else { ExpressionType::UTuple(return_type.clone()) };
            (children, tp)
        },
        Expression::Integer(_) => (ExprChildren::Zero, ExpressionType::Type(Type::Int)),
        Expression::Variable(var) => 
        (
            ExprChildren::Zero, 
            ExpressionType::Type(var_types.get(&expr.data.get(var).unwrap().internal_id).ok_or_else(|| CompileError::UnknownVariable(var))?.clone())
        ),
        Expression::Match(match_expr) => {
            let ExprChildren::Match(match_child_scoped, case_children) = expr.children else { panic!() };

            let match_child_typed = type_expression(*match_child_scoped, var_types.clone(), function_signatures)?;

            let case_scopes: Vec<TypeWrapper<'_>> = match_expr.cases.iter().zip(case_children.into_iter()).map(|(case, child)| {
                match &case.pattern {
                    Pattern::Integer(_) => type_expression(child, var_types.clone(), function_signatures),
                    Pattern::UTuple(vars) => {
                        let types = match &match_child_typed.data.data {
                            ExpressionType::UTuple(tup) => tup.clone(),
                            ExpressionType::Type(tp) => UTuple(vec![tp.clone()]),
                        };

                        let mut var_types = var_types.clone();
                        var_types.extend(
                            vars.0.iter().zip(types.0.iter())
                            .map(|(vid, tp)| {
                                (child.data.get(vid).unwrap().internal_id, tp.clone())
                            })
                        );

                        type_expression( child, var_types, function_signatures)
                    },
                    Pattern::Constructor(fid, vars) => {
                        let cons_sig: &ConstructorSignature = &function_signatures.get(fid).ok_or(CompileError::UnknownConstructor(fid))?.argument_type;
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
                }
            }).collect::<Result<_, _>>()?;

            let tp = get_children_same_type(case_scopes.iter()).ok_or_else(|| CompileError::MissmatchedTypes(expr.expr))?;

            let children = ExprChildren::Match(
                Box::new(match_child_typed),
                case_scopes
            );

            (children, tp)
        }
    };

    Ok(ExprWrapper {
        expr: expr.expr,
        children,
        data: ChainedData { data: tp, prev: expr.data }
    })
}

impl<'a> TypedProgram<'a> {
    pub fn new(program: ScopedProgram<'a>) -> Result<Self, CompileError<'a>> {
        let mut all_function_signatures: HashMap<FID, FunctionSignature> = HashMap::new();
        for op in "+-/*".chars() {
            all_function_signatures.insert(op.to_string(), FunctionSignature { 
                argument_type: UTuple(vec![Type::Int, Type::Int]),
                result_type: UTuple(vec![Type::Int]),
                is_fip: true
            });
        }

        for (fid, cons) in &program.constructors {
            all_function_signatures.insert(
                fid.clone(), 
                FunctionSignature {
                    argument_type: cons.constructor.arguments.clone(),
                    result_type: UTuple(vec! [Type::ADT(fid.clone())]),
                    is_fip: true
                }
            );
        }

        for (fid, func) in &program.functions {
            all_function_signatures.insert(fid.clone(), func.signature.clone());
        }

        let functions = program.functions.into_iter().map(|(fid, func)| {
            let func_vars = &func.vars.0;
            let func_types = &func.signature.argument_type.0;

            if func_vars.len() != func_types.len() {
                return Err(CompileError::InconsistentVariableCountInFunctionDefinition);
            }

            let base_var_types = func_vars.iter().zip(func_types.iter()).map(
                |(vid, tp)| {
                    (func.body.data.get(vid).unwrap().internal_id, tp.clone())
                }
            ).collect::<HashMap<_, _>>();

            type_expression(func.body, base_var_types, &all_function_signatures).map(|body|
                (fid.clone(), WrappedFunction {
                    body,
                    signature: func.signature,
                    vars: func.vars
                })
            )
        }).collect::<Result<HashMap<_, _>, _>>()?;

        Ok(WrappedProgram {
            adts: program.adts,
            constructors: program.constructors,
            functions,
        })
    }
}