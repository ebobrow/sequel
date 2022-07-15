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
        token::Token,
    };

    #[test]
    fn scanner() {
        let stream = Bytes::from_static(b"INSERT 17.6 * (\"one\", \"two\") table");
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
    }
}
