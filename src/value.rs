//! Scheme values: the data programs work with and the code they are read as.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::symbol::Symbol;
use crate::vm::OpCode;

/// A Scheme value.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Value {
    /// Nil, written `()`: a special value, not a list.
    Nil,

    /// An integer.
    Number(i32),

    /// A symbol, such as `define` or `x`.
    Symbol(Symbol),

    /// A proper list. It is never empty: `()` is [`Nil`](Value::Nil).
    List(Vec<Value>),
    Func(Func),
}

#[derive(Clone, Debug)]
pub enum Func {
    Closure {
        env_pointer: usize,
        proto: Rc<Proto>,
    },
    NativeFunc, // TODO
}

/// Procedures are compared by identity: a closure is equal only to itself
/// and its copies.
impl PartialEq for Func {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Func::Closure { env_pointer, proto },
                Func::Closure {
                    env_pointer: other_env_pointer,
                    proto: other_proto,
                },
            ) => env_pointer == other_env_pointer && Rc::ptr_eq(proto, other_proto),
            (Func::NativeFunc, Func::NativeFunc) => true,
            _ => false,
        }
    }
}

impl Eq for Func {}

impl Hash for Func {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Func::Closure { env_pointer, proto } => {
                env_pointer.hash(state);
                Rc::as_ptr(proto).hash(state);
            }
            Func::NativeFunc => {}
        }
    }
}

/// The arity and code of a compiled `lambda`, shared by its closures.
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Proto {
    pub(crate) arity: usize,
    pub(crate) code: Vec<OpCode>,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => f.write_str("()"),
            Value::Number(number) => write!(f, "{number}"),
            Value::Symbol(symbol) => write!(f, "{symbol}"),
            Value::List(items) => {
                f.write_str("(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        f.write_str(" ")?;
                    }
                    write!(f, "{item}")?;
                }
                f.write_str(")")
            }
            Value::Func(_) => f.write_str("#<procedure>"),
        }
    }
}
