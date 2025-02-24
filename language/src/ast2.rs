use std::collections::HashMap;

type AID = usize;
type VID = usize;
type FID = usize;

struct ScopedNode<T> {
    node: T,
    var_tbl: HashMap<VID, VariableDefinition>
}

struct VariableDefinition {}

struct Program {
    adts: HashMap<AID, ADT>,
    constructors: HashMap<FID, ADTConstructor>,
    functions: HashMap<FID, Function>
}

struct ADT {}
struct ADTConstructor {}

struct Function {
    body: ScopedNode<Expression>
}

struct Expression {}