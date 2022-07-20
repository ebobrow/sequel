use crate::{
    connection::Frame,
    db::Db,
    parse::{Key, LiteralValue, Token},
};

// TODO: CmdError, CmdResult
pub fn insert(db: &Db, table: Token, cols: Key, values: Vec<LiteralValue>) -> Frame {
    todo!()
}
