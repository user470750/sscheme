//! Errors reported while reading, compiling and running Scheme code.

use chumsky::span::SimpleSpan;
use thiserror::Error;

use crate::symbol::Symbol;

/// Any error reported by [`crate::Interpreter::eval`].
#[derive(Error, Debug)]
pub enum Error {
    /// Input that is not a valid token.
    #[error("unexpected input at {span}")]
    Lex { span: SimpleSpan },

    /// Tokens that do not form an expression.
    #[error("{message} at {span}")]
    Parse { span: SimpleSpan, message: String },

    /// A malformed special form.
    #[error(transparent)]
    Compiler(#[from] CompilerError),

    /// An error while running compiled code.
    #[error(transparent)]
    Interpreter(#[from] InterpreterError),
}

#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("`{exp_name}` expects {expected} argument(s), got {passed}")]
    ArityError {
        exp_name: &'static str,
        expected: &'static str,
        passed: usize,
    },

    #[error("`{exp_name}` expects {expected}")]
    WrongArgument {
        exp_name: &'static str,
        expected: &'static str,
    },

    #[error("internal error: empty list was not read as `()`")]
    NilCombination,

    #[error("`{exp_name}` is only allowed at the top level")]
    NotTopLevel { exp_name: &'static str },
}

#[derive(Error, Debug)]
pub enum InterpreterError {
    #[error("unbound variable `{0}`")]
    UnboundVariable(Symbol),

    #[error("`{0}` is not a procedure")]
    NotProcedure(String),

    #[error("procedure expects {expected} argument(s), got {passed}")]
    ArityMismatch { expected: usize, passed: usize },
}
