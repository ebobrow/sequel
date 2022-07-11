use std::fmt::Debug;

use super::{
    ast::{Expr, Key},
    token::{Literal, Token, TokenType},
};

pub enum ParseError {
    Unexpected {
        expected: Vec<TokenType>,
        got: Token,
    },
    UnexpectedEnd,
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
            Self::Internal => write!(f, "Internal error"),
        }
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> ParseResult<Expr> {
        self.command()
    }

    fn command(&mut self) -> ParseResult<Expr> {
        let cur = self.advance()?;
        match cur.ty() {
            TokenType::Insert => self.insert(),
            TokenType::Select => self.select(),
            _ => Err(ParseError::Unexpected {
                expected: vec![TokenType::Insert, TokenType::Select],
                got: cur.clone(),
            }),
        }
    }

    fn insert(&mut self) -> ParseResult<Expr> {
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

    fn select(&mut self) -> ParseResult<Expr> {
        let key = self.key()?;
        self.consume(&TokenType::From)?;
        let table = self.consume(&TokenType::Identifier)?.clone();
        Ok(Expr::Select { key, table })
    }

    fn key(&mut self) -> ParseResult<Key> {
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

    fn values(&mut self) -> ParseResult<Vec<Literal>> {
        self.consume(&TokenType::LeftParen)?;
        let first = self.literal()?;
        let mut values = vec![first];
        while self.consume(&TokenType::Comma).is_ok() {
            values.push(self.literal()?);
        }
        self.consume(&TokenType::RightParen)?;
        Ok(values)
    }

    fn literal(&mut self) -> ParseResult<Literal> {
        let tok = self.advance()?;
        match tok.ty() {
            TokenType::Number => Ok(tok.literal().clone()),
            TokenType::String => Ok(tok.literal().clone()),
            _ => Err(ParseError::Unexpected {
                expected: vec![TokenType::Number, TokenType::String],
                got: tok.clone(),
            }),
        }
    }

    fn peek(&self) -> ParseResult<&Token> {
        self.tokens
            .get(self.current)
            .ok_or(ParseError::UnexpectedEnd)
    }

    fn previous(&self) -> ParseResult<&Token> {
        if self.current == 0 {
            Err(ParseError::Internal)
        } else {
            self.tokens
                .get(self.current - 1)
                .ok_or(ParseError::UnexpectedEnd)
        }
    }

    fn advance(&mut self) -> ParseResult<&Token> {
        self.current += 1;
        self.previous()
    }

    fn consume(&mut self, ty: &TokenType) -> ParseResult<&Token> {
        let next = self.peek()?;
        if next.ty() == ty {
            self.advance()
        } else {
            Err(ParseError::Unexpected {
                expected: vec![ty.clone()],
                got: next.clone(),
            })
        }
    }
}
