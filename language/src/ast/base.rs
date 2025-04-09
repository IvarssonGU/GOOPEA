use std::{collections::{BTreeMap, HashMap, HashSet}, iter::once, ops::{Bound, Range}};

use crate::{error::CompileError, grammar, lexer::Lexer};
use color_eyre::Result;

use super::ast::{Constructor, ExpressionNode, FullExpression, FunctionData, Operator, Pattern, Program, Type, UTuple, AID, FID, VID};

pub type BaseSliceNode<'i> = ExpressionNode<&'i str, SyntaxExpression<&'i str>>;
pub type BaseSliceProgram<'i> = Program<&'i str, SyntaxExpression<&'i str>>;

pub type BaseRangeNode = ExpressionNode<Range<usize>, SyntaxExpression<Range<usize>>>;
pub type BaseRangeProgram = Program<Range<usize>, SyntaxExpression<Range<usize>>>;

#[derive(Debug)]
pub enum Definition {
    ADT(AID, Vec<(FID, UTuple<Type>)>),
    Function(FID, (FunctionData, BaseRangeNode))
}

pub struct SourceLocation {
    pub line: usize,
    pub char_offset: usize
}

pub struct SourceReference<'i> {
    pub start: SourceLocation,
    pub end: SourceLocation,
    pub snippet: &'i str
}

impl<'i> BaseSliceProgram<'i> {
    pub fn new(code: &'i str) -> Result<BaseSliceProgram<'i>> {
        let linebreaks = code.bytes().enumerate().filter_map(|(i, c)| (c == b'\n').then(|| i as isize)).chain(once(-1)).enumerate().map(|(i, c)| (c, i+1)).collect::<BTreeMap<_, _>>();

        let program: Vec<Definition> = grammar::ProgramParser::new().parse(Lexer::new(code)).unwrap();

        let builtin_defs = vec![
            Definition::ADT("Bool".to_string(), vec![("False".to_string(), UTuple::empty()), ("True".to_string(), UTuple::empty())])
        ];

        let mut adts = HashMap::new();
        let mut all_constructors = HashMap::new();
        let mut function_datas = HashMap::new();
        let mut function_bodies = HashMap::new();
        for def in program.into_iter().chain(builtin_defs.into_iter()) {
            match def {
                Definition::ADT(aid, constructors) => {
                    if adts.insert(aid.clone(), constructors.iter().map(|(fid, _)| fid.clone()).collect()).is_some() {
                        return Err(CompileError::MultipleADTDefinitions(aid.clone()).into())
                    }

    
                    for (sibling_index, (fid, args)) in constructors.into_iter().enumerate() {    
                        if all_constructors.insert(fid.clone(), Constructor { sibling_index, adt: aid.clone(), args }).is_some() {
                            return Err(CompileError::MultipleFunctionDefinitions(fid).into())
                        }
                    }
                },
                Definition::Function(fid, (data, body)) => {    
                    if function_datas.insert(fid.clone(), data).is_some() {
                        return Err(CompileError::MultipleFunctionDefinitions(fid).into())
                    }
                    function_bodies.insert(fid, body.make_slice(code, &linebreaks));
                }
            }
        }

        if let Some(fid) = function_datas.keys().collect::<HashSet<_>>().intersection(&all_constructors.keys().collect()).next() {
            return Err(CompileError::MultipleFunctionDefinitions((*fid).clone()).into())
        }

        if !function_datas.contains_key("main") {
            return Err(CompileError::MissingMainFunction.into())
        }

        let program = BaseSliceProgram { adts, constructors: all_constructors, function_datas, function_bodies };
        program.validate_all_types()?;

        Ok(program)
    }

    // Checks so that all types use defined ADT names
    fn validate_all_types(&self) -> Result<()> {
        for cons in self.constructors.values() {
            cons.args.validate_in(self)?;
        }

        for (_, func) in &self.function_datas {
            func.signature.argument_type.validate_in(self)?;
            func.signature.result_type.validate_in(self)?;
        }

        Ok(())
    }
}

impl BaseRangeNode {
    pub fn make_slice<'i>(self, code: &'i str, linebreaks: &BTreeMap<isize, usize>) -> BaseSliceNode<'i> {
        let new_expr = match self.expr {
            SyntaxExpression::UTuple(tup) => 
                SyntaxExpression::UTuple(tup.transform_nodes(|e| Ok(e.make_slice(code, linebreaks))).unwrap()),
            SyntaxExpression::FunctionCall(fid, tup) => 
                SyntaxExpression::FunctionCall(fid, tup.transform_nodes(|e| Ok(e.make_slice(code, linebreaks))).unwrap()),
            SyntaxExpression::Integer(x) => SyntaxExpression::Integer(x),
            SyntaxExpression::Variable(vid) => SyntaxExpression::Variable(vid),
            SyntaxExpression::Match(expr, cases) => 
                SyntaxExpression::Match(
                    Box::new(expr.make_slice(code, linebreaks)), 
                    cases.into_iter().map(|(pattern, e)| (pattern, e.make_slice(code, linebreaks))).collect()
                ),
            SyntaxExpression::LetEqualIn(tup, e1, e2) => 
                SyntaxExpression::LetEqualIn(tup, Box::new(e1.make_slice(code, linebreaks)), Box::new(e2.make_slice(code, linebreaks))),
            SyntaxExpression::Operation(e1, operator, e2) => 
                SyntaxExpression::Operation(Box::new(e1.make_slice(code, linebreaks)), operator, Box::new(e2.make_slice(code, linebreaks))),
        };

        let snippet = &code[self.data.clone()];

        let (start_line_start_char, start_line) = linebreaks.lower_bound(Bound::Included(&(self.data.start as isize))).prev().unwrap();
        let (end_line_start_char, end_line) = linebreaks.lower_bound(Bound::Included(&(self.data.end as isize))).prev().unwrap();

        let start = SourceLocation { line: *start_line, char_offset: self.data.start.checked_sub_signed(*start_line_start_char).unwrap() };
        let end = SourceLocation { line: *end_line, char_offset: self.data.end.checked_sub_signed(*end_line_start_char).unwrap() };

        let reference = SourceReference { end, start, snippet };

        BaseSliceNode {
            expr: new_expr,
            data: snippet
        }
    }
}

impl BaseRangeNode {
    pub fn integer(x: i64, location: Range<usize>) -> Self { Self::new(location,SyntaxExpression::Integer(x)) }

    pub fn variable(vid: VID, location: Range<usize>) -> Self { Self::new(location, SyntaxExpression::Variable(vid)) }

    pub fn function_call(fid: FID, args: UTuple<Self>, location: Range<usize>) -> Self {
        Self::new(location, SyntaxExpression::FunctionCall(fid, args))
    }

    pub fn operation(op: Operator, l: Self, r: Self, location: Range<usize>) -> Self {
        Self::new(location, SyntaxExpression::Operation(Box::new(l), op, Box::new(r)))
    }

    pub fn utuple(args: UTuple<Self>, location: Range<usize>) -> Self {
        Self::new(location, SyntaxExpression::UTuple(args))
    }

    pub fn mtch(match_on: Self, cases: Vec<(Pattern, Self)>, location: Range<usize>) -> Self {
        Self::new(location, SyntaxExpression::Match(Box::new(match_on), cases))
    }

    pub fn let_equal_in(vars: UTuple<VID>, e1: Self, e2: Self, location: Range<usize>) -> Self {
        Self::new(location, SyntaxExpression::LetEqualIn(vars, Box::new(e1), Box::new(e2)))
    }
}

impl Type {
    fn validate_in(&self, program: &BaseSliceProgram) -> Result<()> {
        match self {
            Type::Int => Ok(()),
            Type::ADT(aid) => {
                if !program.adts.contains_key(aid) { 
                    Err(CompileError::UnknownADTInType(aid.to_string()).into()) 
                } else { 
                    Ok(()) 
                }
            }
        }
    }
}

impl UTuple<Type> {
    fn validate_in(&self, program: &BaseSliceProgram) -> Result<()> {
        for tp in &self.0 { tp.validate_in(program)?; }
        Ok(())
    }
}

#[derive(Debug)]
pub enum SyntaxExpression<D> {
    UTuple(UTuple<ExpressionNode<D, Self>>),
    FunctionCall(FID, UTuple<ExpressionNode<D, Self>>),
    Integer(i64),
    Variable(VID),
    Match(Box<ExpressionNode<D, Self>>, Vec<(Pattern, ExpressionNode<D, Self>)>),
    LetEqualIn(UTuple<VID>, Box<ExpressionNode<D, Self>>, Box<ExpressionNode<D, Self>>),
    Operation(Box<ExpressionNode<D, Self>>, Operator, Box<ExpressionNode<D, Self>>)
}

impl<'a, D> From<&'a SyntaxExpression<D>> for FullExpression<'a, D, SyntaxExpression<D>> {
    fn from(value: &'a SyntaxExpression<D>) -> Self {
        match value {
            SyntaxExpression::UTuple(x) => FullExpression::UTuple(x),
            SyntaxExpression::FunctionCall(x, y) => FullExpression::FunctionCall(x, y),
            SyntaxExpression::Integer(x) => FullExpression::Integer(x),
            SyntaxExpression::Variable(x) => FullExpression::Variable(x),
            SyntaxExpression::Match(x, y) => FullExpression::MatchOnExpression(x, y),
            SyntaxExpression::LetEqualIn(x, y, z) => FullExpression::LetEqualIn(x, y, z),
            SyntaxExpression::Operation(x, y, z) => FullExpression::Operation(x, y, z)
        }
    }
}