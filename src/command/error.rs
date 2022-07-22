use std::fmt::Debug;

pub type CmdResult<T> = Result<T, CmdError>;

#[derive(PartialEq)]
pub enum CmdError {
    // e.g. `table` Token is always parsed as `Token::Identifier`; should never error
    Internal,
    TableNotFound(String),
    Other(String),
}

impl Debug for CmdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Internal => write!(f, "Internal Error"),
            Self::TableNotFound(table) => write!(f, "Table \"{}\" not found", table),
            Self::Other(e) => write!(f, "{}", e),
        }
    }
}
