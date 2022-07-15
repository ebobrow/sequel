use super::token::Token;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Select {
        key: Key,
        table: Token,
    },
    Insert {
        table: Token,
        cols: Key,
        values: Vec<LiteralValue>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralValue {
    String(String),
    Number(f64),
}

// TODO: better name
#[derive(Debug, PartialEq)]
pub enum Key {
    Glob,
    List(Vec<Token>),
}
