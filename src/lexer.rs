//! Lexer: turns source text into a stream of [`Token`]s.
//!
//! Built on [`logos`].

use std::fmt;

use crate::symbol::Symbol;
use logos::{Lexer, Logos};

/// A lexical token of the Scheme source.
#[derive(Logos, Debug, PartialEq, Eq, Hash, Clone)]
#[logos(skip r"[ \t\n\r]+")]
pub(crate) enum Token {
    /// An integer literal.
    ///
    /// A literal that does not fit in `i32` is a lexing error.
    #[regex("-?[0-9]+", |lex| lex.slice().parse().ok(), priority=3)]
    Number(i32),

    /// An identifier, interned at lex time.
    ///
    /// Anything that is not a valid [`Number`](Token::Number) also lexes as a
    /// symbol, so `-` and `1x` are symbols.
    #[regex("[0-9A-Za-z_\\-+=*^/]+", intern_symbol)]
    Symbol(Symbol),

    /// `(`
    #[token("(")]
    LParen,

    /// `)`
    #[token(")")]
    RParen,

    /// `.` in a dotted pair, as in `(a . b)`.
    #[token(".")]
    Dot,

    /// Quote prefix.
    #[regex("['`]")]
    Quote,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Number(number) => write!(f, "{number}"),
            Token::Symbol(symbol) => write!(f, "{symbol}"),
            Token::LParen => f.write_str("("),
            Token::RParen => f.write_str(")"),
            Token::Dot => f.write_str("."),
            Token::Quote => f.write_str("'"),
        }
    }
}

/// Interns the current slice as a [`Symbol`].
fn intern_symbol(lex: &mut Lexer<Token>) -> Option<Symbol> {
    let interned: Symbol = lex.slice().into();
    Some(interned)
}
