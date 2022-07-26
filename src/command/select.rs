use crate::{
    connection::Frame,
    db::Db,
    parse::{Key, Token},
};

use super::{
    error::{CmdError, CmdResult},
    on_table,
};

pub fn select(db: &Db, key: Key, table: Token) -> CmdResult<Frame> {
    on_table(db, table, |table| {
        match key {
            // TODO: header on top like "ID | name | age"
            // but not if i change to server/client stuff
            Key::Glob => Ok(Frame::Table(
                table
                    .rows()
                    .iter()
                    .map(|row| row.all_cols().map(|col| col.data().clone()).collect())
                    .collect(),
            )),
            Key::List(cols) => {
                let names = cols
                    .iter()
                    .map(|col| Ok(col.ident().ok_or(CmdError::Internal)?.to_string()))
                    .collect::<CmdResult<Vec<_>>>()?;
                Ok(Frame::Table(
                    table
                        .rows()
                        .iter()
                        .map(|row| row.cols(&names[..]).map(|col| col.data().clone()).collect())
                        .collect(),
                ))
            }
        }
    })
}
