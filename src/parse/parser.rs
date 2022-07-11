use std::fmt::Debug;

use super::{
    ast::{Expr, Key},
    token::{Literal, Token, TokenType},
};

enum Error {
    Unexpected {
        expected: Vec<TokenType>,
        got: Token,
    },
    UnexpectedEnd,
    Internal,
}

impl Debug for Error {
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
            Self::Internal => write!(f, "Internal error"),
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Option<Expr> {
        match self.command() {
            Ok(expr) => Some(expr),
            Err(e) => {
                eprintln!("Error:\n{:?}", e);
                None
            }
        }
    }

    fn command(&mut self) -> Result<Expr, Error> {
        let cur = self.advance()?;
        match cur.ty() {
            TokenType::Insert => self.insert(),
            TokenType::Select => self.select(),
            _ => Err(Error::Unexpected {
                expected: vec![TokenType::Insert, TokenType::Select],
                got: cur.clone(),
            }),
        }
    }

    fn insert(&mut self) -> Result<Expr, Error> {
        self.consume(&TokenType::Into)?;
        let table = self.consume(&TokenType::Identifier)?.clone();
        self.consume(&TokenType::LeftParen)?;
        let cols = self.key()?;
        self.consume(&TokenType::RightParen)?;
        self.consume(&TokenType::Values)?;
        let values = self.values()?;
        Ok(Expr::Insert {
            table,
            cols,
            values,
        })
    }

    fn select(&mut self) -> Result<Expr, Error> {
        let key = self.key()?;
        self.consume(&TokenType::From)?;
        let table = self.consume(&TokenType::Identifier)?.clone();
        Ok(Expr::Select { key, table })
    }

    fn key(&mut self) -> Result<Key, Error> {
        if self.peek()?.ty() == &TokenType::Star {
            self.advance()?;
            Ok(Key::Glob)
        } else {
            let first = self.consume(&TokenType::Identifier)?.clone();
            let mut keys = vec![first];
            while self.consume(&TokenType::Comma).is_ok() {
                keys.push(self.consume(&TokenType::Identifier)?.clone());
            }
            Ok(Key::List(keys))
        }
    }

    fn values(&mut self) -> Result<Vec<Literal>, Error> {
        self.consume(&TokenType::LeftParen)?;
        let first = self.literal()?;
        let mut values = vec![first];
        while self.consume(&TokenType::Comma).is_ok() {
            values.push(self.literal()?);
        }
        self.consume(&TokenType::RightParen)?;
        Ok(values)
    }

    fn literal(&mut self) -> Result<Literal, Error> {
        let tok = self.advance()?;
        match tok.ty() {
            TokenType::Number => Ok(tok.literal().clone()),
            TokenType::String => Ok(tok.literal().clone()),
            _ => Err(Error::Unexpected {
                expected: vec![TokenType::Number, TokenType::String],
                got: tok.clone(),
            }),
        }
    }

    fn peek(&self) -> Result<&Token, Error> {
        self.tokens.get(self.current).ok_or(Error::UnexpectedEnd)
    }

    fn previous(&self) -> Result<&Token, Error> {
        if self.current == 0 {
            Err(Error::Internal)
        } else {
            self.tokens
                .get(self.current - 1)
                .ok_or(Error::UnexpectedEnd)
        }
    }

    fn advance(&mut self) -> Result<&Token, Error> {
        self.current += 1;
        self.previous()
    }

    fn consume(&mut self, ty: &TokenType) -> Result<&Token, Error> {
        let next = self.peek()?;
        if next.ty() == ty {
            self.advance()
        } else {
            Err(Error::Unexpected {
                expected: vec![ty.clone()],
                got: next.clone(),
            })
        }
    }
}
