use bytes::Bytes;

use parser::Parser;
use scanner::Scanner;

pub use self::{
    ast::{Expr, Key, LiteralValue, Tokens},
    error::{ParseError, ParseResult},
    token::Token,
};

mod ast;
mod error;
mod parser;
mod scanner;
mod token;

pub fn parse(stream: Bytes) -> ParseResult<Expr> {
    let tokens = Scanner::scan(stream)?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::{
        ast::{Expr, Key},
        parser::Parser,
        scanner::Scanner,
        token::Token,
        ParseError,
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
        assert_eq!(
            Scanner::scan("#".into()),
            Err(ParseError::Unrecognized(b'#'))
        );
        assert_eq!(
            Scanner::scan("\"unterminated string".into()),
            Err(ParseError::UnexpectedEnd)
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

    #[test]
    fn parser_err() {
        assert_eq!(
            Parser::new(vec![Token::From]).parse(),
            Err(ParseError::Unexpected {
                expected: vec![Token::Insert, Token::Select],
                got: Token::From
            })
        );
        assert_eq!(
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
            Err(ParseError::Unexpected {
                expected: vec![Token::Number(0.0), Token::String(String::new())],
                got: Token::Star
            })
        )
    }
}
