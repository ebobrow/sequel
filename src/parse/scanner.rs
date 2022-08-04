use anyhow::{anyhow, bail, Result};
use bytes::Bytes;
use phf::phf_map;

use super::{error::ERROR_EOF, token::Token};

static KEYWORDS: phf::Map<&'static [u8], Token> = phf_map! {
    b"INSERT" => Token::Insert,
    b"SELECT" => Token::Select,
    b"FROM" => Token::From,
    b"INTO" => Token::Into,
    b"VALUES" => Token::Values,
};

pub struct Scanner {
    source: Bytes,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
}

impl Scanner {
    pub fn scan(source: Bytes) -> Result<Vec<Token>> {
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
        scanner.tokens.push(Token::EOF);

        Ok(scanner.tokens)
    }

    fn scan_token(&mut self) -> Result<()> {
        match self.advance()? {
            b'*' => self.add_token(Token::Star),
            b'"' => self.string()?,
            b'(' => self.add_token(Token::LeftParen),
            b')' => self.add_token(Token::RightParen),
            b',' => self.add_token(Token::Comma),
            b' ' => {}
            c => {
                if c.is_ascii_digit() {
                    self.number()?;
                } else if c.is_ascii_alphabetic() {
                    self.identifier()?;
                } else {
                    bail!("Unrecognized token {:?}", *c as char);
                }
            }
        }
        Ok(())
    }

    fn add_token(&mut self, tok: Token) {
        self.tokens.push(tok);
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> Result<&u8> {
        self.current += 1;
        self.source.get(self.current - 1).ok_or(anyhow!(ERROR_EOF))
    }

    fn peek(&self) -> Result<&u8> {
        self.source.get(self.current).ok_or(anyhow!(ERROR_EOF))
    }

    fn peek_next(&self) -> Result<&u8> {
        self.source.get(self.current + 1).ok_or(anyhow!(ERROR_EOF))
    }

    fn string(&mut self) -> Result<()> {
        while self.peek()? != &b'"' {
            self.advance()?;
        }

        if self.is_at_end() {
            bail!(ERROR_EOF);
        }

        self.advance()?;
        self.add_token(Token::String(String::from_utf8(
            self.source[self.start + 1..self.current - 1].to_vec(),
        )?));
        Ok(())
    }

    fn number(&mut self) -> Result<()> {
        while !self.is_at_end() && self.peek()?.is_ascii_digit() {
            self.advance()?;
        }

        if self.peek()? == &b'.' && self.peek_next()?.is_ascii_digit() {
            self.advance()?;
            while self.peek()?.is_ascii_digit() {
                self.advance()?;
            }
        }

        self.add_token(Token::Number(
            std::str::from_utf8(&self.source[self.start..self.current])?.parse()?,
        ));
        Ok(())
    }

    fn identifier(&mut self) -> Result<()> {
        while !self.is_at_end() && self.peek()?.is_ascii_alphabetic() {
            self.advance()?;
        }

        let text = &self.source[self.start..self.current];
        let ty = if let Some(ty) = KEYWORDS.get(text) {
            ty.clone()
        } else {
            Token::Identifier(String::from_utf8(
                self.source[self.start..self.current].to_vec(),
            )?)
        };
        self.add_token(ty);
        Ok(())
    }
}
