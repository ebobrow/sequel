use bytes::Bytes;

use parser::Parser;
use scanner::Scanner;

pub use self::ast::{Expr, Key};
pub use self::error::{ParseError, ParseResult};

mod ast;
mod error;
mod parser;
mod scanner;
mod token;

pub fn parse(stream: Bytes) -> ParseResult<Expr> {
    let tokens = Scanner::scan(stream)?;
    let mut parser = Parser::new(tokens);
    let expr = parser.parse();
    if let Ok(expr) = &expr {
        println!("{:#?}", expr);
    }
    expr
}

// TODO: test failures as well
#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::{
        ast::{Expr, Key},
        parser::Parser,
        scanner::Scanner,
        token::{Literal, Token, TokenType},
    };

    #[test]
    fn scanner() {
        let stream = Bytes::from_static(b"INSERT 17.6 * (\"one\", \"two\") table");
        let tokens = Scanner::scan(stream).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::new(
                    TokenType::Insert,
                    Bytes::from_static(b"INSERT"),
                    Literal::Null
                ),
                Token::new(
                    TokenType::Number,
                    Bytes::from_static(b"17.6"),
                    Literal::Number(17.6)
                ),
                Token::new(TokenType::Star, Bytes::from_static(b"*"), Literal::Null),
                Token::new(
                    TokenType::LeftParen,
                    Bytes::from_static(b"("),
                    Literal::Null
                ),
                Token::new(
                    TokenType::String,
                    Bytes::from_static(b"\"one\""),
                    Literal::String("one".to_string())
                ),
                Token::new(TokenType::Comma, Bytes::from_static(b","), Literal::Null),
                Token::new(
                    TokenType::String,
                    Bytes::from_static(b"\"two\""),
                    Literal::String("two".to_string())
                ),
                Token::new(
                    TokenType::RightParen,
                    Bytes::from_static(b")"),
                    Literal::Null
                ),
                Token::new(
                    TokenType::Identifier,
                    Bytes::from_static(b"table"),
                    Literal::Null
                ),
                Token::new(TokenType::EOF, Bytes::new(), Literal::Null)
            ]
        );
    }

    #[test]
    fn parser() {
        let tokens = vec![
            Token::new(
                TokenType::Select,
                Bytes::from_static(b"SELECT"),
                Literal::Null,
            ),
            Token::new(TokenType::Star, Bytes::from_static(b"*"), Literal::Null),
            Token::new(TokenType::From, Bytes::from_static(b"FROM"), Literal::Null),
            Token::new(
                TokenType::Identifier,
                Bytes::from_static(b"people"),
                Literal::Null,
            ),
            Token::new(TokenType::EOF, Bytes::new(), Literal::Null),
        ];
        let expr = Parser::new(tokens).parse().unwrap();
        assert_eq!(
            expr,
            Expr::Select {
                key: Key::Glob,
                table: Token::new(
                    TokenType::Identifier,
                    Bytes::from_static(b"people"),
                    Literal::Null
                )
            }
        );
    }
}
