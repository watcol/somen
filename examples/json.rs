//! JSON parser implementation.
use somen::{call, prelude::*};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
enum JsonValue {
    Null,
    Boolean(bool),
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
    Number(f64),
    String(String),
}

fn spaces<'a, I: Input<Ok = char> + 'a>() -> impl Parser<I, Output = ()> + 'a {
    one_of(" \t\n\r")
        .expect("a space")
        .repeat(..)
        .discard()
        .expect("spaces")
}

fn null<'a, I: Input<Ok = char> + 'a>() -> impl Parser<I, Output = ()> + 'a {
    tag("null").discard().expect("a null")
}

fn boolean<'a, I: Input<Ok = char> + 'a>() -> impl Parser<I, Output = bool> + 'a {
    choice((tag("true").map(|_| true), tag("false").map(|_| false))).expect("a boolean")
}

fn object<'a, I: Input<Ok = char> + 'a>() -> impl Parser<I, Output = HashMap<String, JsonValue>> + 'a
{
    (
        string().skip((spaces(), token(':'), spaces())),
        call!(json_value).skip(spaces()),
    )
        .sep_by(token(',').skip(spaces()), ..)
        .collect::<HashMap<String, JsonValue>>()
        .between(token('{').skip(spaces()), token('}'))
        .expect("a object")
}

fn array<'a, I: Input<Ok = char> + 'a>() -> impl Parser<I, Output = Vec<JsonValue>> + 'a {
    call!(json_value)
        .skip(spaces())
        .sep_by(token(',').skip(spaces()), ..)
        .collect::<Vec<_>>()
        .between(token('[').skip(spaces()), token(']'))
        .expect("an array")
}

fn number<'a, I: Input<Ok = char> + 'a>() -> impl Parser<I, Output = f64> + 'a {
    (
        token('-').once().opt(),
        choice_iterable((
            token('0').once(),
            (
                one_of("123456789").expect("a non-zero digit").once(),
                one_of("0123456789").expect("a digit").repeat(..),
            ),
        )),
        (token('.').once(), one_of("0123456789").repeat(1..)).opt(),
        (
            one_of("eE").expect("e").once(),
            one_of("+-").once().opt(),
            one_of("0123456789").repeat(1..),
        )
            .opt(),
    )
        .collect::<String>()
        .try_map(|n| n.parse::<f64>().map_err(|_| "a valid number"))
        .expect("a number")
}

fn string<'a, I: Input<Ok = char> + 'a>() -> impl Parser<I, Output = String> + 'a {
    choice((
        none_of("\\\""),
        token('\\').prefix(one_of("\"\\/bfnrtu")).then(|c| match c {
            '\"' => value('\"').left(),
            '\\' => value('\\').left(),
            '/' => value('/').left(),
            'b' => value('\x08').left(),
            'f' => value('\x0c').left(),
            'n' => value('\n').left(),
            'r' => value('\r').left(),
            't' => value('\t').left(),
            'u' => one_of("0123456789abcdefABCDEF")
                .expect("a hex digit")
                .times(4)
                .collect::<String>()
                .try_map(|s| {
                    char::from_u32(u32::from_str_radix(&s, 16).unwrap())
                        .ok_or("a valid unicode codepoint")
                })
                .right(),
            _ => unreachable!(),
        }),
    ))
    .repeat(..)
    .collect::<String>()
    .between(token('"'), token('"'))
    .expect("a string")
}

fn json_value<'a, I: Input<Ok = char> + 'a>() -> impl Parser<I, Output = JsonValue> + 'a {
    choice((
        null().map(|_| JsonValue::Null),
        boolean().map(JsonValue::Boolean),
        object().map(JsonValue::Object),
        array().map(JsonValue::Array),
        number().map(JsonValue::Number),
        string().map(JsonValue::String),
    ))
    .expect("a value")
}

fn json<'a, I: Input<Ok = char> + 'a>() -> impl Parser<I, Output = JsonValue> + 'a {
    json_value().between(spaces(), spaces())
}

fn main() {
    futures::executor::block_on(async {
        let mut stream = stream::from_iter(
            r#"{
                "Image": {
                    "Width": 800,
                    "Height": 600,
                    "Title":  "View from 15th Floor",
                    "Thumbnail": {
                        "Url":    "http://www.example.com/image/481989943",
                        "Height": 125,
                        "Width":  100
                    },
                    "Animated" : false,
                    "IDs": [116, 943, 234, 38793]
                },
                "escaped characters": "\u2192\"\t\r\n"
            }"#
            .chars(),
        )
        .buffered_rewind();
        println!("{:#?}", json().parse(&mut stream).await);
    });
}
