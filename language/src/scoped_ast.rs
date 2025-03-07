use std::{cell::RefMut, collections::{HashMap, HashSet}, fmt::{Display, Formatter}, path::MAIN_SEPARATOR, rc::Rc, sync::{atomic::AtomicUsize, Arc, LazyLock}};
use crate::{ast::{write_implicit_utuple, write_indent, write_separated_list, ADTDefinition, ConstructorDefinition, ConstructorSignature, Definition, Expression, FunctionDefinition, FunctionSignature, Program, Type, UTuple, AID, FID, VID}, error::{CompileError, CompileResult}};

#[derive(Debug)]
pub struct ConstructorReference<'a> {
    pub adt: &'a ADTDefinition,
    pub constructor: &'a ConstructorDefinition,
    pub internal_id: usize // Each constructor in an ADT is given a unique internal_id
}

#[derive(Debug)]
pub struct ScopedFunction<'a> {
    pub def: &'a FunctionDefinition,
    pub body: ScopedExpressionNode<'a>
}

#[derive(Clone, Debug)]
pub struct VariableDefinition {
    id: VID,
    tp: Type,
    internal_id: usize // Each definition is given a definitively different internal_id
}

type Scope<'a> = HashMap<VID, Rc<VariableDefinition>>;

#[derive(Clone, Debug)]
pub struct ScopedExpressionNode<'a> {
    pub expr: &'a Expression,
    pub children: ScopeChildren<'a>,
    pub scope: Scope<'a>,
    pub tp: ExpressionType
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ExpressionType {
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

#[derive(Debug, Clone)]
pub enum ScopeChildren<'a> {
    Many(Vec<ScopedExpressionNode<'a>>),
    Two(Box<ScopedExpressionNode<'a>>, Box<ScopedExpressionNode<'a>>),
    Zero
}

impl<'a> ScopeChildren<'a> {
    fn scopes(&self) -> Vec<&ScopedExpressionNode<'a>> {
        match &self {
            ScopeChildren::Many(s) => s.iter().collect(),
            ScopeChildren::Two(s1, s2) => vec![s1, s2],
            ScopeChildren::Zero => vec![]
        }
    }

    fn get_same_type(&self) -> Option<ExpressionType> {
        let mut iter = self.scopes().into_iter();
        let tp = iter.next()?.tp.clone();

        for x in iter {
            if x.tp != tp { return None; }
        }

        Some(tp)
    }
}

#[derive(Debug)]
pub struct ScopedProgram<'a> {
    adts: HashMap<AID, &'a ADTDefinition>,
    pub constructors: HashMap<FID, ConstructorReference<'a>>,
    pub functions: HashMap<FID, ScopedFunction<'a>>,
    all_signatures: HashMap<FID, FunctionSignature>,
    pub program: &'a Program
}

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
        let mut zero_argument_constructor_variables = Vec::new();
        for def in &program.0 {
            match def {
                Definition::ADTDefinition(def) => {
                    constructor_signatures.extend(def.constructors.iter().map(|cons| (cons.id.clone(), &cons.arguments)));
                    zero_argument_constructor_variables.extend(
                        def.constructors.iter()
                        .filter(|c| c.arguments.0.len() == 0)
                        .map(|c| VariableDefinition { id: c.id.clone(), tp: Type::ADT(def.id.clone()), internal_id: get_new_internal_id() }) 
                    );

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

        let mut default_scope = HashMap::new();
        default_scope.extend(zero_argument_constructor_variables.into_iter().map(|c| (c.id.clone(), Rc::new(c))));

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
    
                    let function_variables = def.variables.0.iter().zip(def.signature.argument_type.0.iter()).map(
                        |(vid, tp)| {
                            VariableDefinition { id: vid.clone(), tp: tp.clone(), internal_id: get_new_internal_id() }
                        }
                    ).collect::<Vec<_>>();
    
                    functions.insert(
                        def.id.clone(), 
                        ScopedFunction { 
                            def, 
                            body: scope_expression(&def.body, &default_scope, function_variables, &all_function_signatures, &constructor_signatures)?
                        }
                    );
                }
            }
        }

        Ok(ScopedProgram {
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
            func.body.validate(self)?;
            
            let return_type = match &func.body.tp {
                ExpressionType::UTuple(utuple) => utuple.clone(),
                ExpressionType::Type(tp) => UTuple(vec![tp.clone()]),
            };

            if return_type != func.def.signature.result_type {
                return Err(CompileError::WrongReturnType)
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

impl<'a> ScopedExpressionNode<'a> {
    fn validate(&self, program: &'a ScopedProgram) -> CompileResult {
        let expected_scope_children_count: Option<usize> = match self.expr {
            Expression::UTuple(_) | Expression::FunctionCall(_, _) | Expression::Match(_) => None,
            Expression::Integer(_) | Expression::Variable(_) => Some(0),
            Expression::LetEqualIn(_, _, _) => Some(2),
        };

        if  expected_scope_children_count.is_some_and(|x| x != self.children.scopes().len()) {
            panic!("Unexpected number of scope children")
        }

        match self.expr {
            Expression::FunctionCall(fid, args) => {
                let expected_arg_type = &program.all_signatures.get(fid).ok_or_else(|| CompileError::UnknownFunction(fid))?.argument_type;

                if args.0.len() != expected_arg_type.0.len() {
                    return Err(CompileError::WrongVariableCountInFunctionCall(&self.expr));
                }

                let arg_type: UTuple<Type> = UTuple(self.children.scopes().iter().map(|scope| scope.tp.expect_tp().map(|x| x.clone())).collect::<Result<_, _>>()?);
                if &arg_type != expected_arg_type {
                    return Err(CompileError::WrongArgumentType(fid.clone(), arg_type, expected_arg_type.clone()))
                }
            },
            Expression::Variable(vid) => { self.get_var(vid)?; },
            Expression::Match(match_expression) => {
                let var_def = self.get_var(&match_expression.variable)?;
                let aid: &AID = match &var_def.tp {
                    Type::Int => return Err(CompileError::WrongVariableTypeInMatch),
                    Type::ADT(aid) => &aid,
                };

                self.children.get_same_type().ok_or_else(|| CompileError::MissmatchedTypes(self.expr))?;

                let mut used_constructors = HashSet::new();
                for case in &match_expression.cases {
                    let cons = program.get_constructor(&case.cons_id)?;
                    if &cons.adt.id != aid {
                        return Err(CompileError::InvalidConstructorInMatchCase);
                    }

                    if case.vars.0.len() != cons.constructor.arguments.0.len() {
                        return Err(CompileError::WrongVariableCountInMatchCase(case))
                    }

                    if !used_constructors.insert(&case.cons_id) {
                        return Err(CompileError::MultipleOccurencesOfConstructorInMatch);
                    }
                }

                let adt = program.adts.get(aid).unwrap();
                if used_constructors.len() < adt.constructors.len() {
                    return Err(CompileError::NonExhaustiveMatch)
                }
            },
            Expression::LetEqualIn(vars, e1, e2) => {
                let fid = match &**e1 {
                    Expression::FunctionCall(fid, _) => fid,
                    _ => return Err(CompileError::LetHasNoFunctionCall(&self.expr))
                };

                let signature = program.all_signatures.get(fid).ok_or_else(|| CompileError::UnknownFunction(fid))?;
                if signature.result_type.0.len() != vars.0.len() {
                    return Err(CompileError::WrongVariableCountInLetStatement(&self.expr));
                }
            },
            Expression::UTuple(_) => (), // Should already be validated by type inference
            Expression::Integer(_) => (),
        }

        for node in self.children.scopes() {
            node.validate(program)?;
        }

        Ok(())
    }

    fn get_var(&'a self, vid: &'a VID) -> Result<&'a Rc<VariableDefinition>, CompileError<'a>> {
        self.scope.get(vid).ok_or_else(|| CompileError::UnknownVariable(vid))
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
    scope: &Scope<'a>, 
    new_vars: Vec<VariableDefinition>, 
    function_signatures: &HashMap<FID, FunctionSignature>,
    constructor_signatures: &HashMap<FID, &'b ConstructorSignature>,
) -> Result<ScopedExpressionNode<'a>, CompileError<'a>> 
{
    let mut scope = scope.clone();
    for var in new_vars {
        scope.insert(var.id.clone(), Rc::new(var));
    }

    let (children, tp) = match expr {
        Expression::UTuple(tup) => {
            let children = ScopeChildren::Many(tup.0.iter().map(|expr| scope_expression(expr, &scope, vec![], function_signatures, constructor_signatures)).collect::<Result<_, _>>()?);
            let tp = ExpressionType::UTuple(UTuple(
                children.scopes().iter().map(|s| s.tp.tp().ok_or_else(|| CompileError::UnexpectedUTuple).map(|t| t.clone())).collect::<Result<_, _>>()?
            ));
            (children, tp)
        },
        Expression::FunctionCall(fid, tup) => {
            let children = ScopeChildren::Many(tup.0.iter().map(|expr| scope_expression(expr, &scope, vec![], function_signatures, constructor_signatures)).collect::<Result<_, _>>()?);
            let return_type = &function_signatures.get(fid).ok_or_else(|| CompileError::UnknownFunction(fid))?.result_type;
            let tp = if return_type.0.len() == 1 { ExpressionType::Type(return_type.0[0].clone()) } else { ExpressionType::UTuple(return_type.clone()) };
            (children, tp)
        },
        Expression::Integer(_) => (ScopeChildren::Zero, ExpressionType::Type(Type::Int)),
        Expression::Variable(var) => 
            (
                ScopeChildren::Zero, 
                ExpressionType::Type(scope.get(var).ok_or_else(|| CompileError::UnknownVariable(var))?.tp.clone())
            ),
        Expression::LetEqualIn(vars, e1, e2) => {
            let mut new_vars = vec![];
            
            let signature = match &**e1 {
                Expression::FunctionCall(fid, _) => function_signatures.get(fid).ok_or(CompileError::UnknownFunction(fid))?,
                _ => return Err(CompileError::LetHasNoFunctionCall(expr))
            };

            if vars.0.len() != signature.result_type.0.len() {
                return Err(CompileError::WrongVariableCountInLetStatement(&expr))
            }

            for (vid, tp) in vars.0.iter().zip(signature.result_type.0.iter()) {
                new_vars.push(VariableDefinition { id: vid.clone(), tp: tp.clone(), internal_id: get_new_internal_id() });
            }

            let e1 = scope_expression(e1, &scope, vec![], function_signatures, constructor_signatures)?;
            let e2 = scope_expression(e2, &scope, new_vars, function_signatures, constructor_signatures)?;

            let tp = e2.tp.clone();
            let children = ScopeChildren::Two(Box::new(e1), Box::new(e2));


            (children, tp)
        },
        Expression::Match(match_expr) => {
            let children = ScopeChildren::Many(
                match_expr.cases.iter().map(|case| {
                    let cons_sig: &ConstructorSignature = constructor_signatures.get(&case.cons_id).ok_or(CompileError::UnknownConstructor(&case.cons_id))?;
                    if cons_sig.0.len() != case.vars.0.len() {
                        panic!("Wrong number of arguments in match statement of case {}", case.cons_id);
                    }

                    scope_expression(
                        &case.body,
                        &scope, 
                        case.vars.0.iter().zip(cons_sig.0.iter()).map(|(new_vid, tp)| {
                            VariableDefinition {
                                id: new_vid.clone(),
                                tp: tp.clone(),
                                internal_id: get_new_internal_id()
                            }
                        }).collect(),
                        function_signatures,
                        constructor_signatures
                    )
                }).collect::<Result<_, _>>()?
            );

            let tp = children.get_same_type().ok_or_else(|| CompileError::MissmatchedTypes(expr))?;
            (children, tp)
        }
    };

    Ok(ScopedExpressionNode {
        expr,
        children,
        scope,
        tp
    })
}

impl PartialEq for VariableDefinition {
    fn eq(&self, other: &Self) -> bool {
       self.internal_id == other.internal_id
    }
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

impl Display for ScopedFunction<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.def.signature)?;
        write!(f, "{} ", self.def.id)?;
        write_implicit_utuple(f, &self.def.variables.0, ", ", |f, x| write!(f, "{x}"))?;
        writeln!(f, " = ")?;

        write_scoped_expression_node(f, &self.body,1)?;

        Ok(())
    }
}

fn write_scoped_expression_node<'a>(f: &mut Formatter<'_>, node: &'a ScopedExpressionNode<'a>, indent: usize) -> std::fmt::Result {
    write_indent(f, indent)?;
    write!(f, "// type = {}, scope = ", node.tp)?;
    write_scope(f, &node.scope, |f, x| write!(f, "{}|{}[{}]", x.id, x.internal_id, x.tp))?;
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
            for child in node.children.scopes() {
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