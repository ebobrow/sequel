use bytes::Bytes;

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
    on_table(db, table, |table| match key {
        Key::Glob => {
            // TODO: make sure same order and then add test for default column
            let headers: Vec<_> = table
                .col_headers()
                .iter()
                .map(|header| Bytes::copy_from_slice(header.name().as_bytes()))
                .collect();
            let mut contents: Vec<Vec<_>> = table
                .rows()
                .iter()
                .map(|row| row.all_cols().map(|col| col.data().clone()).collect())
                .collect();
            contents.insert(0, headers);
            Ok(Frame::Table(contents))
        }
        Key::List(cols) => {
            let names = cols
                .iter()
                .map(|col| Ok(col.ident().ok_or(CmdError::Internal)?.to_string()))
                .collect::<CmdResult<Vec<_>>>()?;
            let mut contents: Vec<Vec<_>> = table
                .rows()
                .iter()
                .map(|row| row.cols(&names[..]).map(|col| col.data().clone()).collect())
                .collect();
            contents.insert(0, names.into_iter().map(Bytes::from).collect());
            Ok(Frame::Table(contents))
        }
    })
}
