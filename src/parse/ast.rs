use anyhow::{anyhow, Result};
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
        rows: Vec<Vec<LiteralValue>>,
    },
    CreateTable {
        name: Token,
        col_decls: Vec<ColDecl>,
    },
}

// TODO: real `Ty`s (varchar, etc.)
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Ty {
    String,
    Number,
}

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralValue {
    String(String),
    Number(f64),
}

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

#[derive(Debug, PartialEq)]
pub struct ColDecl {
    ident: Token,
    ty: Ty,
    constraints: Vec<Constraint>,
}

impl ColDecl {
    pub fn new(ident: Token, ty: Ty, constraints: Vec<Constraint>) -> Self {
        ColDecl {
            ident,
            ty,
            constraints,
        }
    }

    pub fn ident(&self) -> Result<&String> {
        self.ident.ident().ok_or_else(|| anyhow!("Internal error"))
    }

    pub fn ty(&self) -> &Ty {
        &self.ty
    }

    pub fn constraints(&self) -> &[Constraint] {
        self.constraints.as_ref()
    }
}

// TODO: default and check need params (parse accordingly)
#[derive(Debug, PartialEq, Eq)]
pub enum Constraint {
    NotNull,
    Unique,
    PrimaryKey,
    ForeignKey,
    Check,
    Default,
    CreateIndex,
}
