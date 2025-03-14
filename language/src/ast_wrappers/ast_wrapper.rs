use std::{collections::{HashMap, HashSet}, fmt::{Display, Formatter}, hash::Hash, iter, ops::Deref, rc::Rc, sync::atomic::AtomicUsize};
use crate::{ast::{ADTDefinition, ConstructorDefinition, ConstructorSignature, Definition, Expression, FunctionDefinition, FunctionSignature, Pattern, Program, Type, UTuple, AID, FID, VID}, error::{CompileError, CompileResult}};

use super::base_wrapper::BaseWrapper;

#[derive(Debug)]
pub struct WrappedProgram<ED> {
    pub adts: HashMap<AID, ADTDefinition>,
    pub constructors: HashMap<FID, ConstructorReference>,
    pub functions: HashMap<FID, WrappedFunction<ED>>,
}

#[derive(Debug)]
pub struct WrappedFunction<D> {
    pub vars: UTuple<VID>,
    pub signature: FunctionSignature,
    pub body: ExprWrapper<D>,
}

#[derive(Debug)]
pub struct ExprWrapper<D> {
    pub children: ExprChildren<D>,
    pub expr: Expression,
    pub data: D
}

#[derive(Debug, Clone)]
pub struct ChainedData<D, P> {
    pub data: D,
    pub prev: P
}

#[derive(Debug, Clone)]
pub struct ConstructorReference {
    pub adt: AID,
    pub constructor: ConstructorDefinition,
    pub internal_id: usize // Each constructor in an ADT is given a unique internal_id
}

#[derive(Debug)]
pub enum ExprChildren<D> {
    Many(Vec<ExprWrapper<D>>),
    Match(Box<ExprWrapper<D>>, Vec<ExprWrapper<D>>),
    Zero
}

impl<D> ExprChildren<D> {
    pub fn all_children(&self) -> Vec<&ExprWrapper<D>> {
        match &self {
            ExprChildren::Many(s) => s.iter().collect(),
            ExprChildren::Match(expr, exprs) => iter::once(&**expr).chain(exprs.iter()).collect(),
            ExprChildren::Zero => vec![]
        }
    }
}

impl<D> ExprWrapper<D> {
    pub fn new(expr: Expression, data: D, children: ExprChildren<D>) -> Self {
        ExprWrapper { data, children, expr }
    }
}

impl<D, P> Deref for ChainedData<D, P> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

/*impl<'a> ScopeWrapper<'a> {
    fn validate_recursively(&self, program: &'a ScopedProgram) -> CompileResult {
        self.validate_correct_scope_children()?;

        match self.expr {
            Expression::FunctionCall(fid, args) => self.validate_as_function_call(program, fid, args)?,
            Expression::Variable(vid) => { self.get_var(vid)?; },
            Expression::Match(match_expression) => self.validate_as_match(program, match_expression)?,
            Expression::UTuple(_) => (), // Should already be validated by type inference
            Expression::Integer(_) => (),
        }

        for node in self.children.all_children() {
            node.validate_recursively(program)?;
        }

        Ok(())
    }

    fn validate_correct_scope_children(&self) -> CompileResult {
        match self.expr {
            Expression::UTuple(_) | Expression::FunctionCall(_, _) => {
                let ExprChildren::Many(_) = self.children else {
                    return Err(CompileError::InternalError);
                };
            },
            Expression::Integer(_) | Expression::Variable(_) => {
                let ExprChildren::Zero = self.children else {
                    return Err(CompileError::InternalError);
                };
            },
            Expression::Match(_) => {
                let ExprChildren::Match(_, _) = self.children else {
                    return Err(CompileError::InternalError);
                };
            },
        }

        Ok(())
    }

    fn validate_as_function_call(&self, program: &'a ScopedProgram, fid: &'a FID, args: &'a UTuple<Expression>) -> CompileResult {
        let expected_arg_type = &program.all_signatures.get(fid).ok_or_else(|| CompileError::UnknownFunction(fid))?.argument_type;

        if args.0.len() != expected_arg_type.0.len() {
            return Err(CompileError::WrongVariableCountInFunctionCall(&self.expr));
        }

        let arg_type: UTuple<Type> = UTuple(self.children.all_children().iter().map(|scope| scope.data.0.expect_tp().map(|x| x.clone())).collect::<Result<_, _>>()?);
        if &arg_type != expected_arg_type {
            return Err(CompileError::WrongArgumentType(fid.clone(), arg_type, expected_arg_type.clone()))
        }

        Ok(())
    }

    fn validate_as_match(&self, program: &'a ScopedProgram, match_expression: &'a MatchExpression) -> CompileResult {
        let ExprChildren::Match(expr_scope, case_scopes) = &self.children else { panic!() };
        get_scopes_same_type(case_scopes.iter()).ok_or_else(|| CompileError::MissmatchedTypes(self.expr))?;
    
        let mut has_wildcard = false;
        for case in &match_expression.cases {
            let mut case_is_wildcard = false;

            match &case.pattern {
                Pattern::UTuple(vars) => if vars.0.len() == 1 { case_is_wildcard = true },
                _ => ()
            }

            if has_wildcard {
                if !case_is_wildcard {
                    return Err(CompileError::MatchHasCaseAfterWildcard)
                } else {
                    return Err(CompileError::MatchHasMultipleWildcards)
                }
            }

            has_wildcard = has_wildcard || case_is_wildcard;
        }

        match &expr_scope.data.0 {
            ExpressionType::UTuple(_) => {
                if match_expression.cases.len() == 0 {
                    return Err(CompileError::NonExhaustiveMatch)
                } else if match_expression.cases.len() > 1 {
                    return Err(CompileError::MatchHasMultipleTupleCases)
                }

                match &match_expression.cases[0].pattern {
                    Pattern::Integer(_) |
                    Pattern::Constructor(_, _) => return Err(CompileError::InvalidPatternInMatchCase),
                    Pattern::UTuple(_) => {
                        // We know the tuple has the correct argument and variable types from type inference
                    },
                }
            },
            ExpressionType::Type(Type::Int) => {
                let mut used_ints = HashSet::new();
                for case in &match_expression.cases {
                    match &case.pattern {
                        Pattern::Integer(i) => {
                            if !used_ints.insert(i) {
                                return Err(CompileError::MultipleOccurencesOfIntInMatch);
                            }
                        },
                        Pattern::Constructor(_, _) => return Err(CompileError::InvalidPatternInMatchCase),
                        Pattern::UTuple(tup) => if tup.0.len() != 1 { return Err(CompileError::InvalidPatternInMatchCase) },
                    }
                }

                if !has_wildcard {
                    return Err(CompileError::NonExhaustiveMatch)
                }
            },
            ExpressionType::Type(Type::ADT(aid)) => {
                let mut used_constructors = HashSet::new();
                for case in &match_expression.cases {
                    match &case.pattern {
                        Pattern::Integer(_) => return Err(CompileError::InvalidPatternInMatchCase),
                        Pattern::UTuple(tup) => if tup.0.len() != 1 { return Err(CompileError::InvalidPatternInMatchCase) },
                        Pattern::Constructor(fid, vars) => {
                            let cons = program.get_constructor(fid)?;
                            if &cons.adt.id != aid {
                                return Err(CompileError::InvalidPatternInMatchCase);
                            }
        
                            if vars.0.len() != cons.constructor.arguments.0.len() {
                                return Err(CompileError::WrongVariableCountInMatchCase(case))
                            }
        
                            if !used_constructors.insert(fid) {
                                return Err(CompileError::MultipleOccurencesOfConstructorInMatch);
                            }
                        }
                    }
                }

                let adt = program.adts.get(aid).unwrap();
                if !has_wildcard && used_constructors.len() < adt.constructors.len() {
                    return Err(CompileError::NonExhaustiveMatch)
                }
            },
        };

        Ok(())
    }

    // Returns a list of variables used within all paths of execution
    // TODO: Check for reuse pairs / allocations
    fn recursively_validate_fip_expression(&self, program: &'a ScopedProgram) -> Result<HashSet<&VariableDefinition>, CompileError>
    {
        let mut used_vars = HashSet::new();

        /*if let Expression::FunctionCall(fid, vars) = &self.expr {
            if program.constructors.contains_key(fid) && vars.0.len() > 0 {
                return Err(CompileError::FIPFunctionAllocatesMemory)
            }
        }*/

        match &self.expr {
            Expression::FunctionCall(_, _) | Expression::UTuple(_) => {
                for child in self.children.all_children() {
                    let child_used_vars = child.recursively_validate_fip_expression(program)?;
                    if let Some(double_var) = used_vars.intersection(&child_used_vars).next() {
                        return Err(CompileError::FIPFunctionHasMultipleUsedVar(double_var.id.clone()))
                    }

                    used_vars.extend(child_used_vars);
                }
            },
            Expression::Integer(_) => (),
            Expression::Variable(vid) => { used_vars.insert(self.data.1.get(vid).unwrap()); },
            Expression::Match(expr) => {
                let ExprChildren::Match(e1, case_scopes) = &self.children else { panic!() };

                used_vars.extend(e1.recursively_validate_fip_expression(program)?);

                let mut cases_used_vars = None;

                for (case, child) in expr.cases.iter().zip(case_scopes) {
                    let mut child_used_vars = child.recursively_validate_fip_expression(program)?;

                    match &case.pattern {
                        Pattern::Integer(_) => (),
                        Pattern::Constructor(_, vars) | Pattern::UTuple(vars) => {
                            for vid in &vars.0 {
                                if !child_used_vars.remove(&**child.data.1.get(vid).unwrap()) {
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
    }

    fn get_var(&'a self, vid: &'a VID) -> Result<&'a Rc<VariableDefinition>, CompileError<'a>> {
        self.data.1.get(vid).ok_or_else(|| CompileError::UnknownVariable(vid))
    }
}*/


// ==== PRETTY PRINT CODE ====

pub fn write_indent(f: &mut Formatter, indent: usize) -> std::fmt::Result {
    write!(f, "{}", "    ".repeat(indent))
}

impl<D: Display> Display for WrappedProgram<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (_, def) in &self.adts {
            writeln!(f, "{def}")?;
        }

        Ok(())
    }
}

impl Display for ADTDefinition {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "enum {} = \n", self.id)?;
        write_indent(f, 1)?;

        write!(f, "{}", self.constructors[0])?;
        for cons in self.constructors.iter().skip(1) {
            writeln!(f, ",")?;
            write_indent(f, 1)?;
            write!(f, "{cons}")?;
        }

        write!(f, ";")?;

        Ok(())
    }
}

impl Display for ConstructorDefinition {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.id, self.arguments)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::ADT(id) => write!(f, "{}", id)
        }
    }
}

impl Display for FunctionSignature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_fip { write!(f, "fip")?; }

        write!(f, "{}:{}", self.argument_type, self.result_type)
    }
}

impl<T : Display> Display for UTuple<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_implicit_utuple(f, &self.0, ", ", |f, t| write!(f, "{t}"))
    }
}

pub fn write_implicit_utuple<T>(
    f: &mut Formatter, 
    items: &Vec<T>,
    separator: &str,
    write: impl Fn(&mut Formatter, &T) -> std::fmt::Result
) -> std::fmt::Result
{
    if items.len() == 0 { Ok(()) }
    else if items.len() == 1 { 
        write(f, &items[0]) 
    } else {
        write!(f, "(")?;
        write_separated_list(f, items.iter(), separator, write)?;
        write!(f, ")")
    }
}


pub fn write_separated_list<T>(
    f: &mut Formatter, 
    iter: impl Iterator<Item = T>, 
    separator: &str,
    write: impl Fn(&mut Formatter, T) -> std::fmt::Result
) -> std::fmt::Result 
{
    let mut iter = iter.peekable();
    while let Some(item) = iter.next() {
        write(f, item)?;

        if iter.peek().is_some() { write!(f, "{separator}")?; }
    }

    Ok(())
}

impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Pattern::Integer(x) => write!(f, "{x}"),
            Pattern::Constructor(fid, vars) => {
                write!(f, "{} ", fid)?;
                write_implicit_utuple(f, &vars.0, ", ", |f, vid| write!(f, "{vid}"))
            },
            Pattern::UTuple(utuple) => {
                write_implicit_utuple(f, &utuple.0, ", ", |f, vid| { write!(f, "{vid}") })
            },
        }
    }
}