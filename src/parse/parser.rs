use super::{
    ast::{Expr, Key, LiteralValue},
    token::Token,
    ParseError, ParseResult,
};

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
        match cur {
            Token::Insert => self.insert(),
            Token::Select => self.select(),
            _ => Err(ParseError::Unexpected {
                expected: vec![Token::Insert, Token::Select],
                got: cur.clone(),
            }),
        }
    }

    fn insert(&mut self) -> ParseResult<Expr> {
        self.consume(&Token::Into)?;
        let table = self.consume_ident()?.clone();
        self.consume(&Token::LeftParen)?;
        let cols = self.key()?;
        self.consume(&Token::RightParen)?;
        self.consume(&Token::Values)?;
        let values = self.values()?;
        Ok(Expr::Insert {
            table,
            cols,
            values,
        })
    }

    fn select(&mut self) -> ParseResult<Expr> {
        let key = self.key()?;
        self.consume(&Token::From)?;
        let table = self.consume_ident()?.clone();
        Ok(Expr::Select { key, table })
    }

    fn key(&mut self) -> ParseResult<Key> {
        if self.peek()? == &Token::Star {
            self.advance()?;
            Ok(Key::Glob)
        } else {
            let first = self.consume_ident()?.clone();
            let mut keys = vec![first];
            while self.consume(&Token::Comma).is_ok() {
                keys.push(self.consume_ident()?.clone())
            }
            Ok(Key::List(keys))
        }
    }

    fn values(&mut self) -> ParseResult<Vec<LiteralValue>> {
        self.consume(&Token::LeftParen)?;
        let first = self.literal()?;
        let mut values = vec![first];
        while self.consume(&Token::Comma).is_ok() {
            values.push(self.literal()?);
        }
        self.consume(&Token::RightParen)?;
        Ok(values)
    }

    fn literal(&mut self) -> ParseResult<LiteralValue> {
        let tok = self.advance()?;
        match tok {
            Token::Number(n) => Ok(LiteralValue::Number(*n)),
            Token::String(s) => Ok(LiteralValue::String(s.clone())),
            _ => Err(ParseError::Unexpected {
                expected: vec![Token::Number(0.0), Token::String(String::new())],
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

    fn consume(&mut self, ty: &Token) -> ParseResult<&Token> {
        let next = self.peek()?;
        if next == ty {
            self.advance()
        } else {
            Err(ParseError::Unexpected {
                expected: vec![ty.clone()],
                got: next.clone(),
            })
        }
    }

    fn consume_ident(&mut self) -> ParseResult<&Token> {
        let next = self.peek()?;
        if let Token::Identifier(_) = next {
            self.advance()
        } else {
            Err(ParseError::Unexpected {
                expected: vec![Token::Identifier(String::new())],
                got: next.clone(),
            })
        }
    }
}
