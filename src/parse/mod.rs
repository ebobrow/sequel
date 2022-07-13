use bytes::Bytes;

use parser::Parser;
use scanner::Scanner;

use self::ast::Expr;
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
