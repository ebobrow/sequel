use anyhow::Result;
use bytes::Bytes;

use parser::Parser;
use scanner::Scanner;

pub use self::{
    ast::{ColDecl, Command, Constraint, Key, LiteralValue, Tokens, Ty},
    token::Token,
};

mod ast;
mod error;
mod parser;
mod scanner;
mod token;

pub fn parse(stream: Bytes) -> Result<Command> {
    let tokens = Scanner::scan(stream)?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::parse::{
        ast::{Constraint, Expr},
        ColDecl,
    };

    use super::{
        ast::{Command, Key, Ty},
        error::ERROR_EOF,
        parser::Parser,
        scanner::Scanner,
        token::Token,
    };

    #[test]
    fn scanner() {
        let stream = "INSERT 17.6 * (\"one\", \"two\") table".into();
        let tokens = Scanner::scan(stream).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Insert,
                Token::Number(17.6),
                Token::Star,
                Token::LeftParen,
                Token::String("one".to_string()),
                Token::Comma,
                Token::String("two".to_string()),
                Token::RightParen,
                Token::Identifier(String::from("table")),
                Token::EOF,
            ]
        );

        let stream = "CREATE TABLE people (ID number PRIMARY KEY, FirstName string CHECK (FirstName >= \"Brian\"), Age number NOT NULL UNIQUE)".into();
        let tokens = Scanner::scan(stream).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Create,
                Token::Table,
                Token::Identifier("people".into()),
                Token::LeftParen,
                Token::Identifier("ID".into()),
                Token::Identifier("number".into()),
                Token::Primary,
                Token::Key,
                Token::Comma,
                Token::Identifier("FirstName".into()),
                Token::Identifier("string".into()),
                Token::Check,
                Token::LeftParen,
                Token::Identifier("FirstName".into()),
                Token::GreaterEqual,
                Token::String("Brian".into()),
                Token::RightParen,
                Token::Comma,
                Token::Identifier("Age".into()),
                Token::Identifier("number".into()),
                Token::Not,
                Token::Null,
                Token::Unique,
                Token::RightParen,
                Token::EOF,
            ]
        );
    }

    #[test]
    fn scanner_err() {
        assert_err(Scanner::scan("#".into()), "Unrecognized token '#'");
        assert_err(Scanner::scan("\"unterminated string".into()), ERROR_EOF);
    }

    #[test]
    fn parser() {
        let tokens = vec![
            Token::Select,
            Token::Star,
            Token::From,
            Token::Identifier(String::from("people")),
            Token::EOF,
        ];
        let expr = Parser::new(tokens).parse().unwrap();
        assert_eq!(
            expr,
            Command::Select {
                key: Key::Glob,
                table: Token::Identifier(String::from("people")),
            }
        );

        let tokens = vec![
            Token::Create,
            Token::Table,
            Token::Identifier(String::from("people")),
            Token::LeftParen,
            Token::Identifier(String::from("ID")),
            Token::Identifier(String::from("number")),
            Token::Primary,
            Token::Key,
            Token::Comma,
            Token::Identifier(String::from("FirstName")),
            Token::Identifier(String::from("string")),
            Token::Comma,
            Token::Identifier(String::from("LastName")),
            Token::Identifier(String::from("string")),
            Token::Comma,
            Token::Identifier(String::from("Age")),
            Token::Identifier(String::from("number")),
            Token::Not,
            Token::Null,
            Token::Check,
            Token::LeftParen,
            Token::Identifier("Age".into()),
            Token::GreaterEqual,
            Token::Number(18.0),
            Token::RightParen,
            Token::RightParen,
        ];
        let expr = Parser::new(tokens).parse().unwrap();
        assert_eq!(
            expr,
            Command::CreateTable {
                name: Token::Identifier(String::from("people")),
                col_decls: vec![
                    ColDecl::new(
                        Token::Identifier(String::from("ID")),
                        Ty::Number,
                        vec![Constraint::PrimaryKey]
                    ),
                    ColDecl::new(
                        Token::Identifier(String::from("FirstName")),
                        Ty::String,
                        Vec::new()
                    ),
                    ColDecl::new(
                        Token::Identifier(String::from("LastName")),
                        Ty::String,
                        Vec::new()
                    ),
                    ColDecl::new(
                        Token::Identifier(String::from("Age")),
                        Ty::Number,
                        vec![
                            Constraint::NotNull,
                            Constraint::Check(Expr::Binary {
                                left: Token::Identifier("Age".into()),
                                op: Token::GreaterEqual,
                                right: Token::Number(18.0)
                            })
                        ]
                    ),
                ]
            }
        );
    }

    #[test]
    fn parser_err() {
        assert_err(
            Parser::new(vec![Token::From]).parse(),
            "Unexpected token: `From`; expected one of: Insert, Select",
        );
        assert_err(
            Parser::new(vec![
                Token::Insert,
                Token::Into,
                Token::Identifier("table".to_string()),
                Token::LeftParen,
                Token::Identifier("field".to_string()),
                Token::RightParen,
                Token::Values,
                Token::LeftParen,
                Token::Star,
                Token::RightParen,
            ])
            .parse(),
            "Unexpected token: `Star`; expected one of: Number(0.0), String(\"\")",
        )
    }

    fn assert_err<T>(res: Result<T>, expected: &str) {
        assert!(res.is_err());
        match res {
            Ok(_) => unreachable!(),
            Err(e) => assert_eq!(e.to_string(), expected),
        }
    }
}
