// A program consists of a list of function definitions
pub struct Program(Vec<ADTDefinition>, Vec<FunctionDefinition>);

pub struct FID(String); // Function ID, (also including ADT constructors)
pub struct VID(String); // Variable ID
pub struct AID(String); // ADT ID

struct ADTDefinition {
    id: AID,
    constructors: Vec<ConstructorDefinition> 
}

struct ConstructorDefinition {
    id: VID,
    argument: TupleType
}

pub struct TupleType(Vec<Type>); // Type of a tuple is a list of other types
// Any general type
enum Type {
    Tuple(TupleType),
    Int,
    ADT(AID)
}

struct TypeSignature {
    argument_type: TupleType,
    result_type: Type,
    is_fip: bool
}

struct FunctionDefinition {
    id: FID,
    body: Expression,
    signature: TypeSignature
}

// Expressions are something that has a result, for example
// 1
// 4 * 5
// Cons (0, Nil)
// (1, 5, Nil)
enum Expression {
    Tuple(TupleExpression),
    FunctionCall(FID, Box<TupleExpression>),
    Integer(i32),
    Match(MatchExpression),

    Equals()
}

enum Operator {
    Equal, NotEqual, Less, LessOrEq, Greater, GreaterOrEqual,
    Add, Sub, Mul, Div 
}

// An expression to create a tuple is a list of other expressions
struct TupleExpression(Vec<Expression>);

struct MatchExpression {
    variable: VID,
    cases: Vec<MatchCase>
}

// A case in a match statement
struct MatchCase {
    constructor_id: FID,
    variables: Vec<VID>,
    body: Expression
}