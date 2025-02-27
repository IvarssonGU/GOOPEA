use std::{collections::{HashMap, HashSet}, rc::Rc};
use crate::ast::{ADTDefinition, ConstructorDefinition, Definition, Expression, FunctionDefinition, Program, Type, AID, FID, VID};

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
    tp: Type
}

type Scope<'a> = HashMap<VID, Rc<VariableDefinition>>;

#[derive(Clone, Debug)]
pub struct ScopedExpressionNode<'a> {
    expr: &'a Expression,
    child_scopes: ChildScopes<'a>,
    scope: Scope<'a>,
}

#[derive(Debug, Clone)]
pub enum ChildScopes<'a> {
    Many(Vec<ScopedExpressionNode<'a>>),
    Two(Box<ScopedExpressionNode<'a>>, Box<ScopedExpressionNode<'a>>),
    Zero
}

impl<'a> ChildScopes<'a> {
    fn scopes(&self) -> Vec<&ScopedExpressionNode<'a>> {
        match &self {
            ChildScopes::Many(s) => s.iter().collect(),
            ChildScopes::Two(s1, s2) => vec![s1, s2],
            ChildScopes::Zero => vec![]
        }
    }
}

#[derive(Debug)]
pub struct ADTScope<'a> {
    adts: HashMap<AID, &'a ADTDefinition>,
    constructors: HashMap<FID, ConstructorReference<'a>>,
}

#[derive(Debug)]
pub struct ScopedProgram<'a> {
    adts: ADTScope<'a>,
    functions: HashMap<FID, ScopedFunction<'a>>
}

impl<'a> ScopedProgram<'a> {
    // Creates a new program with scope information
    // Performs minimum required validation
    pub fn new(program: &'a Program) -> ScopedProgram<'a> {
        let mut top_level_fids = HashSet::new();
        let mut top_level_aids = HashSet::new();

        let mut adts = HashMap::new();
        let mut constructors = HashMap::new();
        for def in &program.0 {
            if let Definition::ADTDefinition(def) = def {
                if !top_level_aids.insert(&def.id) {
                    panic!("ADT identifier {} used twice", def.id);
                }

                adts.insert(def.id.clone(), def);

                for cons in &def.constructors {
                    if !top_level_fids.insert(&cons.id) {
                        panic!("Function identifier {} used twice", cons.id);
                    }

                    constructors.insert(cons.id.clone(), ConstructorReference { adt: &def, constructor: &cons });
                }
            }
        }

        let adt_scope = ADTScope { adts, constructors };

        let mut functions = HashMap::new();
        for def in &program.0 {
            if let Definition::FunctionDefinition(def) = def {
                if !top_level_fids.insert(&def.id) {
                    panic!("Function identifier {} used twice", def.id);
                }

                if def.variables.0.len() != def.signature.argument_type.0.len() {
                    panic!("Missmatched variable count in signature vs definition of function {}", def.id);
                }

                let function_variables = def.variables.0.iter().zip(def.signature.result_type.0.iter()).map(
                    |(vid, tp)| {
                        VariableDefinition { id: vid.clone(), tp: tp.clone() }
                    }
                ).collect::<Vec<_>>();

                functions.insert(
                    def.id.clone(), 
                    ScopedFunction { 
                        def, 
                        body: scope_expression(&def.body, &HashMap::new(), function_variables, &adt_scope) 
                    }
                );
            }
        }

        ScopedProgram {
            adts: adt_scope,
            functions
        }
    }
}

// Creates a ScopeExpressionNode recursively for the expression
// Each node contains a mapping from VID to VariableDefinition
// A variable definition contains type information 
// Checks that each case in match has correct number of arguments for the constructor
// Doesn't perform any type checking or other types of validation
fn scope_expression<'a>(expr: &'a Expression, scope: &Scope<'a>, new_vars: Vec<VariableDefinition>, adt_scope: &ADTScope) -> ScopedExpressionNode<'a> {
    let mut scope = scope.clone();
    for var in new_vars {
        scope.insert(var.id.clone(), Rc::new(var));
    }

    ScopedExpressionNode {
        expr,
        child_scopes: match expr {
            Expression::FunctionCall(_, tup) |
            Expression::UTuple(tup) => 
                ChildScopes::Many(tup.0.iter().map(|expr| scope_expression(expr, &scope, vec![], adt_scope)).collect()),
            Expression::Integer(_) |
            Expression::Variable(_) => 
                ChildScopes::Zero,
            Expression::Operation(_, e1, e2) =>
                ChildScopes::Two(
                    Box::new(scope_expression(e1, &scope, vec![], adt_scope)), 
                    Box::new(scope_expression(e2, &scope, vec![], adt_scope))
                ),
            Expression::Match(expr) => {
                ChildScopes::Many(
                    expr.cases.iter().map(|case| {
                        let cons = adt_scope.constructors.get(&case.cons_id).expect(&format!("The constructor {} has not been defined", case.cons_id)).constructor;
                        if cons.arguments.0.len() != case.vars.0.len() {
                            panic!("Wrong number of arguments in match statement of case {}", case.cons_id);
                        }

                        scope_expression(
                            &case.body,
                                &scope, 
                                case.vars.0.iter().zip(cons.arguments.0.iter()).map(|(new_vid, tp)| {
                                VariableDefinition {
                                    id: new_vid.clone(),
                                    tp: tp.clone()
                                }
                                }).collect(),
                                adt_scope
                        )
                    }).collect()
                )
            }
        },
        scope,
    }
}

fn test() {
    let a: Program = Program(vec![]);
    let p = ScopedProgram::new(&a);
    //p.functions.get("A").unwrap().;
}