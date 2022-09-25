use anyhow::{anyhow, bail, Ok, Result};
use bytes::Bytes;
use ordered_float::OrderedFloat;

use crate::db::Column;

use super::token::Token;

#[derive(Debug, PartialEq)]
pub enum Command {
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

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Binary {
        left: Token,
        op: Token,
        right: Token,
    },
}

impl Expr {
    pub fn eval(&self, env: &[Column]) -> Result<LiteralValue> {
        match self {
            Expr::Binary { left, op, right } => {
                // don't collapse
                let extract_value = |tok: &Token| -> Result<LiteralValue> {
                    match tok {
                        Token::Identifier(ident) => {
                            Ok(env
                                .iter()
                                .find(|item| item.name() == ident)
                                // TODO: check this when first creating table?
                                .ok_or_else(|| anyhow!("Invalid identifier in check condition"))?
                                .data()
                                .clone())
                        }
                        Token::String(s) => Ok(LiteralValue::String(s.into())),
                        Token::Number(n) => Ok(LiteralValue::Number(OrderedFloat(*n))),
                        _ => bail!("expected identifier or literal value"),
                    }
                };
                let left = extract_value(left)?;
                let right = extract_value(right)?;
                Ok(LiteralValue::Bool(match op {
                    Token::GreaterThan => left.number()? > right.number()?,
                    Token::GreaterEqual => left.number()? >= right.number()?,
                    Token::Equal => left == right,
                    Token::LessThan => left.number()? < right.number()?,
                    Token::LessEqual => left.number()? <= right.number()?,
                    _ => unreachable!(),
                }))
            }
        }
    }
}

// TODO: real `Ty`s (varchar, etc.)
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Ty {
    String,
    Number,
    Bool,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum LiteralValue {
    String(String),
    Number(OrderedFloat<f64>),
    Bool(bool),
    Null,
}

impl From<&LiteralValue> for Bytes {
    fn from(val: &LiteralValue) -> Self {
        match val {
            LiteralValue::String(s) => Bytes::copy_from_slice(s[..].as_bytes()),
            LiteralValue::Number(n) => Bytes::from(n.to_string()),
            LiteralValue::Bool(b) => {
                if *b {
                    Bytes::from("true")
                } else {
                    Bytes::from("false")
                }
            }
            LiteralValue::Null => Bytes::new(),
        }
    }
}

impl LiteralValue {
    fn number(&self) -> Result<OrderedFloat<f64>> {
        if let LiteralValue::Number(n) = self {
            Ok(*n)
        } else {
            bail!("Expected number")
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
#[derive(Debug, PartialEq)]
pub enum Constraint {
    NotNull,
    Unique,
    PrimaryKey,
    ForeignKey,
    Check(Expr),
    Default(LiteralValue),
    CreateIndex,
}
