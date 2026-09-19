//! Parser: reads Scheme expressions.
//!
//! Built on [`chumsky`].

use chumsky::input::ValueInput;
use chumsky::prelude::*;

use crate::lexer::Token;
use crate::symbol::Symbol;
use crate::value::Value;

/// Parses expressions until the end of input.
pub(crate) fn program<'a, I>() -> impl Parser<'a, I, Vec<Value>, extra::Err<Rich<'a, Token>>>
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
