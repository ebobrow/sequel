use bytes::Bytes;

use super::token::Token;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Select {
        key: Key,
        table: Token,
    },
    Insert {
        table: Token,
        cols: Tokens,
        values: Vec<LiteralValue>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralValue {
    String(String),
    Number(f64),
}

// TODO: do we want this
impl From<&LiteralValue> for Bytes {
    fn from(val: &LiteralValue) -> Self {
        match val {
            LiteralValue::String(s) => Bytes::copy_from_slice(s[..].as_bytes()),
            LiteralValue::Number(n) => Bytes::from(n.to_string()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Key {
    Glob,
    List(Vec<Token>),
}

#[derive(Debug, PartialEq)]
pub enum Tokens {
    Omitted,
    List(Vec<Token>),
}
