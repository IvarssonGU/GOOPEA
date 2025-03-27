use std::{cell::RefCell, collections::{HashMap, HashSet}};

use crate::error::{CompileError, CompileResult};

use super::{ast::{transform_box, ExpressionNode, FullExpression, FunctionData, Pattern, Program, UTuple, FID, VID}, scoped::SimplifiedExpression, typed::{TypedData, TypedNode, TypedProgram}};

pub type FIPData = TypedData;

pub type FIPNode = ExpressionNode<FIPData, FIPExpression<FIPData>>;
pub type FIPProgram = Program<FIPData, FIPExpression<FIPData>>;

impl FIPProgram {
    pub fn new<'a>(program: TypedProgram) -> Result<Self, CompileError> {
        let constructors = program.constructors.keys().cloned().collect();

        let program = program.transform_functions(|body, _, _| {
            body.transform(|expr| Ok(expr.restore_constructors(&constructors)))
        })?;

        let program = program.transform_functions(|body, func, _| {
            if !func.signature.is_fip { return Ok(body) }
            
            let tokens = RefCell::new(HashMap::new());
            let new_body = body.fipify(&tokens)?;

            if tokens.borrow().len() > 0 {
                return Err(CompileError::FIPFunctionDeallocatesMemory);
            }

            Ok(new_body)
        })?;

        Ok(program)
    }
}

impl SimplifiedExpression<TypedData> {
    fn restore_constructors(self, constructors: &HashSet<FID>) -> FIPExpression<TypedData> {
        let func = |expr: Self| Ok(expr.restore_constructors(constructors));

        match self {
            SimplifiedExpression::UTuple(utuple) => FIPExpression::UTuple(utuple.transform_expressions(func).unwrap()),
            SimplifiedExpression::Integer(x) => FIPExpression::Integer(x),
            SimplifiedExpression::Variable(vid) => FIPExpression::Variable(vid),
            SimplifiedExpression::Match(e1, cases) => {
                FIPExpression::Match(
                    transform_box(e1, func).unwrap(), 
                    cases.into_iter().map(|(pattern, e)| (pattern, e.transform(func).unwrap())).collect()
                )
            },
            SimplifiedExpression::FunctionCall(fid, args) => {
                if constructors.contains(&fid) {
                    FIPExpression::FunctionCall(fid, args.transform_expressions(func).unwrap())
                } else {
                    if args.0.len() == 0 {
                        FIPExpression::Atom(fid)
                    } else {
                        FIPExpression::FreshConstructor(fid, args.transform_expressions(func).unwrap())
                    }
                }
            },
        }
    }
}

impl FIPNode {
    fn fipify(self, tokens_cell: &RefCell<HashMap<usize, Vec<usize>>>) -> Result<Self, CompileError> {
        let new_node = self.transform(|expr| {
            match expr {
                FIPExpression::UTuple(tup) => Ok(FIPExpression::UTuple(tup.transform_nodes(|e|e.fipify(tokens_cell))?)),
                FIPExpression::FunctionCall(fid, tup) => Ok(FIPExpression::FunctionCall(fid, tup.transform_nodes(|e|e.fipify(tokens_cell))?)),
                FIPExpression::Integer(_) | FIPExpression::Atom(_) | FIPExpression::Variable(_) => Ok(expr),
                FIPExpression::FreshConstructor(fid, args) => {
                    let size = args.0.len();

                    let mut tokens = tokens_cell.borrow_mut();
                    let Some(vars) = tokens.get_mut(&size) else {
                        return Err(CompileError::FIPFunctionAllocatesMemory);
                    };

                    let Some(reuse_var) = vars.pop() else {
                        return Err(CompileError::FIPFunctionAllocatesMemory);
                    };

                    if vars.len() == 0 {
                        tokens.remove(&size);
                    }

                    return Ok(FIPExpression::ReusedConstructor(fid, args.transform_nodes(|e| e.fipify(tokens_cell))?, reuse_var))
                },
                FIPExpression::ReusedConstructor(_, _, _) => unreachable!(),
                FIPExpression::Match(e1, cases) => {
                    /*let e1 = Box::new(e1.fipify(tokens_cell)?);

                    if let FIPExpression::Variable(vid) = e1.expr {

                    } else {
                        if 
                    }*/
                }
            }
        })?;

        Ok(new_node)
    }
}

#[derive(Debug)]
pub enum FIPExpression<D> {
    UTuple(UTuple<ExpressionNode<D, Self>>),
    FunctionCall(FID, UTuple<ExpressionNode<D, Self>>),
    Integer(i64),
    Variable(VID),
    Match(Box<ExpressionNode<D, Self>>, Vec<(Pattern, ExpressionNode<D, Self>)>),
    FreshConstructor(FID, UTuple<ExpressionNode<D, Self>>),
    ReusedConstructor(FID, UTuple<ExpressionNode<D, Self>>, usize),
    Atom(FID)
}

impl<'a, D> From<&'a FIPExpression<D>> for FullExpression<'a, D, FIPExpression<D>> {
    fn from(value: &'a FIPExpression<D>) -> Self {
        match value {
            FIPExpression::UTuple(x) => FullExpression::UTuple(x),
            FIPExpression::FunctionCall(x, y) |
            FIPExpression::FreshConstructor(x, y) |
            FIPExpression::ReusedConstructor(x, y, _) => FullExpression::FunctionCall(x, y),
            FIPExpression::Integer(x) => FullExpression::Integer(x),
            FIPExpression::Variable(x) => FullExpression::Variable(x),
            FIPExpression::Match(x, y) => FullExpression::Match(x, y),
            FIPExpression::Atom(x) => FullExpression::Atom(x),
        }
    }
}