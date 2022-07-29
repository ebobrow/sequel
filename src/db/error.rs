use std::fmt::Debug;

pub type DbResult<T> = Result<T, DbError>;

#[derive(PartialEq)]
pub enum DbError {
    Creation(String),
    Internal,
}

impl Debug for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Creation(msg) => write!(f, "{}", msg),
            Self::Internal => write!(f, "Internal error"),
        }
    }
}
