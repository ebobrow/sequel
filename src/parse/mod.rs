use bytes::Bytes;

use scanner::Scanner;

mod scanner;
mod token;

pub fn parse(stream: Bytes) {
    let tokens = Scanner::scan(stream);

    println!("{:#?}", tokens);
}
