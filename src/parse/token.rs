#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Star,
    LeftParen,
    RightParen,
    Comma,

    Insert,
    Select,
    From,
    Into,
    Values,

    Identifier(String),
    Number(f64),
    String(String),

    EOF,
}

impl Token {
    pub fn ident(&self) -> Option<&String> {
        if let Token::Identifier(ident) = self {
            Some(ident)
        } else {
            None
        }
    }
}
