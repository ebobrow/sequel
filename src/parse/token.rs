use macros::Keywords;

#[derive(Debug, Clone, PartialEq, Keywords)]
pub enum Token {
    Star,
    LeftParen,
    RightParen,
    Comma,
    GreaterThan,
    LessThan,
    Equal,
    GreaterEqual,
    LessEqual,

    #[keyword]
    And,
    #[keyword]
    Or,

    #[keyword]
    Insert,
    #[keyword]
    Select,
    #[keyword]
    From,
    #[keyword]
    Into,
    #[keyword]
    Values,
    #[keyword]
    Create,
    #[keyword]
    Table,

    #[keyword]
    Not,
    #[keyword]
    Null,
    #[keyword]
    Unique,
    #[keyword]
    Primary,
    #[keyword]
    Foreign,
    #[keyword]
    Key,
    #[keyword]
    Check,
    #[keyword]
    Default,
    #[keyword]
    Index,

    Identifier(String),
    Number(f64),
    String(String),
    #[keyword(true = true, false = false)]
    Bool(bool),

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
