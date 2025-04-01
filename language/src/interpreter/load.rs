use super::iast::*;
use crate::ir::*;
use std::iter::Peekable;

fn extract_ifs<'a, T: Iterator<Item = &'a Statement>>(
    statements: &mut Peekable<T>,
) -> Vec<(IOperand, Vec<IStatement>)> {
    let mut chain = Vec::new();

    loop {
        let condition = match statements.next().unwrap() {
            Statement::If(operand) | Statement::ElseIf(operand) => {
                let body = extract_body(statements);
                (IOperand::from_op(operand), body)
            }
            Statement::Else => {
                let body = extract_body(statements);
                (IOperand::Int(1), body)
            }
            _ => panic!("yolo"),
        };

        chain.push(condition);

        match statements.peek() {
            Some(Statement::ElseIf(_)) | Some(Statement::Else) => {}
            _ => {
                break;
            }
        }
    }

    chain
}

pub fn extract_body<'a, T: Iterator<Item = &'a Statement>>(
    statements: &mut Peekable<T>,
) -> Vec<IStatement> {
    let mut istatements = Vec::new();
    while let Some(statement) = statements.peek() {
        let x = match statement {
            Statement::If(_) => IStatement::IfExpr(extract_ifs(statements)),
            Statement::ElseIf(_) | Statement::Else => panic!("this should not happen"),
            Statement::EndIf => {
                statements.next();
                break;
            }
            Statement::Decl(id) => {
                statements.next();
                IStatement::Decl(id.clone())
            }
            Statement::InitConstructor(id, i) => {
                statements.next();
                IStatement::InitConstructor(id.clone(), *i)
            }
            Statement::Return(operand) => {
                statements.next();
                IStatement::Return(IOperand::from_op(operand))
            }
            Statement::Print(operand) => {
                statements.next();
                IStatement::Print(IOperand::from_op(operand))
            }
            Statement::Inc(operand) => {
                statements.next();
                IStatement::Inc(IOperand::from_op(operand))
            }
            Statement::Dec(operand) => {
                statements.next();
                IStatement::Dec(IOperand::from_op(operand))
            }
            Statement::Assign(_, id, operand) => {
                statements.next();
                IStatement::Assign(id.clone(), IOperand::from_op(operand))
            }
            Statement::AssignToField(id, i, operand) => {
                statements.next();
                IStatement::AssignToField(id.clone(), *i, IOperand::from_op(operand))
            }
            Statement::AssignFromField(id, i, operand) => {
                statements.next();
                IStatement::AssignFromField(id.clone(), *i, IOperand::from_op(operand))
            }
            Statement::AssignBinaryOperation(id, operator, operand, operand1) => {
                statements.next();
                IStatement::AssignBinaryOperation(
                    id.clone(),
                    operator.clone(),
                    IOperand::from_op(operand),
                    IOperand::from_op(operand1),
                )
            }
            Statement::AssignConditional(id, b, operand, i) => {
                statements.next();
                IStatement::AssignConditional(id.clone(), *b, IOperand::from_op(operand), *i)
            }
            Statement::AssignFunctionCall(id, f, operands) => {
                statements.next();
                // first add a function call that puts the returned value in a register
                istatements.push(IStatement::FunctionCall(
                    f.clone(),
                    operands.iter().map(IOperand::from_op).collect(),
                ));
                // then assign the value to the identifier
                IStatement::AssignReturnvalue(id.clone())
            }
        };
        istatements.push(x);
    }

    istatements
}
