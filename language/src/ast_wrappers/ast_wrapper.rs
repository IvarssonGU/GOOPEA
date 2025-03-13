use std::{collections::{HashMap, HashSet}, fmt::{Display, Formatter}, hash::Hash, iter, rc::Rc, sync::atomic::AtomicUsize};
use crate::{ast::{write_implicit_utuple, write_indent, write_separated_list, ADTDefinition, ConstructorDefinition, ConstructorSignature, Definition, Expression, FunctionDefinition, FunctionSignature, MatchExpression, Pattern, Program, Type, UTuple, AID, FID, VID}, error::{CompileError, CompileResult}};

#[derive(Debug)]
pub struct WrappedProgram<'a, D, C, P> {
    pub adts: HashMap<AID, &'a ADTDefinition>,
    pub constructors: HashMap<FID, ConstructorReference<'a>>,
    pub functions: HashMap<FID, WrappedFunction<'a, D, C>>,
    pub all_signatures: HashMap<FID, FunctionSignature>,
    pub program: &'a P
}

#[derive(Debug)]
pub struct WrappedFunction<'a, D, C> {
    pub def: &'a FunctionDefinition,
    pub body: ExprWrapper<'a, D, C>,
}

#[derive(Debug)]
pub struct ExprWrapper<'a, D, C> {
    pub expr: &'a Expression,
    pub content: &'a C,
    pub children: ExprChildren<'a, D, C>,
    pub data: D
}

#[derive(Debug, Clone)]
pub struct ConstructorReference<'a> {
    pub adt: &'a ADTDefinition,
    pub constructor: &'a ConstructorDefinition,
    pub internal_id: usize // Each constructor in an ADT is given a unique internal_id
}

#[derive(Debug)]
pub enum ExprChildren<'a, D, C> {
    Many(Vec<ExprWrapper<'a, D, C>>),
    Match(Box<ExprWrapper<'a, D, C>>, Vec<ExprWrapper<'a, D, C>>),
    Zero
}

impl<'a, D, C> ExprChildren<'a, D, C> {
    pub fn all_children(&self) -> Vec<&ExprWrapper<'a, D, C>> {
        match &self {
            ExprChildren::Many(s) => s.iter().collect(),
            ExprChildren::Match(expr, exprs) => iter::once(&**expr).chain(exprs.iter()).collect(),
            ExprChildren::Zero => vec![]
        }
    }
}

impl<'a, 'b: 'a, D, C, P> WrappedProgram<'a, D, C, P> {
    pub fn wrap<ND, NC>(&'a self, wrapper_func: impl Fn(&WrappedFunction<'a, D, C>) -> Result<WrappedFunction<'b, ND, NC>, CompileError<'b>>) -> Result<WrappedProgram<'b, ND, NC, Self>, CompileError<'b>> {
        Ok(WrappedProgram {
            adts: self.adts.clone(),
            constructors: self.constructors.clone(),
            functions: self.functions.iter().map(|(fid, func)| wrapper_func(func).map(|func| (fid.clone(), func))).collect::<Result<HashMap<_, _>, _>>()?,
            all_signatures: self.all_signatures.clone(),
            program: self
        })
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

/*pub fn write_scope<T>(
    f: &mut std::fmt::Formatter<'_>, 
    scope: &HashMap<String, T>, 
    write: impl Fn(&mut Formatter, &T) -> std::fmt::Result
) -> std::fmt::Result 
{
    write!(f, "{{")?;

    write_separated_list(f, scope.iter(), ", ", |f, (_, val)| { write(f, val) })?;

    write!(f, "}}")
}

impl Display for ExpressionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            ExpressionType::UTuple(utuple) => write_implicit_utuple(f, &utuple.0, ", ", |f, x| write!(f, "{x}")),
            ExpressionType::Type(tp) => write!(f, "{tp}"),
        }
    }
}

impl Display for ScopedProgram<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "// === Scoped Program ===")?;

        write!(f, "// ADTs: ")?;
        write_scope(f, &self.adts, |f, x| write!(f, "{}", x.id))?;
        writeln!(f)?;

        write!(f, "// Constructors: ")?;
        write_scope(f, &self.constructors, |f, x| write!(f, "{}/{}", x.adt.id, x.constructor.id))?;
        writeln!(f)?;

        write!(f, "// Functions: ")?;
        write_scope(f, &self.functions, |f, x| write!(f, "{}[{}]", x.def.id, x.def.signature))?;
        writeln!(f)?;

        writeln!(f)?;
        writeln!(f, "// === ADT Definitions ===")?;
        for (_, adt) in &self.adts {
            writeln!(f, "{}\n", adt)?;
        }

        writeln!(f, "// === Scoped Functions ===")?;
        for (_, func) in &self.functions {
            writeln!(f, "{}\n", func)?;
        }


        Ok(())
    }
}

impl<'a> Display for WrappedFunction<'a, (ExpressionType, Scope), Expression> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.def.signature)?;
        write!(f, "{} ", self.def.id)?;
        write_implicit_utuple(f, &self.def.variables.0, ", ", |f, x| write!(f, "{x}"))?;
        writeln!(f, " = ")?;

        write_scoped_expression_node(f, &self.body,1)?;

        Ok(())
    }
}

fn write_scoped_expression_node<'a>(f: &mut Formatter<'_>, node: &'a ScopeWrapper<'a>, indent: usize) -> std::fmt::Result {
    write_indent(f, indent)?;
    write!(f, "// type = {}, scope = ", node.data.0)?;
    write_scope(f, &node.data.1, |f, x| write!(f, "{}|{}[{}]", x.id, x.internal_id, x.tp))?;
    writeln!(f)?;

    match &node.expr {
        /*Expression::UTuple(_) => {
            write_implicit_utuple(f, &node.children.scopes(), ",\n", move |f, x| {
                write_indent(f, indent)?;
                write_scoped_expression_node(f, x, previous_scope, indent+1)
            })?;

            writeln!(f)?;
        },*/
        //Expression::FunctionCall(_, utuple) => todo!(),
        //Expression::Integer(_) => todo!(),
        //Expression::Variable(_) => todo!(),
        //Expression::Match(match_expression) => todo!(),
        //Expression::Operation(operator, expression, expression1) => todo!(),
        //Expression::LetEqualIn(utuple, expression, expression1) => todo!(),
        _ => {
            for child in node.children.all_children() {
                write_scoped_expression_node(f, child, indent+1)?;
            }
        }
    }

    Ok(())
}

fn scopes_are_equal(s1: &Scope, s2: &Scope) -> bool {
    for (id, def) in s1 {
        let Some(other_def) = s2.get(id) else { return false; };
        if def != other_def { return false; }
    }

    for (id, def) in s2 {
        let Some(other_def) = s1.get(id) else { return false; };
        if def != other_def { return false; }
    }

    true
}*/