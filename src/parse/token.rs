use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Star,
    LeftParen,
    RightParen,
    Comma,
    GreaterThan,
    LessThan,
    Equal,
    GreaterEqual,
    LessEqual,

    And,
    Or,

    Insert,
    Select,
    From,
    Into,
    Values,
    Create,
    Table,

    Not,
    Null,
    Unique,
    Primary,
    Foreign,
    Key,
    Check,
    Default,
    Index,

    Identifier(String),
    Number(f64),
    String(String),
    Bool(bool),

    EOF,
}

impl Token {
    // TODO: this use Result too?
    pub fn ident(&self) -> Option<&String> {
        if let Token::Identifier(ident) = self {
            Some(ident)
        } else {
            None
        }
    }

    pub fn number(&self) -> Result<f64> {
        if let Token::Number(n) = self {
            Ok(*n)
        } else {
            Err(anyhow!("Expected number"))
        }
    }
}
