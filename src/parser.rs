//! Parser: reads Scheme expressions.
//!
//! Built on [`chumsky`].

use chumsky::input::{Stream, ValueInput};
use chumsky::prelude::*;
use logos::Logos;

use crate::errors::Error;
use crate::lexer::Token;
use crate::symbol::Symbol;
use crate::value::Value;

/// Reads every expression in `source`, reporting only the first error.
pub(crate) fn read(source: &str) -> Result<Vec<Value>, Error> {
    let tokens = Token::lexer(source)
        .spanned()
        .map(|(token, span)| {
            let span = SimpleSpan::from(span);
            token
                .map(|token| (token, span))
                .map_err(|()| Error::Lex { span })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let end = SimpleSpan::from(source.len()..source.len());
    let input = Stream::from_iter(tokens).map(end, |(token, span)| (token, span));
    program().parse(input).into_result().map_err(|errors| {
        let error = &errors[0];
        Error::Parse {
            span: *error.span(),
            message: error.reason().to_string(),
        }
    })
}

/// Parses expressions until the end of input.
fn program<'a, I>() -> impl Parser<'a, I, Vec<Value>, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    expression().repeated().collect()
}

/// Parses one expression: an atom, a list or a quoted expression.
///
/// `'x` is read as `(quote x)` and `()` as [`Value::Nil`]. A dotted list is
/// kept only when its tail is a list, as in `(a . (b))`; improper lists such
/// as `(a . b)` are reported as errors.
fn expression<'a, I>() -> impl Parser<'a, I, Value, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    let atom = select! {
        Token::Symbol(symbol) => Value::Symbol(symbol),
        Token::Number(number) => Value::Number(number),
        Token::Bool(boolean) => Value::Bool(boolean),
    };

    recursive(|s_expression| {
        let quoted = just(Token::Quote)
            .then(s_expression.clone())
            .map(|(_, expr)| Value::List(vec![Value::Symbol(Symbol::from("quote")), expr]));

        let tail = just(Token::Dot).ignore_then(s_expression.clone());
        let list = s_expression
            .repeated()
            .at_least(1)
            .collect::<Vec<_>>()
            .then(tail.or_not())
            .or_not()
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .validate(|list, e, emitter| match list {
                None => Value::Nil,
                Some((items, None | Some(Value::Nil))) => Value::List(items),
                Some((mut items, Some(Value::List(tail)))) => {
                    items.extend(tail);
                    Value::List(items)
                }
                Some((items, Some(_))) => {
                    emitter.emit(Rich::custom(e.span(), "improper lists are not supported"));
                    Value::List(items)
                }
            });

        atom.or(quoted).or(list)
    })
}
