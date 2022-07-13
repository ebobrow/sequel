use bytes::Bytes;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Star,
    LeftParen,
    RightParen,
    Comma,

    // Should these all be stored as `Token`s or like a standard library or something
    Insert,
    Select,
    From,
    Into,
    Values,

    Identifier,
    Number,
    String,

    EOF,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    String(String),
    Number(f64),
    Null,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    ty: TokenType,
    // TODO: Bytes or String? Or &[u8] or Vec<u8>?
    // TODO: also is this needed except for identifiers, which can be moved to `literal`
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

    pub fn ty(&self) -> &TokenType {
        &self.ty
    }

    pub fn literal(&self) -> &Literal {
        &self.literal
    }
}
