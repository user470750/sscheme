//! Errors reported while reading Scheme code.

use chumsky::span::SimpleSpan;
use thiserror::Error;

/// An error in the source code, with its position.
#[derive(Error, Debug)]
pub(crate) enum Error {
    /// Input that is not a valid token.
    #[error("unexpected input at {span}")]
    Lex { span: SimpleSpan },

    /// Tokens that do not form an expression.
    #[error("{message} at {span}")]
    Parse { span: SimpleSpan, message: String },
}

#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("{exp_name} expression takes {expected} arguments but {passed} argument were passed.")]
    ArityError {
        exp_name: &'static str,
        expected: usize,
        passed: usize,
    },

    #[error("{exp_name} expected {expected}.")]
    WrongArgument {
        exp_name: &'static str,
        expected: &'static str,
    },

    #[error("It is bug. Empty list must be nil. Open issue.")]
    NilCombination,
}

#[derive(Error, Debug)]
pub enum InterpreterError {}
