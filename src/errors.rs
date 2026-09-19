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
