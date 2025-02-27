use std::{cell::RefMut, collections::{HashMap, HashSet}, fmt::{Display, Formatter}, rc::Rc, sync::{atomic::AtomicUsize, Arc, LazyLock}};
use crate::ast::{write_implicit_utuple, write_indent, write_separated_list, ADTDefinition, ConstructorDefinition, ConstructorSignature, Definition, Expression, FunctionDefinition, FunctionSignature, Program, Type, UTuple, AID, FID, VID};

#[derive(Debug)]
pub struct ConstructorReference<'a> {
    adt: &'a ADTDefinition,
    constructor: &'a ConstructorDefinition
}

#[derive(Debug)]
pub struct ScopedFunction<'a> {
    def: &'a FunctionDefinition,
    body: ScopedExpressionNode<'a>
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
    expr: &'a Expression,
    children: ScopeChildren<'a>,
    scope: Scope<'a>,
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
}

#[derive(Debug)]
pub struct ScopedProgram<'a> {
    adts: HashMap<AID, &'a ADTDefinition>,
    constructors: HashMap<FID, ConstructorReference<'a>>,
    functions: HashMap<FID, ScopedFunction<'a>>,
    program: &'a Program
}

impl<'a> ScopedProgram<'a> {
    // Creates a new program with scope information
    // Performs minimum required validation, such as no top level symbol collisions
    pub fn new(program: &'a Program) -> ScopedProgram<'a> {
        program.validate_top_level_ids();

        let mut function_signatures: HashMap<FID, &FunctionSignature> = HashMap::new();
        let mut constructor_signatures: HashMap<FID, &ConstructorSignature> = HashMap::new();
        for def in &program.0 {
            match def {
                Definition::ADTDefinition(def) =>
                    constructor_signatures.extend(def.constructors.iter().map(|cons| (cons.id.clone(), &cons.arguments))),
                Definition::FunctionDefinition(def) => {
                    function_signatures.insert(def.id.clone(), &def.signature);
                }
            }
        }

        let mut adts = HashMap::new();
        let mut constructors = HashMap::new();
        let mut functions = HashMap::new();
        for def in &program.0 {
            match def {
                Definition::ADTDefinition(def) => {
                    adts.insert(def.id.clone(), def);
    
                    for cons in &def.constructors {    
                        constructors.insert(cons.id.clone(), ConstructorReference { adt: &def, constructor: &cons });
                    }
                },
                Definition::FunctionDefinition(def) => {
                    if def.variables.0.len() != def.signature.argument_type.0.len() {
                        panic!("Missmatched argument count in signature vs definition of function {}", def.id);
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
                            body: scope_expression(&def.body, &HashMap::new(), function_variables, &function_signatures, &constructor_signatures) 
                        }
                    );
                }
            }
        }

        ScopedProgram {
            adts,
            constructors,
            functions,
            program
        }
    }

    // Checks so that ADT constructors use valid types
    pub fn validate_adt_types(&self) {
        for (fid, cons) in &self.constructors {
            if !cons.constructor.arguments.is_valid_in(self) {
                panic!("Constructor {} has invalid type", fid);
            }
        }
    }
}

impl Type {
    fn is_valid_in(&self, program: &ScopedProgram) -> bool {
        match self {
            Type::Int => true,
            Type::ADT(aid) => program.adts.contains_key(aid),
        }
    }
}

impl UTuple<Type> {
    fn is_valid_in(&self, program: &ScopedProgram) -> bool {
        self.0.iter().map(|tp| tp.is_valid_in(program)).all(|x| x)
    }
}

impl<'a> ScopedExpressionNode<'a> {
    fn validate(&self, program: &ScopedProgram) {
        match self.expr {
            Expression::FunctionCall(fid, args) => {
                let Some(func) = program.functions.get(fid) else { panic!("Unknown function {fid}"); };
                let arg_type = &func.def.signature.argument_type;

                if args.0.len() != arg_type.0.len() {
                    panic!("Expected {} arguments but found {} when invoking {}", arg_type.0.len(), args.0.len(), func.def.id);
                }
            },
            Expression::Variable(vid) => if !self.scope.contains_key(vid) { panic!("Unknown variable {vid}") },
            Expression::Match(match_expression) => todo!(),
            Expression::Operation(operator, expression, expression1) => todo!(),
            _ => ()
        }

        for node in self.children.scopes() {
            node.validate(program);
        }
    }
}

fn get_new_internal_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

// Creates a ScopeExpressionNode recursively for the expression
// Each node contains a mapping from VID to VariableDefinition
// A variable definition contains type information 
// Checks that each case in match has correct number of arguments for the constructor
// Doesn't perform any type checking or other types of validation, but does do type inference
fn scope_expression<'a>(
    expr: &'a Expression, 
    scope: &Scope<'a>, 
    new_vars: Vec<VariableDefinition>, 
    function_signatures: &HashMap<FID, &'a FunctionSignature>,
    constructor_signatures: &HashMap<FID, &'a ConstructorSignature>,
) -> ScopedExpressionNode<'a> 
{
    let mut scope = scope.clone();
    for var in new_vars {
        scope.insert(var.id.clone(), Rc::new(var));
    }

    ScopedExpressionNode {
        expr,
        children: match expr {
            Expression::FunctionCall(_, tup) |
            Expression::UTuple(tup) => 
                ScopeChildren::Many(tup.0.iter().map(|expr| scope_expression(expr, &scope, vec![], function_signatures, constructor_signatures)).collect()),
            Expression::Integer(_) |
            Expression::Variable(_) => 
                ScopeChildren::Zero,
            Expression::Operation(_, e1, e2) =>
                ScopeChildren::Two(
                    Box::new(scope_expression(e1, &scope, vec![], function_signatures, constructor_signatures)), 
                    Box::new(scope_expression(e2, &scope, vec![], function_signatures, constructor_signatures))
                ),
            Expression::LetEqualIn(vars, e1, e2) => {
                let mut new_vars = vec![];
                
                let signature = match &**e1 {
                    Expression::FunctionCall(fid, _) => function_signatures.get(fid).expect(&format!("Unknown function \"{}\"", fid)),
                    _ => panic!("Expected a function call in let statement")
                };

                if vars.0.len() != signature.result_type.0.len() {
                    panic!("Wrong number of arguments in let statement");
                }

                for (vid, tp) in vars.0.iter().zip(signature.result_type.0.iter()) {
                    new_vars.push(VariableDefinition { id: vid.clone(), tp: tp.clone(), internal_id: get_new_internal_id() });
                }

                ScopeChildren::Two(
                    Box::new(scope_expression(e1, &scope, vec![], function_signatures, constructor_signatures)), 
                    Box::new(scope_expression(e2, &scope, new_vars, function_signatures, constructor_signatures))
                )
            },
            Expression::Match(expr) => {
                ScopeChildren::Many(
                    expr.cases.iter().map(|case| {
                        let cons_sig: &ConstructorSignature = constructor_signatures.get(&case.cons_id).expect(&format!("Unknown constructor \"{}\"", case.cons_id));
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
                    }).collect()
                )
            }
        },
        scope,
    }
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

        write_indent(f, 1)?;
        write!(f, "// ")?;
        write_scope(f, &self.body.scope, |f, x| write!(f, "{}|{}[{}]", x.id, x.internal_id, x.tp))?;

        write_scoped_expression_node(f, &self.body, &self.body.scope,1)?;

        Ok(())
    }
}

fn write_scoped_expression_node<'a>(f: &mut Formatter<'_>, node: &'a ScopedExpressionNode<'a>, mut previous_scope: &'a Scope, indent: usize) -> std::fmt::Result {
    write_indent(f, indent)?;
    if !scopes_are_equal(previous_scope, &node.scope) {
        previous_scope = &node.scope;

        write!(f, "// ")?;
        write_scope(f, &node.scope, |f, x| write!(f, "{}|{}[{}]", x.id, x.internal_id, x.tp))?;
    }

    for child in node.children.scopes() {
        writeln!(f)?;
        write_scoped_expression_node(f, child, previous_scope, indent+1)?;
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