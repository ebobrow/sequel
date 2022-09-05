use anyhow::Result;
use bytes::Bytes;

use parser::Parser;
use scanner::Scanner;

pub use self::{
    ast::{ColDecls, Expr, Key, LiteralValue, Tokens, Ty},
    token::Token,
};

mod ast;
mod error;
mod parser;
mod scanner;
mod token;

pub fn parse(stream: Bytes) -> Result<Expr> {
    let tokens = Scanner::scan(stream)?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::{
        ast::{Expr, Key, Ty},
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
            Expr::Select {
                key: Key::Glob,
                table: Token::Identifier(String::from("people")),
            }
        );

        let tokens = vec![
            Token::Create,
            Token::Table,
            Token::Identifier(String::from("people")),
            Token::LeftParen,
            Token::Identifier(String::from("FirstName")),
            Token::Identifier(String::from("string")),
            Token::Comma,
            Token::Identifier(String::from("LastName")),
            Token::Identifier(String::from("string")),
            Token::Comma,
            Token::Identifier(String::from("Age")),
            Token::Identifier(String::from("number")),
            Token::RightParen,
        ];
        let expr = Parser::new(tokens).parse().unwrap();
        assert_eq!(
            expr,
            Expr::CreateTable {
                name: Token::Identifier(String::from("people")),
                col_decls: vec![
                    (Token::Identifier(String::from("FirstName")), Ty::String),
                    (Token::Identifier(String::from("LastName")), Ty::String),
                    (Token::Identifier(String::from("Age")), Ty::Number),
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
