use std::{collections::{HashMap, HashSet}, fmt::{Display, Formatter}, hash::Hash, iter, rc::Rc, sync::atomic::AtomicUsize};
use crate::{ast::{write_implicit_utuple, write_indent, write_separated_list, ADTDefinition, ConstructorDefinition, ConstructorSignature, Definition, Expression, FunctionDefinition, FunctionSignature, MatchExpression, Pattern, Program, Type, UTuple, AID, FID, VID}, error::{CompileError, CompileResult}};

#[derive(Debug)]
pub struct ConstructorReference<'a> {
    pub adt: &'a ADTDefinition,
    pub constructor: &'a ConstructorDefinition,
    pub internal_id: usize // Each constructor in an ADT is given a unique internal_id
}

#[derive(Debug)]
pub struct WrappedFunction<'a, D, C> {
    pub def: &'a FunctionDefinition,
    pub body: ASTWrapper<'a, D, C>,
}

#[derive(Clone, Debug, Eq)]
pub struct VariableDefinition {
    id: VID,
    tp: Type,
    internal_id: usize // Each definition is given a definitively different internal_id
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

type Scope = HashMap<VID, Rc<VariableDefinition>>;
type ScopeWrapper<'a> = ASTWrapper<'a, (ExpressionType, Scope), Expression>;

#[derive(Debug)]
pub struct ASTWrapper<'a, D, C> {
    pub expr: &'a Expression,
    pub content: &'a C,
    pub children: WrapperChildren<'a, D, C>,
    pub data: D
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

#[derive(Debug)]
pub enum WrapperChildren<'a, D, C> {
    Many(Vec<ASTWrapper<'a, D, C>>),
    Match(Box<ASTWrapper<'a, D, C>>, Vec<ASTWrapper<'a, D, C>>),
    Zero
}

impl<'a, D, C> WrapperChildren<'a, D, C> {
    fn all_children(&self) -> Vec<&ASTWrapper<'a, D, C>> {
        match &self {
            WrapperChildren::Many(s) => s.iter().collect(),
            WrapperChildren::Match(expr, exprs) => iter::once(&**expr).chain(exprs.iter()).collect(),
            WrapperChildren::Zero => vec![]
        }
    }
}

fn get_scopes_same_type<'a, 'b: 'a>(mut iter: impl Iterator<Item = &'a ScopeWrapper<'b>>) -> Option<ExpressionType> {
    let tp = iter.next()?.data.0.clone();

    for x in iter {
        if x.data.0 != tp { return None; }
    }

    Some(tp)
}

#[derive(Debug)]
pub struct WrappedProgram<'a, D, C> {
    adts: HashMap<AID, &'a ADTDefinition>,
    pub constructors: HashMap<FID, ConstructorReference<'a>>,
    pub functions: HashMap<FID, WrappedFunction<'a, D, C>>,
    all_signatures: HashMap<FID, FunctionSignature>,
    pub program: &'a Program
}

pub(crate) type ScopedProgram<'a> = WrappedProgram<'a, (ExpressionType, Scope), Expression>;

impl<'a> ScopedProgram<'a> {
    // Creates a new program with scope information
    // Performs minimum required validation, such as no top level symbol collisions
    pub fn new(program: &'a Program) -> Result<ScopedProgram<'a>, CompileError<'a>> {
        program.validate_top_level_ids();

        let mut all_function_signatures: HashMap<FID, FunctionSignature> = HashMap::new();
        for op in "+-/*".chars() {
            all_function_signatures.insert(op.to_string(), FunctionSignature { 
                argument_type: UTuple(vec![Type::Int, Type::Int]),
                result_type: UTuple(vec![Type::Int]),
                is_fip: true
            });
        }

        let mut constructor_signatures: HashMap<FID, &ConstructorSignature> = HashMap::new();
        for def in &program.0 {
            match def {
                Definition::ADTDefinition(def) => {
                    constructor_signatures.extend(def.constructors.iter().map(|cons| (cons.id.clone(), &cons.arguments)));

                    all_function_signatures.extend(def.constructors.iter().map(
                        |cons| {
                            (cons.id.clone(),
                                FunctionSignature {
                                    argument_type: cons.arguments.clone(),
                                    result_type: UTuple(vec! [Type::ADT(def.id.clone())]),
                                    is_fip: true
                                }
                            )
                        }
                    ));
                },
                Definition::FunctionDefinition(def) => {
                    all_function_signatures.insert(def.id.clone(), def.signature.clone());
                }
            }
        }

        reset_internal_id_counter();

        let mut adts = HashMap::new();
        let mut constructors = HashMap::new();
        let mut functions = HashMap::new();
        for def in &program.0 {
            match def {
                Definition::ADTDefinition(def) => {
                    adts.insert(def.id.clone(), def);
    
                    for (internal_id, cons) in def.constructors.iter().enumerate() {    
                        constructors.insert(cons.id.clone(), ConstructorReference { adt: &def, constructor: &cons, internal_id });
                    }
                },
                Definition::FunctionDefinition(def) => {
                    if def.variables.0.len() != def.signature.argument_type.0.len() {
                        return Err(CompileError::InconsistentVariableCountInFunctionDefinition(def))
                    }
    
                    let base_scope = def.variables.0.iter().zip(def.signature.argument_type.0.iter()).map(
                        |(vid, tp)| {
                            (vid.clone(), Rc::new(VariableDefinition { id: vid.clone(), tp: tp.clone(), internal_id: get_new_internal_id() }))
                        }
                    ).collect::<Scope>();
    
                    functions.insert(
                        def.id.clone(), 
                        WrappedFunction { 
                            def,
                            body: scope_expression(&def.body, base_scope, &all_function_signatures, &constructor_signatures)?
                        }
                    );
                }
            }
        }

        Ok(WrappedProgram {
            adts,
            constructors,
            functions,
            program,
            all_signatures: all_function_signatures
        })
    }

    pub fn validate(&self) -> CompileResult {
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
    }

    pub fn get_constructor(&self, fid: &'a FID) -> Result<&ConstructorReference<'a>, CompileError<'a>> {
        self.constructors.get(fid).ok_or_else(|| CompileError::UnknownConstructor(fid))
    }
}

impl Type {
    fn validate_in(&self, program: &ScopedProgram) -> CompileResult {
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
    fn validate_in(&self, program: &ScopedProgram) -> CompileResult {
        for tp in &self.0 { tp.validate_in(program)?; }
        Ok(())
    }
}

impl FunctionSignature {
    fn validate_in(&self, program: &ScopedProgram) -> CompileResult {
        self.argument_type.validate_in(program)?;
        self.result_type.validate_in(program)
    }
}

impl<'a> ScopeWrapper<'a> {
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
                let WrapperChildren::Many(_) = self.children else {
                    return Err(CompileError::InternalError);
                };
            },
            Expression::Integer(_) | Expression::Variable(_) => {
                let WrapperChildren::Zero = self.children else {
                    return Err(CompileError::InternalError);
                };
            },
            Expression::Match(_) => {
                let WrapperChildren::Match(_, _) = self.children else {
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
        let WrapperChildren::Match(expr_scope, case_scopes) = &self.children else { panic!() };
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
                let WrapperChildren::Match(e1, case_scopes) = &self.children else { panic!() };

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
}

static COUNTER: AtomicUsize = AtomicUsize::new(0);
fn reset_internal_id_counter() {
    COUNTER.store(0, std::sync::atomic::Ordering::Relaxed);
}

fn get_new_internal_id() -> usize {
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

// Creates a ScopeExpressionNode recursively for the expression
// Each node contains a mapping from VID to VariableDefinition and the resulting type of the expression
// A variable definition contains type information 
// Checks that each case in match has correct number of arguments for the constructor
// Does type inference on variables and expression, with minimum type checking
fn scope_expression<'a, 'b>(
    expr: &'a Expression, 
    scope: Scope,
    function_signatures: &HashMap<FID, FunctionSignature>,
    constructor_signatures: &HashMap<FID, &'b ConstructorSignature>,
) -> Result<ScopeWrapper<'a>, CompileError<'a>> 
{
    let (children, tp) = match expr {
        Expression::UTuple(tup) => {
            let children = WrapperChildren::Many(tup.0.iter().map(|expr| scope_expression(expr, scope.clone(), function_signatures, constructor_signatures)).collect::<Result<_, _>>()?);
            let tp = ExpressionType::UTuple(UTuple(
                children.all_children().iter().map(|s| s.data.0.tp().ok_or_else(|| CompileError::UnexpectedUTuple).map(|t| t.clone())).collect::<Result<_, _>>()?
            ));
            (children, tp)
        },
        Expression::FunctionCall(fid, tup) => {
            let children = WrapperChildren::Many(tup.0.iter().map(|expr| scope_expression(expr, scope.clone(), function_signatures, constructor_signatures)).collect::<Result<_, _>>()?);
            let return_type = &function_signatures.get(fid).ok_or_else(|| CompileError::UnknownFunction(fid))?.result_type;
            let tp = if return_type.0.len() == 1 { ExpressionType::Type(return_type.0[0].clone()) } else { ExpressionType::UTuple(return_type.clone()) };
            (children, tp)
        },
        Expression::Integer(_) => (WrapperChildren::Zero, ExpressionType::Type(Type::Int)),
        Expression::Variable(var) => 
        (
            WrapperChildren::Zero, 
            ExpressionType::Type(scope.get(var).ok_or_else(|| CompileError::UnknownVariable(var))?.tp.clone())
        ),
        Expression::Match(match_expr) => {
            let match_on_scope = scope_expression(&match_expr.expr, scope.clone(), function_signatures, constructor_signatures)?;
            let match_on_type = match_on_scope.data.0.clone();

            let case_scopes: Vec<ScopeWrapper<'_>> = match_expr.cases.iter().map(|case| {
                match &case.pattern {
                    Pattern::Integer(_) => scope_expression(&case.body, scope.clone(), function_signatures, constructor_signatures),
                    Pattern::UTuple(vars) => {
                        let types = match &match_on_type {
                            ExpressionType::UTuple(tup) => tup.clone(),
                            ExpressionType::Type(tp) => UTuple(vec![tp.clone()]),
                        };

                        scope_expression(
                            &case.body,
                            extended_scope(
                                &scope, 
                                vars.0.iter().zip(types.0.iter())
                                .map(|(new_vid, tp)| {
                                    VariableDefinition {
                                        id: new_vid.clone(),
                                        tp: tp.clone(),
                                        internal_id: get_new_internal_id()
                                    }
                                })
                            ),
                            function_signatures,
                            constructor_signatures
                        )
                    },
                    Pattern::Constructor(fid, vars) => {
                        let cons_sig: &ConstructorSignature = constructor_signatures.get(fid).ok_or(CompileError::UnknownConstructor(fid))?;
                        if cons_sig.0.len() != vars.0.len() {
                            panic!("Wrong number of arguments in match statement of case {}", fid);
                        }

                        scope_expression(
                            &case.body,
                            extended_scope(
                                &scope, 
                                vars.0.iter().zip(cons_sig.0.iter())
                                .map(|(new_vid, tp)| {
                                    VariableDefinition {
                                        id: new_vid.clone(),
                                        tp: tp.clone(),
                                        internal_id: get_new_internal_id()
                                    }
                                })
                            ),
                            function_signatures,
                            constructor_signatures
                        )
                    },
                }
            }).collect::<Result<_, _>>()?;

            let tp = get_scopes_same_type(case_scopes.iter()).ok_or_else(|| CompileError::MissmatchedTypes(expr))?;

            let children = WrapperChildren::Match(
                Box::new(match_on_scope),
                case_scopes
            );

            (children, tp)
        }
    };

    Ok(ASTWrapper {
        expr,
        content: expr,
        children,
        data: (tp, scope)
    })
}

fn extended_scope(base: &Scope, new_vars: impl Iterator<Item = VariableDefinition>) -> Scope {
    let mut new_scope = base.clone();
    new_scope.extend(new_vars.map(|x| (x.id.clone(), Rc::new(x))));
    new_scope
}

// ==== PRETTY PRINT CODE ====

pub fn write_scope<T>(
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
}