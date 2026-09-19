//! Scheme values: the data programs work with and the code they are read as.

use std::fmt;

use crate::symbol::Symbol;
use crate::vm::OpCode;

/// A Scheme value.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) enum Value {
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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Func {
    Closure {
        env_pointer: usize,
        arity: usize,
        code: Vec<OpCode>,
    },
    NativeFunc, // TODO
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
