use anyhow::{anyhow, bail, Result};

use crate::{parse::error::ERROR_EOF, Ty};

use super::{
    ast::{ColDecl, Command, Constraint, Expr, Key, LiteralValue, Tokens},
    error::throw_unexpected,
    token::Token,
    TableDef,
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Command> {
        self.command()
    }

    fn command(&mut self) -> Result<Command> {
        let cur = self.advance()?;
        match cur {
            Token::Insert => self.insert(),
            Token::Select => self.select(),
            Token::Create => self.create_table(),
            _ => throw_unexpected(cur, vec![Token::Insert, Token::Select]),
        }
    }

    fn insert(&mut self) -> Result<Command> {
        self.consume(&Token::Into)?;
        let table = self.consume_ident()?.clone();
        let cols = self.tokens()?;
        self.consume(&Token::Values)?;
        let rows = self.rows()?;
        Ok(Command::Insert { table, cols, rows })
    }

    fn select(&mut self) -> Result<Command> {
        let key = self.key()?;
        self.consume(&Token::From)?;
        let table = self.consume_ident()?.clone();
        Ok(Command::Select { key, table })
    }

    fn create_table(&mut self) -> Result<Command> {
        self.consume(&Token::Table)?;
        let name = self.consume_ident()?.clone();
        match self.peek()? {
            Token::LeftParen => {
                let col_decls = self.col_decls()?;
                Ok(Command::CreateTable {
                    name,
                    def: TableDef::Cols(col_decls),
                })
            }
            Token::As => {
                self.consume(&Token::As)?;
                self.consume(&Token::Select)?;
                let cmd = self.select()?;
                Ok(Command::CreateTable {
                    name,
                    def: TableDef::As(Box::new(cmd)),
                })
            }
            other => throw_unexpected(other, vec![Token::LeftParen, Token::As]),
        }
    }

    fn col_decls(&mut self) -> Result<Vec<ColDecl>> {
        self.consume(&Token::LeftParen)?;
        let first_ident = self.consume_ident()?.clone();
        let first_ty = self.ty()?;
        let mut decls = vec![ColDecl::new(first_ident, first_ty, self.constraints()?)];
        while self.consume(&Token::Comma).is_ok() {
            decls.push(ColDecl::new(
                self.consume_ident()?.clone(),
                self.ty()?,
                self.constraints()?,
            ));
        }
        self.consume(&Token::RightParen)?;
        Ok(decls)
    }

    fn constraints(&mut self) -> Result<Vec<Constraint>> {
        let mut constraints = Vec::new();
        loop {
            match self.peek()? {
                Token::Not => {
                    self.advance()?;
                    self.consume(&Token::Null)?;
                    constraints.push(Constraint::NotNull);
                }
                Token::Unique => {
                    self.advance()?;
                    constraints.push(Constraint::Unique);
                }
                Token::Primary => {
                    self.advance()?;
                    self.consume(&Token::Key)?;
                    constraints.push(Constraint::PrimaryKey);
                }
                Token::Foreign => {
                    self.advance()?;
                    self.consume(&Token::Key)?;
                    constraints.push(Constraint::ForeignKey);
                }
                Token::Check => {
                    self.advance()?;
                    self.consume(&Token::LeftParen)?;
                    let expr = self.expr()?;
                    self.consume(&Token::RightParen)?;
                    constraints.push(Constraint::Check(expr));
                }
                Token::Default => {
                    self.advance()?;
                    let lit = self.literal()?;
                    constraints.push(Constraint::Default(lit));
                }
                Token::Create => {
                    self.advance()?;
                    self.consume(&Token::Index)?;
                    constraints.push(Constraint::CreateIndex);
                }
                _ => return Ok(constraints),
            };
        }
    }

    fn expr(&mut self) -> Result<Expr> {
        let left = self.bin_side()?;
        let next = self.advance()?;
        let op = match next {
            Token::LessThan
            | Token::LessEqual
            | Token::GreaterThan
            | Token::GreaterEqual
            | Token::Equal => Ok(next.clone()),
            _ => throw_unexpected(
                next,
                vec![
                    Token::LessThan,
                    Token::LessEqual,
                    Token::GreaterThan,
                    Token::GreaterEqual,
                    Token::Equal,
                ],
            ),
        }?;
        let right = self.bin_side()?;
        Ok(Expr::Binary { left, op, right })
    }

    fn bin_side(&mut self) -> Result<Token> {
        let next = self.advance()?;
        match next {
            Token::String(_) | Token::Number(_) | Token::Identifier(_) => Ok(next.clone()),
            _ => throw_unexpected(
                next,
                vec![
                    Token::String(String::new()),
                    Token::Number(0.0),
                    Token::Identifier(String::new()),
                ],
            ),
        }
    }

    fn ty(&mut self) -> Result<Ty> {
        let name = self.consume_ident()?;
        Ok(
            match &name.ident().ok_or_else(|| anyhow!("expected ident"))?[..] {
                "string" => Ty::String,
                "number" => Ty::Number,
                other => bail!("unknown type {}", other),
            },
        )
    }

    fn key(&mut self) -> Result<Key> {
        if self.peek()? == &Token::Star {
            self.advance()?;
            Ok(Key::Glob)
        } else {
            let keys = self.token_list()?;
            Ok(Key::List(keys))
        }
    }

    fn tokens(&mut self) -> Result<Tokens> {
        if self.consume(&Token::LeftParen).is_ok() {
            let tokens = self.token_list()?;
            self.consume(&Token::RightParen)?;
            Ok(Tokens::List(tokens))
        } else {
            Ok(Tokens::Omitted)
        }
    }

    fn token_list(&mut self) -> Result<Vec<Token>> {
        let first = self.consume_ident()?.clone();
        let mut tokens = vec![first];
        while self.consume(&Token::Comma).is_ok() {
            tokens.push(self.consume_ident()?.clone())
        }
        Ok(tokens)
    }

    fn rows(&mut self) -> Result<Vec<Vec<LiteralValue>>> {
        let first = self.values()?;
        let mut rows = vec![first];
        while self.consume(&Token::Comma).is_ok() {
            rows.push(self.values()?);
        }
        Ok(rows)
    }

    fn values(&mut self) -> Result<Vec<LiteralValue>> {
        self.consume(&Token::LeftParen)?;
        let first = self.literal()?;
        let mut values = vec![first];
        while self.consume(&Token::Comma).is_ok() {
            values.push(self.literal()?);
        }
        self.consume(&Token::RightParen)?;
        Ok(values)
    }

    fn literal(&mut self) -> Result<LiteralValue> {
        let tok = self.advance()?;
        match tok {
            Token::Number(n) => Ok(LiteralValue::Number(ordered_float::OrderedFloat(*n))),
            Token::String(s) => Ok(LiteralValue::String(s.clone())),
            _ => throw_unexpected(tok, vec![Token::Number(0.0), Token::String(String::new())]),
        }
    }

    fn peek(&self) -> Result<&Token> {
        self.tokens
            .get(self.current)
            .ok_or_else(|| anyhow!(ERROR_EOF))
    }

    fn previous(&self) -> Result<&Token> {
        if self.current == 0 {
            Err(anyhow!("Internal error"))
        } else {
            self.tokens
                .get(self.current - 1)
                .ok_or_else(|| anyhow!(ERROR_EOF))
        }
    }

    fn advance(&mut self) -> Result<&Token> {
        self.current += 1;
        self.previous()
    }

    fn consume(&mut self, ty: &Token) -> Result<&Token> {
        let next = self.peek()?;
        if next == ty {
            self.advance()
        } else {
            throw_unexpected(next, vec![ty.clone()])
        }
    }

    fn consume_ident(&mut self) -> Result<&Token> {
        let next = self.peek()?;
        if let Token::Identifier(_) = next {
            self.advance()
        } else {
            throw_unexpected(next, vec![Token::Identifier(String::new())])
        }
    }
}
