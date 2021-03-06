use std::{fmt::Debug, io};

pub type ConnResult<T> = std::result::Result<T, ConnError>;

pub enum ConnError {
    Incomplete,
    Io(io::Error),
    Reset,
    Protocol(u8),
}

impl Debug for ConnError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConnError::Incomplete => write!(f, "stream ended early"),
            ConnError::Io(err) => err.fmt(f),
            ConnError::Reset => write!(f, "Connection reset by peer"),
            ConnError::Protocol(c) => write!(f, "Unexpected char: {}", *c as char),
        }
    }
}
impl From<io::Error> for ConnError {
    fn from(e: io::Error) -> Self {
        ConnError::Io(e)
    }
}
