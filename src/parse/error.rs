use std::fmt::Debug;

use super::token::{Token, TokenType};

pub enum ParseError {
    Unexpected {
        expected: Vec<TokenType>,
        got: Token,
    },
    UnexpectedEnd,
    Unrecognized(u8),
    Internal,
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unexpected { expected, got } => {
                let mut msg = format!("Expected one of: {:?}", expected[0]);
                for ty in &expected[1..] {
                    msg.push_str(&format!(", {:?}", ty)[..]);
                }
                write!(f, "{}\nGot: {:#?}", msg, got)
            }
            Self::UnexpectedEnd => write!(f, "Unexpected end of file"),
            Self::Unrecognized(c) => write!(f, "Unrecognized token {:?}", *c as char),
            Self::Internal => write!(f, "Internal error"),
        }
    }
}

pub type ParseResult<T> = Result<T, ParseError>;
