use crate::symbol::Symbol;

#[derive(Clone, Hash, PartialEq, Eq)]
pub(crate) enum Value {
    Nil,
    Number(i32),
    Symbol(Symbol),
    List(Vec<Value>),
}
