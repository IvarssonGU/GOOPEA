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
