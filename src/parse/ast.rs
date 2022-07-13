use super::token::{Literal, Token};

#[derive(Debug, PartialEq)]
pub enum Expr {
    Select {
        key: Key,
        table: Token,
    },
    Insert {
        table: Token,
        cols: Key,
        values: Vec<Literal>,
    },
}

// TODO: better name
#[derive(Debug, PartialEq)]
pub enum Key {
    Glob,
    List(Vec<Token>),
}
