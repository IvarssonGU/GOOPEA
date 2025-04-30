use core::panic;
//stir = Sequentially-Transformed-Intermediate-Representation
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter, Result};

use crate::ast::ast::{ExpressionNode, UTuple};
use crate::ast::typed::ExpressionType;
use crate::ast::{
    ast, scoped,
    typed::{TypedNode, TypedProgram},
};

pub type Stir = Vec<Function>;

type Var = (String, Type);
type Constant = String;
type Tag = u8;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Type {
    Int,
    Heaped,
    Unboxed(Vec<Type>),
}

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
                    .map(|((var, _))| var.clone())
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
    Ident(String, Type),
    Int(i64, Type),
    Operation(Operator, Box<Simple>, Box<Simple>, Type),
    Constructor(i64, Vec<Simple>, Type),
    App(String, Vec<Simple>, Type),
    Match(Box<Simple>, Vec<(Pattern, Simple)>, Type),
    Let(String, Box<Simple>, Box<Simple>, Type),
    UTuple(Vec<Simple>, Type),
    LetApp(Vec<String>, Box<Simple>, Box<Simple>, Type),
}

type Pattern = (i64, Vec<Binder>);

#[derive(Debug, Clone)]
pub enum Binder {
    Variable(String, Type),
    Wildcard,
}

fn next_var() -> String {
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
            fip: func.signature.is_fip,
            id: id.clone(),
            typ: from_exp_type(&body.data.data),
            args: func
                .vars
                .0
                .iter()
                .zip(func.signature.argument_type.0.iter())
                .map(|(var, typ)| (var.clone(), from_type(typ)))
                .collect(),
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
                from_exp_type(&expr.data.data),
            ),
            "-" => Simple::Operation(
                Operator::Sub,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            "*" => Simple::Operation(
                Operator::Mul,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            "/" => Simple::Operation(
                Operator::Div,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            ">" => Simple::Operation(
                Operator::Greater,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            "<" => Simple::Operation(
                Operator::Less,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            ">=" => Simple::Operation(
                Operator::GreaterOrEqual,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            "<=" => Simple::Operation(
                Operator::LessOrEq,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            "==" => Simple::Operation(
                Operator::Equal,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            "!=" => Simple::Operation(
                Operator::NotEqual,
                from_typed_expr(&args.0[0], context).into(),
                from_typed_expr(&args.0[1], context).into(),
                from_exp_type(&expr.data.data),
            ),
            _ => match context.constructors.get(id) {
                Some(cons) => {
                    if args.0.is_empty() {
                        Simple::Int(cons.sibling_index as i64, from_exp_type(&expr.data.data))
                    } else {
                        Simple::Constructor(
                            cons.sibling_index as i64,
                            args.0
                                .iter()
                                .map(|arg| from_typed_expr(arg, context))
                                .collect(),
                            from_exp_type(&expr.data.data),
                        )
                    }
                }
                None => Simple::App(
                    id.clone(),
                    args.0
                        .iter()
                        .map(|arg| from_typed_expr(arg, context))
                        .collect(),
                    from_exp_type(&expr.data.data),
                ),
            },
        },
        scoped::SimplifiedExpression::Integer(i) => {
            Simple::Int(i.clone(), from_exp_type(&expr.data.data))
        }
        scoped::SimplifiedExpression::Variable(id) => {
            Simple::Ident(id.clone(), from_exp_type(&expr.data.data))
        }
        scoped::SimplifiedExpression::Match(var_node, cases) => Simple::Match(
            Simple::Ident(var_node.expr.clone(), from_exp_type(&expr.data.data)).into(),
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
                                                .enumerate()
                                                .map(|(i, var)| {
                                                    Binder::Variable(var.clone(), {
                                                        context.constructors.get(fid).unwrap();
                                                        from_type(
                                                            &context
                                                                .constructors
                                                                .get(fid)
                                                                .unwrap()
                                                                .args
                                                                .0[i],
                                                        )
                                                    })
                                                })
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
            from_exp_type(&expr.data.data),
        ),
        scoped::SimplifiedExpression::UTuple(args) => Simple::UTuple(
            args.0
                .iter()
                .map(|arg| from_typed_expr(arg, context))
                .collect(),
            from_exp_type(&expr.data.data),
        ),
        scoped::SimplifiedExpression::LetEqualIn(bindings, exp, next) if bindings.0.len() == 1 => {
            Simple::Let(
                bindings.0[0].clone(),
                from_typed_expr(exp, context).into(),
                from_typed_expr(next, context).into(),
                from_exp_type(&expr.data.data),
            )
        }
        scoped::SimplifiedExpression::LetEqualIn(bindings, exp, next) => Simple::LetApp(
            bindings.0.clone(),
            from_typed_expr(exp, context).into(),
            from_typed_expr(next, context).into(),
            from_exp_type(&expr.data.data),
        ),
    }
}

fn from_exp_type(typ: &ExpressionType) -> Type {
    match typ {
        ExpressionType::UTuple(vec) => Type::Unboxed(vec.0.iter().map(from_type).collect()),
        ExpressionType::Type(typ) => from_type(&typ),
    }
}

fn from_type(typ: &ast::Type) -> Type {
    match typ {
        ast::Type::Int => Type::Int,
        ast::Type::ADT(_) => Type::Heaped,
    }
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

fn get_type(expr: &Simple) -> Type {
    match expr {
        Simple::Ident(_, typ) => typ.clone(),
        Simple::Int(_, typ) => typ.clone(),
        Simple::Operation(_, _, _, typ) => typ.clone(),
        Simple::Constructor(_, _, typ) => typ.clone(),
        Simple::App(_, _, typ) => typ.clone(),
        Simple::Match(_, _, typ) => typ.clone(),
        Simple::Let(_, _, _, typ) => typ.clone(),
        Simple::UTuple(_, typ) => typ.clone(),
        Simple::LetApp(_, _, _, typ) => typ.clone(),
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

fn evaluate_reuse_in_case(var: Var, len: u8, body: &Body) -> Body {
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
                Body::Let((fresh, Type::Heaped), Exp::Reset(var), try_replace.into())
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
                Exp::Reuse((var, Type::Heaped), *tag, vars.clone()),
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
            fip: func.fip,
            id: func.id.clone(),
            typ: func.typ.clone(),
            args: func.args.clone(),
            body: if func.fip {
                reuse_all_matches(&func.body)
            } else {
                func.body.clone()
            },
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
    let ownership = get_ownership(prog);
    if flag {
        prog.iter()
            .map(|func| insert_rc_fun(func, &ownership))
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
        fip: func.fip,
        id: func.id.clone(),
        typ: func.typ.clone(),
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
            Exp::Proj(_, proj_var)
                if default_betal(proj_var, betal) == Status::Owned && var.1 == Type::Heaped =>
            {
                Body::Let(
                    var.clone(),
                    exp.clone(),
                    Body::Inc(
                        var.clone(),
                        owned_minus(proj_var, &insert_rc_body(next, betal, beta_map), betal).into(),
                    )
                    .into(),
                )
            }
            Exp::Proj(_, proj_var)
                if default_betal(proj_var, betal) == Status::Borrowed || var.1 != Type::Heaped =>
            {
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
            _ => panic!("Shouldn't be possible!"),
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
    } else if var.1 == Type::Heaped {
        Body::Inc(var.clone(), body.clone().into())
    } else {
        body.clone()
    }
}

fn owned_minus(var: &Var, body: &Body, betal: &HashMap<Var, Status>) -> Body {
    if default_betal(var, betal) == Status::Owned
        && !free_vars(body).contains(var)
        && var.1 != Type::Int
    {
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
