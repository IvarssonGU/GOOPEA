//stir = Sequentially-Transformed-Intermediate-Representation
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter, Result};

use crate::ast::{
    ast, scoped,
    typed::{TypedNode, TypedProgram},
};

pub type Stir = Vec<Function>;

type Var = String;
type Constant = String;
type Tag = u8;

#[derive(Debug, Clone)]
pub struct Function {
    pub id: Constant,
    pub args: Vec<Var>,
    pub body: Body,
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{} {} = {}", self.id, self.args.join(" "), self.body)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Body {
    Ret(Var),
    Let(Var, Exp, Box<Body>),
    Match(Var, Vec<(u8, Body)>),
    Inc(Var, Box<Body>),
    Dec(Var, Box<Body>),
}

impl Body {
    pub fn member(&self, var: &Var) -> bool {
        match self {
            Body::Ret(v) => v == var,
            Body::Let(_, exp, body) => exp.member(var) || body.member(var), //Not sure if this should be like this or as below
            //Body::Let(v, exp, body) => v == var || exp.member(var) || body.member(var),
            Body::Match(_, branches) => branches.iter().any(|(i, b)| b.member(var)),
            _ => todo!(),
        }
    }

    fn pretty_string(&self, indent: usize) -> String {
        match self {
            Body::Ret(var) => format!("{}ret {}\n", " ".repeat(indent), var),
            Body::Let(var, exp, body) => format!(
                "{}let {} = {};\n{}",
                " ".repeat(indent),
                var,
                exp,
                body.pretty_string(indent)
            ),
            Body::Match(var, branches) => {
                let mut result = format!("{}case {} of\n", " ".repeat(indent), var);
                for (i, branch) in branches {
                    result.push_str(&branch.pretty_string(indent + 2));
                }
                result
            }
            Body::Inc(var, body) => format!(
                "{}inc {};\n{}",
                " ".repeat(indent),
                var,
                body.pretty_string(indent)
            ),
            Body::Dec(var, body) => format!(
                "{}dec {};\n{}",
                " ".repeat(indent),
                var,
                body.pretty_string(indent)
            ),
        }
    }
}

impl Display for Body {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.pretty_string(0))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Exp {
    App(Constant, Vec<Var>),
    Ctor(Tag, Vec<Var>),
    Proj(u8, Var),
    UTuple(Vec<Var>),
    Int(i64),
    Op(Operator, Var, Var),
    Reset(Var),
    Reuse(Var, Tag, Vec<Var>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Status {
    Owned,
    Borrowed,
}

impl Exp {
    pub fn member(&self, var: &Var) -> bool {
        match self {
            Exp::App(_, vars) => vars.iter().any(|v| v == var),
            Exp::Ctor(_, vars) => vars.iter().any(|v| v == var),
            Exp::Proj(_, v) => v == var,
            Exp::Op(_, v1, v2) => v1 == var || v2 == var,
            Exp::Int(_) => false,
            Exp::Reset(_) => false,
            Exp::Reuse(_, _, _) => false,
            Exp::UTuple(vars) => vars.iter().any(|v| v == var),
        }
    }
}

impl Display for Exp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Exp::App(name, args) => write!(
                f,
                "{}({})",
                name,
                if args.is_empty() {
                    "[]".to_string()
                } else {
                    args.iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                }
            ),
            Exp::Ctor(tag, args) => write!(
                f,
                "Ctor({}, {})",
                tag,
                if args.is_empty() {
                    "[]".to_string()
                } else {
                    args.iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                }
            ),
            Exp::Proj(tag, var) => write!(f, "Proj({}, {})", tag, var),
            Exp::Int(i) => write!(f, "{}", i),
            Exp::Op(op, var1, var2) => write!(f, "{} {} {}", var1, op, var2),
            Exp::Reset(var) => write!(f, "reset {}", var),
            Exp::Reuse(var, tag, args) => write!(
                f,
                "reuse {} in Ctor({}, {})",
                var,
                tag,
                if args.is_empty() {
                    "[]".to_string()
                } else {
                    args.iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                }
            ),
            Exp::UTuple(vars) => write!(
                f,
                "UTuple({})",
                if vars.is_empty() {
                    "[]".to_string()
                } else {
                    vars.iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                }
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Equal,
    NotEqual,
    Less,
    LessOrEq,
    Greater,
    GreaterOrEqual,
    Add,
    Sub,
    Mul,
    Div,
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Operator::Equal => write!(f, "=="),
            Operator::NotEqual => write!(f, "!="),
            Operator::Less => write!(f, "<"),
            Operator::LessOrEq => write!(f, "<="),
            Operator::Greater => write!(f, ">"),
            Operator::GreaterOrEqual => write!(f, ">="),
            Operator::Add => write!(f, "+"),
            Operator::Sub => write!(f, "-"),
            Operator::Mul => write!(f, "*"),
            Operator::Div => write!(f, "/"),
        }
    }
}

#[derive(Clone)]
pub enum Simple {
    Ident(Var),
    Int(i64),
    Operation(Operator, Box<Simple>, Box<Simple>),
    Constructor(i64, Vec<Simple>),
    App(String, Vec<Simple>),
    Match(Box<Simple>, Vec<(Pattern, Simple)>),
    Let(String, Box<Simple>, Box<Simple>),
    UTuple(Vec<Simple>),
    LetApp(Vec<String>, Box<Simple>, Box<Simple>),
}

type Pattern = (i64, Vec<Binder>);

#[derive(Debug, Clone)]
pub enum Binder {
    Variable(String),
    Wildcard,
}

fn next_var() -> Var {
    thread_local!(
        static COUNTER: RefCell<usize> = Default::default();
    );
    let current = COUNTER.with_borrow_mut(|c| {
        *c += 1;
        *c
    });
    format!("fresh{}", current)
}

pub fn from_typed(typed: &TypedProgram) -> Stir {
    let mut program = vec![];

    for (id, func, body) in typed.function_iter() {
        program.push(Function {
            id: id.clone(),
            args: func.vars.0.clone(),
            body: remove_dead_bindings(from_simple(&from_typed_expr(&body, typed), &|var| {
                Body::Ret(var)
            })),
        });
    }
    program
}

fn from_typed_expr(expr: &TypedNode, context: &TypedProgram) -> Simple {
    match &expr.expr {
        scoped::SimplifiedExpression::FunctionCall(id, args) => match id.as_str() {
            "+" => Simple::Operation(
                Operator::Add,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
            ),
            "-" => Simple::Operation(
                Operator::Sub,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
            ),
            "*" => Simple::Operation(
                Operator::Mul,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
            ),
            "/" => Simple::Operation(
                Operator::Div,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
            ),
            ">" => Simple::Operation(
                Operator::Greater,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
            ),
            "<" => Simple::Operation(
                Operator::Less,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
            ),
            ">=" => Simple::Operation(
                Operator::GreaterOrEqual,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
            ),
            "<=" => Simple::Operation(
                Operator::LessOrEq,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
            ),
            "==" => Simple::Operation(
                Operator::Equal,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
            ),
            "!=" => Simple::Operation(
                Operator::NotEqual,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
            ),
            _ => match context.constructors.get(id) {
                Some(cons) => {
                    if args.0.is_empty() {
                        Simple::Int(cons.sibling_index as i64)
                    } else {
                        Simple::Constructor(
                            cons.sibling_index as i64,
                            args.0
                                .iter()
                                .map(|arg| from_typed_expr(arg, context))
                                .collect(),
                        )
                    }
                }
                None => Simple::App(
                    id.clone(),
                    args.0
                        .iter()
                        .map(|arg| from_typed_expr(arg, context))
                        .collect(),
                ),
            },
        },
        scoped::SimplifiedExpression::Integer(i) => Simple::Int(i.clone()),
        scoped::SimplifiedExpression::Variable(id) => Simple::Ident(id.clone()),
        scoped::SimplifiedExpression::Match(var_node, cases) => Simple::Match(
            Simple::Ident(var_node.expr.clone()).into(),
            cases
                .iter()
                .map(|(pattern, exp)| {
                    (
                        {
                            match pattern {
                                ast::Pattern::Integer(_) => todo!(),
                                ast::Pattern::Variable(_) => todo!(),
                                ast::Pattern::Constructor(fid, vars) => {
                                    if vars.0.is_empty() {
                                        (
                                            context.constructors.get(fid).unwrap().sibling_index
                                                as i64,
                                            vec![],
                                        )
                                    } else {
                                        (
                                            context.constructors.get(fid).unwrap().sibling_index
                                                as i64,
                                            vars.0
                                                .iter()
                                                .map(|var| Binder::Variable(var.clone()))
                                                .collect(),
                                        )
                                    }
                                }
                            }
                        },
                        from_typed_expr(exp, context),
                    )
                })
                .collect(),
        ),
        scoped::SimplifiedExpression::UTuple(args) => Simple::UTuple(
            args.0
                .iter()
                .map(|arg| from_typed_expr(arg, context))
                .collect(),
        ),
        scoped::SimplifiedExpression::LetEqualIn(bindings, exp, next) if bindings.0.len() == 1 => {
            Simple::Let(
                bindings.0[0].clone(),
                from_typed_expr(exp, context).into(),
                from_typed_expr(next, context).into(),
            )
        }
        scoped::SimplifiedExpression::LetEqualIn(bindings, exp, next) => Simple::LetApp(
            bindings.0.clone(),
            from_typed_expr(exp, context).into(),
            from_typed_expr(next, context).into(),
        ),
    }
}

pub fn from_simple(expr: &Simple, k: &dyn Fn(Var) -> Body) -> Body {
    match expr {
        Simple::Ident(var) => k(var.clone()),
        Simple::Int(i) => {
            let fresh = next_var();
            Body::Let(fresh.clone(), Exp::Int(*i), k(fresh).into())
        }
        Simple::Operation(op, left, right) => from_simple(left, &move |var1| {
            from_simple(right, &move |var2| {
                let fresh: String = next_var();
                Body::Let(
                    fresh.clone(),
                    Exp::Op(*op, var1.clone(), var2),
                    k(fresh).into(),
                )
            })
        }),
        Simple::App(id, inner) => translate_list(inner.clone(), &move |vars| {
            let fresh = next_var();
            Body::Let(fresh.clone(), Exp::App(id.clone(), vars), k(fresh).into())
        }),
        Simple::Constructor(tag, inner) => translate_list(inner.clone(), &move |vars| {
            let fresh = next_var();
            Body::Let(fresh.clone(), Exp::Ctor(*tag as u8, vars), k(fresh).into())
        }),
        Simple::UTuple(inner) => translate_list(inner.clone(), &move |vars| {
            let fresh = next_var();
            Body::Let(fresh.clone(), Exp::UTuple(vars), k(fresh).into())
        }),
        Simple::Match(expr, branches) => {
            let mut branches = branches.clone();
            branches.sort_by_key(|((tag, _), _)| *tag);
            from_simple(expr, &move |var| {
                let mut new_bodies: Vec<(u8, Body)> = vec![];
                for ((_, binders), expr) in &branches {
                    let mut body = from_simple(expr, &move |var: String| Body::Ret(var));
                    for i in (0..binders.len()).rev() {
                        match &binders[i] {
                            Binder::Variable(binder) => {
                                body = Body::Let(
                                    binder.clone(),
                                    Exp::Proj(i as u8, var.clone()),
                                    body.into(),
                                );
                            }
                            Binder::Wildcard => (),
                        }
                    }
                    new_bodies.push((binders.len() as u8, body));
                }
                Body::Match(var, new_bodies)
            })
        }
        Simple::Let(var, exp, next) => from_simple(exp, &move |var1| {
            replace_var_body(var1, var, from_simple(next, &move |var2| k(var2).into()))
        }),
        Simple::LetApp(vars, exp, next) => from_simple(exp, &move |var1| {
            vars.iter().enumerate().rev().fold(
                from_simple(next, &move |var2| k(var2).into()),
                |acc, (i, var)| {
                    Body::Let(var.clone(), Exp::Proj(i as u8, var1.clone()), acc.into())
                },
            )
        }),
    }
}

fn translate_list(exprs: Vec<Simple>, k: &dyn Fn(Vec<Var>) -> Body) -> Body {
    if exprs.is_empty() {
        k(vec![])
    } else {
        let first = &exprs[0];
        let rest = exprs[1..].to_vec();
        from_simple(first, &move |var_first| {
            translate_list(rest.clone(), &move |vars_rest| {
                let mut all_vars = vec![var_first.clone()];
                all_vars.extend(vars_rest);
                k(all_vars)
            })
        })
    }
}

fn replace_var_body(replacing: String, replacee: &String, body: Body) -> Body {
    match body {
        Body::Ret(var) => Body::Ret(replace_var(var, replacing.clone(), replacee)),
        Body::Let(var, exp, next) => Body::Let(
            replace_var(var, replacing.clone(), replacee),
            replace_var_exp(replacing.clone(), replacee, exp),
            replace_var_body(replacing.clone(), replacee, *next).into(),
        ),
        Body::Match(var, branches) => Body::Match(
            replace_var(var, replacing.clone(), replacee),
            branches
                .iter()
                .map(|(cons_len, branch)| {
                    (
                        *cons_len,
                        replace_var_body(replacing.clone(), replacee, branch.clone()),
                    )
                })
                .collect(),
        ),
        _ => panic!("Does not exist at this stage"),
    }
}

fn replace_var_exp(replacing: String, replacee: &String, exp: Exp) -> Exp {
    match exp {
        Exp::App(id, args) => Exp::App(
            id,
            args.iter()
                .map(|arg| replace_var(arg.clone(), replacing.clone(), replacee))
                .collect(),
        ),
        Exp::Ctor(tag, args) => Exp::Ctor(
            tag,
            args.iter()
                .map(|arg| replace_var(arg.clone(), replacing.clone(), replacee))
                .collect(),
        ),
        Exp::Proj(tag, var) => Exp::Proj(tag, replace_var(var, replacing.clone(), replacee)),
        Exp::Int(i) => Exp::Int(i),
        Exp::Op(op, var1, var2) => Exp::Op(
            op,
            replace_var(var1, replacing.clone(), replacee),
            replace_var(var2, replacing.clone(), replacee),
        ),
        _ => panic!("Does not exist at this stage"),
    }
}

fn replace_var(var: String, replacing: String, replacee: &String) -> String {
    if var == *replacee { replacing } else { var }
}

fn remove_dead_bindings(body: Body) -> Body {
    match body {
        Body::Ret(var) => Body::Ret(var),
        Body::Let(var, exp, next) => {
            if free_vars(&next).contains(&var) {
                Body::Let(var, exp, remove_dead_bindings(*next).into())
            } else {
                remove_dead_bindings(*next)
            }
        }
        Body::Match(var, branches) => Body::Match(
            var,
            branches
                .iter()
                .map(|(cons_len, branch)| (*cons_len, remove_dead_bindings(branch.clone())))
                .collect(),
        ),
        _ => panic!("Does not exist at this stage"),
    }
}

fn reuse_all_matches(body: &Body) -> Body {
    match body {
        Body::Ret(var) => Body::Ret(var.clone()),
        Body::Let(var, exp, next) => {
            Body::Let(var.clone(), exp.clone(), reuse_all_matches(next).into())
        }
        Body::Match(var, branches) => {
            let mut new_branches = vec![];
            for (cons_len, branch) in branches {
                new_branches.push((
                    *cons_len,
                    evaluate_reuse_in_case(var.clone(), *cons_len, &reuse_all_matches(branch)),
                ));
            }
            Body::Match(var.clone(), new_branches)
        }
        _ => panic!("Does not exist at this stage"),
    }
}

fn evaluate_reuse_in_case(var: String, len: u8, body: &Body) -> Body {
    match body {
        Body::Match(case_var, branches) => Body::Match(
            case_var.clone(),
            branches
                .iter()
                .map(|(cons_len, branch)| {
                    (*cons_len, evaluate_reuse_in_case(var.clone(), len, branch))
                })
                .collect(),
        ),
        Body::Ret(ret_var) => Body::Ret(ret_var.clone()),
        Body::Let(let_var, exp, next) if exp.member(&var) || next.member(&var) => Body::Let(
            let_var.clone(),
            exp.clone(),
            evaluate_reuse_in_case(var, len, next).into(),
        ),
        _ => {
            let fresh = next_var();
            let try_replace = insert_reuse(fresh.clone(), len, body);
            if try_replace != *body {
                Body::Let(fresh, Exp::Reset(var), try_replace.into())
            } else {
                try_replace
            }
        }
    }
}

fn insert_reuse(var: String, len: u8, body: &Body) -> Body {
    match body {
        Body::Let(let_var, exp, next) => match exp {
            Exp::Ctor(tag, vars) if vars.len() as u8 == len => Body::Let(
                let_var.clone(),
                Exp::Reuse(var, *tag, vars.clone()),
                next.clone(),
            ),
            _ => Body::Let(
                let_var.clone(),
                exp.clone(),
                insert_reuse(var, len, next).into(),
            ),
        },
        Body::Ret(ret_var) => Body::Ret(ret_var.clone()),
        Body::Match(case_var, branches) => Body::Match(
            case_var.clone(),
            branches
                .iter()
                .map(|(cons_len, branch)| (*cons_len, insert_reuse(var.clone(), len, branch)))
                .collect(),
        ),
        _ => panic!("Does not exist at this stage"),
    }
}

pub fn add_reuse(prog: &Stir) -> Stir {
    prog.iter()
        .map(|func| Function {
            id: func.id.clone(),
            args: func.args.clone(),
            body: reuse_all_matches(&func.body),
        })
        .collect()
}

pub fn get_ownership(prog: &Stir) -> HashMap<Constant, Vec<Status>> {
    let mut map = HashMap::new();
    for func in prog {
        map.insert(func.id.clone(), vec![Status::Borrowed; func.args.len()]);
    }
    let mut change = true;
    while change {
        change = false;
        let mut new_map = map.clone();
        for func in prog {
            let vars = collect(&func.body, &new_map);
            for (i, arg) in func.args.iter().enumerate() {
                if vars.contains(arg) {
                    let mut ownership = new_map.get(&func.id).unwrap().clone();
                    ownership[i] = Status::Owned;
                    new_map.insert(func.id.clone(), ownership.clone());
                }
            }
        }
        if map != new_map {
            map = new_map;
            change = true;
        }
    }
    map
}

fn collect(body: &Body, map: &HashMap<Constant, Vec<Status>>) -> HashSet<Var> {
    match body {
        Body::Let(var, e, next) => match e {
            Exp::Int(_) => collect(next, map),
            Exp::Ctor(_, _) => collect(next, map),
            Exp::Reset(var) => {
                let mut set = collect(next, map);
                set.insert(var.clone());
                set
            }
            Exp::Reuse(_, _, _) => collect(next, map),
            Exp::App(fid, args) => {
                let mut set = collect(next, map);
                if let Some(statuses) = map.get(fid) {
                    for i in 0..args.len() {
                        match statuses[i] {
                            Status::Owned => {
                                set.insert(args[i].clone());
                            }
                            Status::Borrowed => (),
                        }
                    }
                }
                set
            }
            Exp::Op(_, _, _) => collect(next, map),
            Exp::Proj(_, v) => {
                let mut set = collect(next, map);
                if set.contains(var) {
                    set.insert(v.clone());
                }
                set
            }
            Exp::UTuple(_) => collect(next, map),
        },
        Body::Ret(_) => HashSet::new(),
        Body::Match(_, branches) => {
            let mut combined = HashSet::new();
            for (_, branch) in branches {
                combined = combined.union(&collect(branch, map)).cloned().collect();
            }
            combined
        }
        _ => panic!("Does not exist at this stage "),
    }
}

pub fn add_rc(prog: &Stir, flag: bool) -> Stir {
    if flag {
        prog.iter()
            .map(|func| insert_rc_fun(func, &get_ownership(prog)))
            .collect()
    } else {
        let mut ownership = HashMap::new();
        for func in prog {
            ownership.insert(func.id.clone(), vec![Status::Owned; func.args.len()]);
        }
        prog.iter()
            .map(|func| insert_rc_fun(func, &ownership))
            .collect()
    }
}

fn insert_rc_fun(func: &Function, beta_map: &HashMap<Constant, Vec<Status>>) -> Function {
    let mut betal = HashMap::new();
    for (i, status) in beta_map.get(&func.id).unwrap().iter().enumerate() {
        betal.insert(func.args[i].clone(), *status);
    }
    Function {
        id: func.id.clone(),
        args: func.args.clone(),
        body: owned_minus_all(
            func.args.clone(),
            &insert_rc_body(&func.body, &betal, beta_map),
            &betal,
        ),
    }
}

fn insert_rc_body(
    body: &Body,
    betal: &HashMap<Var, Status>,
    beta_map: &HashMap<Constant, Vec<Status>>,
) -> Body {
    match body {
        Body::Ret(var) => owned_plus(var.clone(), HashSet::new(), body, betal),
        Body::Match(var, branches) => {
            let vars = Vec::from_iter(free_vars(body).iter().cloned());
            Body::Match(
                var.clone(),
                branches
                    .iter()
                    .map(|(i, branch)| {
                        (
                            *i,
                            owned_minus_all(
                                vars.clone(),
                                &insert_rc_body(branch, betal, beta_map),
                                betal,
                            ),
                        )
                    })
                    .collect(),
            )
        }
        Body::Let(var, exp, next) => match exp {
            Exp::Proj(_, proj_var) if default_betal(proj_var, betal) == Status::Owned => Body::Let(
                var.clone(),
                exp.clone(),
                Body::Inc(
                    var.clone(),
                    owned_minus(proj_var, &insert_rc_body(next, betal, beta_map), betal).into(),
                )
                .into(),
            ),
            Exp::Proj(_, proj_var) if default_betal(proj_var, betal) == Status::Borrowed => {
                let mut new_betal = betal.clone();
                new_betal.insert(var.clone(), Status::Borrowed);
                Body::Let(
                    var.clone(),
                    exp.clone(),
                    insert_rc_body(next, &new_betal, beta_map).into(),
                )
            }
            Exp::Reset(_) => Body::Let(
                var.clone(),
                exp.clone(),
                insert_rc_body(next, betal, beta_map).into(),
            ),
            Exp::App(fid, args) => cappy(
                args.clone(),
                beta_map.get(fid).unwrap().clone(),
                &Body::Let(
                    var.clone(),
                    exp.clone(),
                    insert_rc_body(next, betal, beta_map).into(),
                ),
                betal,
            ),
            Exp::Ctor(_, args) => cappy(
                args.clone(),
                vec![Status::Owned; args.len()],
                &Body::Let(
                    var.clone(),
                    exp.clone(),
                    insert_rc_body(next, betal, beta_map).into(),
                ),
                betal,
            ),
            Exp::UTuple(args) => cappy(
                args.clone(),
                vec![Status::Owned; args.len()],
                &Body::Let(
                    var.clone(),
                    exp.clone(),
                    insert_rc_body(next, betal, beta_map).into(),
                ),
                betal,
            ),
            Exp::Reuse(_, _, args) => cappy(
                args.clone(),
                vec![Status::Owned; args.len()],
                &Body::Let(
                    var.clone(),
                    exp.clone(),
                    insert_rc_body(next, betal, beta_map).into(),
                ),
                betal,
            ),
            Exp::Int(_) => Body::Let(
                var.clone(),
                exp.clone(),
                insert_rc_body(next, betal, beta_map).into(),
            ),
            Exp::Op(_, _, _) => Body::Let(
                var.clone(),
                exp.clone(),
                insert_rc_body(next, betal, beta_map).into(),
            ),
            _ => todo!(),
        },
        _ => todo!(),
    }
}

fn cappy(
    mut vars: Vec<Var>,
    mut stats: Vec<Status>,
    body: &Body,
    betal: &HashMap<Var, Status>,
) -> Body {
    let Body::Let(var, exp, next) = body else {
        panic!("Expected a Let body");
    };
    if vars.is_empty() {
        body.clone()
    } else {
        let top = vars.pop().unwrap();
        let new_vars = vars.clone();
        let top_status = stats.pop().unwrap();
        let new_stats = stats.clone();
        if top_status == Status::Owned {
            let mut live_vars = free_vars(next);
            live_vars.extend(new_vars.clone());
            owned_plus(
                top.clone(),
                live_vars,
                &cappy(new_vars, new_stats, body, betal),
                betal,
            )
        } else {
            cappy(
                new_vars,
                new_stats,
                &Body::Let(
                    var.clone(),
                    exp.clone(),
                    owned_minus(&top, next, betal).into(),
                ),
                betal,
            )
        }
    }
}

fn default_betal(var: &Var, map: &HashMap<Var, Status>) -> Status {
    if let Some(status) = map.get(var) {
        *status
    } else {
        Status::Owned
    }
}

fn owned_plus(
    var: Var,
    live_vars: HashSet<Var>,
    body: &Body,
    betal: &HashMap<Var, Status>,
) -> Body {
    if default_betal(&var, betal) == Status::Owned && !live_vars.contains(&var) {
        body.clone()
    } else {
        Body::Inc(var.clone(), body.clone().into())
    }
}

fn owned_minus(var: &String, body: &Body, betal: &HashMap<Var, Status>) -> Body {
    if default_betal(var, betal) == Status::Owned && !free_vars(body).contains(var) {
        Body::Dec(var.clone(), body.clone().into())
    } else {
        body.clone()
    }
}

fn owned_minus_all(vars: Vec<Var>, body: &Body, betal: &HashMap<Var, Status>) -> Body {
    let mut new_body = body.clone();
    for var in vars {
        new_body = owned_minus(&var, &new_body, betal);
    }
    new_body
}

fn free_vars_exp(exp: &Exp, bound: &HashSet<Var>) -> HashSet<Var> {
    match exp {
        Exp::App(_, args) => {
            let mut set = HashSet::new();
            for arg in args {
                if !bound.contains(arg) {
                    set.insert(arg.clone());
                }
            }
            set
        }
        Exp::Ctor(_, args) => {
            let mut set = HashSet::new();
            for arg in args {
                if !bound.contains(arg) {
                    set.insert(arg.clone());
                }
            }
            set
        }
        Exp::Proj(_, var) => {
            let mut set = HashSet::new();
            if !bound.contains(var) {
                set.insert(var.clone());
            }
            set
        }
        Exp::Op(_, left, right) => {
            let mut set = HashSet::new();
            if !bound.contains(left) {
                set.insert(left.clone());
            }
            if !bound.contains(right) {
                set.insert(right.clone());
            }
            set
        }
        Exp::Reset(var) => {
            let mut set = HashSet::new();
            if !bound.contains(var) {
                set.insert(var.clone());
            }
            set
        }
        Exp::Reuse(var, _, args) => {
            let mut set = HashSet::new();
            if !bound.contains(var) {
                set.insert(var.clone());
            }
            for arg in args {
                if !bound.contains(arg) {
                    set.insert(arg.clone());
                }
            }
            set
        }
        Exp::Int(_) => HashSet::new(),
        Exp::UTuple(args) => {
            let mut set = HashSet::new();
            for arg in args {
                if !bound.contains(arg) {
                    set.insert(arg.clone());
                }
            }
            set
        }
    }
}

fn free_vars_helper(body: &Body, mut bound: HashSet<Var>) -> HashSet<Var> {
    match body {
        Body::Ret(var) => {
            let mut set = HashSet::new();
            if !bound.contains(var) {
                set.insert(var.clone());
            }
            set
        }
        Body::Let(var, exp, next) => {
            let expr_set = free_vars_exp(exp, &bound);
            bound.insert(var.clone());
            let mut set = free_vars_helper(next, bound);
            set.extend(expr_set);
            set
        }
        Body::Match(var, branches) => {
            let mut set = HashSet::new();
            for (_, branch) in branches {
                set.extend(free_vars_helper(branch, bound.clone()));
            }
            if !bound.contains(var) {
                set.insert(var.clone());
            }
            set
        }
        Body::Inc(var, next) => {
            let mut set = free_vars_helper(next, bound.clone());
            if !bound.contains(var) {
                set.insert(var.clone());
            }
            set
        }
        Body::Dec(var, next) => {
            let mut set = free_vars_helper(next, bound.clone());
            if !bound.contains(var) {
                set.insert(var.clone());
            }
            set
        }
    }
}

fn free_vars(body: &Body) -> HashSet<Var> {
    free_vars_helper(body, HashSet::new())
}
