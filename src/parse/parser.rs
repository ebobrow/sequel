use super::{
    ast::{Expr, Key},
    token::{Literal, Token, TokenType},
};

// TODO: this sucks (and remove `unwrap`s)
enum Error {
    Parse,
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
        self.command().ok()
    }

    fn command(&mut self) -> Result<Expr, Error> {
        match self.advance().ok_or(Error::Parse)?.ty() {
            TokenType::Insert => self.insert(),
            TokenType::Select => self.select(),
            _ => Err(Error::Parse),
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
        if self.peek().ok_or(Error::Parse)?.ty() == &TokenType::Star {
            self.advance();
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
        let tok = self.advance().ok_or(Error::Parse)?;
        match tok.ty() {
            TokenType::Number => Ok(tok.literal().clone()),
            TokenType::String => Ok(tok.literal().clone()),
            _ => Err(Error::Parse),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn previous(&self) -> Option<&Token> {
        if self.current == 0 {
            None
        } else {
            self.tokens.get(self.current - 1)
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().unwrap().ty() == &TokenType::EOF
    }

    fn advance(&mut self) -> Option<&Token> {
        if self.is_at_end() {
            None
        } else {
            self.current += 1;
            self.previous()
        }
    }

    fn consume(&mut self, ty: &TokenType) -> Result<&Token, Error> {
        if let Some(tok) = self.peek() {
            if tok.ty() == ty {
                Ok(self.advance().unwrap())
            } else {
                Err(Error::Parse)
            }
        } else {
            Err(Error::Parse)
        }
    }
}
