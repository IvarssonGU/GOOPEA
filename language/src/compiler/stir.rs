use core::panic;
//stir = Sequentially-Transformed-Intermediate-Representation
use crate::compiler::simple::{Binder, Operator, Simple, Type, get_type};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter, Result};

pub type Stir = Vec<Function>;

pub type Var = (String, Type);
pub type Constant = String;
type Tag = u8;

#[derive(Debug, Clone)]
pub struct Function {
    pub fip: bool,
    pub id: Constant,
    pub typ: Type,
    pub args: Vec<Var>,
    pub body: Body,
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}{} =\n{}",
            self.id,
            {
                let args = self
                    .args
                    .iter()
                    .map(|(var, _)| var.clone())
                    .collect::<Vec<String>>()
                    .join(" ");
                if args.is_empty() {
                    "".to_string()
                } else {
                    format!(" {}", args)
                }
            },
            self.body.pretty_body(2)
        )
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

    fn pretty_body(&self, indent: usize) -> String {
        let tab = " ".repeat(indent);
        match self {
            Body::Ret(var) => format!("{}ret {}\n", tab, var.0),
            Body::Let(var, exp, body) => format!(
                "{}let {} = {};\n{}",
                " ".repeat(indent),
                var.0,
                exp,
                body.pretty_body(indent)
            ),
            Body::Match(var, branches) => {
                let mut result = format!("{}match {}\n", tab, var.0);
                for (i, (_, branch)) in branches.iter().enumerate() {
                    result.push_str(&format!("{}{} ->\n", " ".repeat(indent + 2), i));
                    result.push_str(&branch.pretty_body(indent + 4));
                }
                result
            }
            Body::Inc(var, body) => format!("{}inc {};\n{}", tab, var.0, body.pretty_body(indent)),
            Body::Dec(var, body) => format!("{}dec {};\n{}", tab, var.0, body.pretty_body(indent)),
        }
    }
}

impl Display for Body {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.pretty_body(0))
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
                        .map(|x| x.0.to_string())
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
                        .map(|x| x.0.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                }
            ),
            Exp::Proj(tag, var) => write!(f, "Proj({}, {})", tag, var.0),
            Exp::Int(i) => write!(f, "{}", i),
            Exp::Op(op, var1, var2) => write!(f, "{} {} {}", var1.0, op, var2.0),
            Exp::Reset(var) => write!(f, "reset {}", var.0),
            Exp::Reuse(var, tag, args) => write!(
                f,
                "reuse {} in Ctor({}, {})",
                var.0,
                tag,
                if args.is_empty() {
                    "[]".to_string()
                } else {
                    args.iter()
                        .map(|x| x.0.to_string())
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
                        .map(|x| x.0.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                }
            ),
        }
    }
}

pub fn next_var() -> String {
    thread_local!(
        static COUNTER: RefCell<usize> = Default::default();
    );
    let current = COUNTER.with_borrow_mut(|c| {
        *c += 1;
        *c
    });
    format!("fresh{}", current)
}

pub fn from_simple(expr: &Simple, k: &dyn Fn(Var) -> Body) -> Body {
    match expr {
        Simple::Ident(var, typ) => k((var.clone(), typ.clone())),
        Simple::Int(i, typ) => {
            let fresh = next_var();
            let binding = (fresh, typ.clone());
            Body::Let(binding.clone(), Exp::Int(*i), k(binding).into())
        }
        Simple::Operation(op, left, right, typ) => from_simple(left, &move |var1| {
            from_simple(right, &move |var2: (String, Type)| {
                let fresh: String = next_var();
                let binding = (fresh, typ.clone());
                Body::Let(
                    binding.clone(),
                    Exp::Op(*op, var1.clone(), var2),
                    k(binding).into(),
                )
            })
        }),
        Simple::App(id, inner, typ) => translate_list(inner.clone(), &move |bindings| {
            let fresh = next_var();
            let binding = (fresh, typ.clone());
            Body::Let(
                binding.clone(),
                Exp::App(id.clone(), bindings),
                k(binding.clone()).into(),
            )
        }),
        Simple::Constructor(tag, inner, typ) => translate_list(inner.clone(), &move |bindings| {
            let fresh = next_var();
            let binding = (fresh, typ.clone());
            Body::Let(
                binding.clone(),
                Exp::Ctor(*tag as u8, bindings),
                k(binding).into(),
            )
        }),
        Simple::UTuple(inner, typ) => translate_list(inner.clone(), &move |vars| {
            let fresh = next_var();
            let binding = (fresh, typ.clone());
            Body::Let(binding.clone(), Exp::UTuple(vars), k(binding).into())
        }),
        Simple::Match(expr, branches, _) => {
            let mut branches = branches.clone();
            branches.sort_by_key(|((tag, _), _)| *tag);
            from_simple(expr, &move |var| {
                let mut new_bodies: Vec<(u8, Body)> = vec![];
                for ((_, binders), expr) in &branches {
                    let mut body = from_simple(expr, k);
                    for i in (0..binders.len()).rev() {
                        match &binders[i] {
                            Binder::Variable(binder, t) => {
                                body = Body::Let(
                                    (binder.clone(), t.clone()),
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
        Simple::Let(var, exp, next, _) => from_simple(exp, &move |var1| {
            replace_var_body(
                var1,
                &(var.clone(), get_type(exp)),
                from_simple(next, &move |var2| k(var2).into()),
            )
        }),
        Simple::LetApp(vars, exp, next, _) => from_simple(exp, &move |var1| {
            vars.iter().enumerate().rev().fold(
                from_simple(next, &move |var2| k(var2).into()),
                |acc, (i, var)| {
                    Body::Let(
                        (
                            var.clone(),
                            if let Type::Unboxed(vec) = get_type(exp) {
                                vec[i].clone()
                            } else {
                                panic!("Expected UTuple in LetApp binding")
                            },
                        ),
                        Exp::Proj(i as u8, var1.clone()),
                        acc.into(),
                    )
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

fn replace_var_body(replacing: Var, replacee: &Var, body: Body) -> Body {
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

fn replace_var_exp(replacing: Var, replacee: &Var, exp: Exp) -> Exp {
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

fn replace_var(var: Var, replacing: Var, replacee: &Var) -> Var {
    if var == *replacee { replacing } else { var }
}

pub fn remove_dead_bindings(body: Body) -> Body {
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

pub fn free_vars(body: &Body) -> HashSet<Var> {
    free_vars_helper(body, HashSet::new())
}
