use crate::symbol::Symbol;
use logos::{Lexer, Logos};

#[derive(Logos, Debug, PartialEq, Eq, Hash, Clone)]
#[logos(skip r"[ \t\n\r]+")]
pub(crate) enum Token {
    #[regex("-?[0-9]+", |lex| lex.slice().parse().ok(), priority=3)]
    Number(i32),

    #[regex("[0-9A-Za-z_\\-+=*^/]+", intern_symbol)]
    Symbol(Symbol),

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token(".")]
    Dot,

    #[regex("['`]")]
    Quote,
}

fn intern_symbol(lex: &mut Lexer<Token>) -> Option<Symbol> {
    let interned: Symbol = lex.slice().into();
    Some(interned)
}
