use chumsky::input::ValueInput;
use chumsky::prelude::*;

use crate::lexer::Token;
use crate::symbol::Symbol;
use crate::value::Value;

pub fn parser<'a, I>() -> impl Parser<'a, I, Value, extra::Err<Rich<'a, Token>>>
where
    I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    let nil = just(Token::LParen)
        .then(just(Token::RParen))
        .map(|_| Value::Nil);

    let atom = select! {
        Token::Symbol(symbol) => Value::Symbol(symbol),
        Token::Number(number) => Value::Number(number),
    };

    recursive(|s_expression| {
        let quoted = just(Token::Quote)
            .then(s_expression.clone())
            .map(|(_, expr)| Value::List(vec![Value::Symbol(Symbol::from("quote")), expr]));

        let list = s_expression
            .clone()
            .repeated()
            .collect()
            .map(|exps: Vec<Value>| Value::List(exps))
            .delimited_by(just(Token::LParen), just(Token::RParen));

        let dot_notation = s_expression
            .clone()
            .then_ignore(just(Token::Dot))
            .then(s_expression)
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .map(|(car, cdr)| match cdr {
                Value::Nil => {
                    if let Value::List(pair) = car {
                        Value::List(pair)
                    } else {
                        Value::List(vec![car])
                    }
                }
                Value::List(mut pair) => {
                    pair.insert(0, car);
                    Value::List(pair)
                }
                _ => {
                    panic!("Cons list always ends with nil.");
                }
            });

        nil.or(atom).or(quoted).or(list).or(dot_notation)
    })
}
