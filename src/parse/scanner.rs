use bytes::Bytes;
use phf::phf_map;

use super::{
    token::{Literal, Token, TokenType},
    ParseError, ParseResult,
};

static KEYWORDS: phf::Map<&'static [u8], TokenType> = phf_map! {
    b"INSERT" => TokenType::Insert,
    b"SELECT" => TokenType::Select,
    b"FROM" => TokenType::From,
    b"INTO" => TokenType::Into,
    b"VALUES" => TokenType::Values,
};

pub struct Scanner {
    // TODO: again, Bytes or String
    source: Bytes,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
}

impl Scanner {
    pub fn scan(source: Bytes) -> ParseResult<Vec<Token>> {
        let mut scanner = Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
        };

        while !scanner.is_at_end() {
            scanner.start = scanner.current;
            scanner.scan_token()?;
        }
        scanner.tokens.push(Token::new(
            TokenType::EOF,
            Bytes::from_static(&[]),
            Literal::Null,
        ));

        Ok(scanner.tokens)
    }

    fn scan_token(&mut self) -> ParseResult<()> {
        match self.advance()? {
            b'*' => self.add_token(TokenType::Star, Literal::Null),
            b'"' => self.string()?,
            b'(' => self.add_token(TokenType::LeftParen, Literal::Null),
            b')' => self.add_token(TokenType::RightParen, Literal::Null),
            b',' => self.add_token(TokenType::Comma, Literal::Null),
            b' ' => {}
            c => {
                if c.is_ascii_digit() {
                    self.number()?;
                } else if c.is_ascii_alphabetic() {
                    self.identifier()?;
                } else {
                    return Err(ParseError::Unrecognized(*c));
                }
            }
        }
        Ok(())
    }

    fn add_token(&mut self, ty: TokenType, literal: Literal) {
        self.tokens.push(Token::new(
            ty,
            Bytes::copy_from_slice(&self.source[self.start..self.current]),
            literal,
        ));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> ParseResult<&u8> {
        self.current += 1;
        self.source
            .get(self.current - 1)
            .ok_or(ParseError::UnexpectedEnd)
    }

    fn peek(&self) -> ParseResult<&u8> {
        self.source
            .get(self.current)
            .ok_or(ParseError::UnexpectedEnd)
    }

    fn peek_next(&self) -> ParseResult<&u8> {
        self.source
            .get(self.current + 1)
            .ok_or(ParseError::UnexpectedEnd)
    }

    fn string(&mut self) -> ParseResult<()> {
        while self.peek()? != &b'"' {
            self.advance()?;
        }

        if self.is_at_end() {
            return Err(ParseError::UnexpectedEnd);
        }

        self.advance()?;
        self.add_token(
            TokenType::String,
            Literal::String(
                String::from_utf8(self.source[self.start + 1..self.current - 1].to_vec()).unwrap(),
            ),
        );
        Ok(())
    }

    fn number(&mut self) -> ParseResult<()> {
        while !self.is_at_end() && self.peek()?.is_ascii_digit() {
            self.advance()?;
        }

        if self.peek()? == &b'.' && self.peek_next()?.is_ascii_digit() {
            self.advance()?;
            while self.peek()?.is_ascii_digit() {
                self.advance()?;
            }
        }

        self.add_token(
            TokenType::Number,
            Literal::Number(
                std::str::from_utf8(&self.source[self.start..self.current])
                    .unwrap()
                    .parse()
                    .unwrap(),
            ),
        );
        Ok(())
    }

    fn identifier(&mut self) -> ParseResult<()> {
        while !self.is_at_end() && self.peek()?.is_ascii_alphabetic() {
            self.advance()?;
        }

        let text = &self.source[self.start..self.current];
        let ty = if let Some(ty) = KEYWORDS.get(text) {
            ty.clone()
        } else {
            TokenType::Identifier
        };
        self.add_token(ty, Literal::Null);
        Ok(())
    }
}
