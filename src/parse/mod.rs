use bytes::Bytes;

use parser::Parser;
use scanner::Scanner;

mod ast;
mod parser;
mod scanner;
mod token;

pub fn parse(stream: Bytes) {
    let tokens = Scanner::scan(stream);
    let mut parser = Parser::new(tokens);
    if let Some(expr) = parser.parse() {
        println!("{:#?}", expr);
    }
}
