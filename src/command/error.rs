// TODO: this is probably stupid and pointless, just send Frame::Error from command
use std::fmt::Debug;

use crate::db::DbError;

pub type CmdResult<T> = Result<T, CmdError>;

#[derive(PartialEq)]
pub enum CmdError {
    // e.g. `table` Token is always parsed as `Token::Identifier`; should never error
    Internal,
    TableNotFound(String),
    User(String),
    Db(DbError),
}

impl Debug for CmdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Internal => write!(f, "Internal Error"),
            Self::TableNotFound(table) => write!(f, "Table \"{}\" not found", table),
            Self::User(msg) => write!(f, "{}", msg),
            Self::Db(err) => write!(f, "{:?}", err),
        }
    }
}

impl From<DbError> for CmdError {
    fn from(e: DbError) -> Self {
        Self::Db(e)
    }
}
