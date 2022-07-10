use super::token::{Literal, Token};

#[derive(Debug)]
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
#[derive(Debug)]
pub enum Key {
    Glob,
    List(Vec<Token>),
}
