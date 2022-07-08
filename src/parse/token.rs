use bytes::Bytes;

#[derive(Debug, Clone)]
pub enum TokenType {
    Star,

    // Should these all be stored as `Token`s or like a standard library or something
    Insert,
    Select,
    From,

    Identifier,
    Number,
    String,

    EOF,
}

#[derive(Debug)]
// TODO: needed?
pub enum Literal {
    String(String),
    Number(f64),
    Null,
}

#[derive(Debug)]
pub struct Token {
    ty: TokenType,
    // TODO: Bytes or String? Or &[u8] or Vec<u8>?
    lexeme: Bytes,
    literal: Literal,
}

impl Token {
    pub fn new(ty: TokenType, lexeme: Bytes, literal: Literal) -> Token {
        Token {
            ty,
            lexeme,
            literal,
        }
    }
}
