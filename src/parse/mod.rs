use bytes::Bytes;

use parser::Parser;
use scanner::Scanner;

use self::ast::Expr;

mod ast;
mod parser;
mod scanner;
mod token;

pub fn parse(stream: Bytes) -> Option<Expr> {
    let tokens = Scanner::scan(stream);
    let mut parser = Parser::new(tokens);
    let expr = parser.parse();
    // TODO: Frame::Error and send error to client
    if let Some(expr) = &expr {
        println!("{:#?}", expr);
    }
    expr
}
