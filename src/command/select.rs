use bytes::Bytes;

use crate::{
    connection::Frame,
    db::{Column, Db},
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
            Key::Glob => Ok(Frame::Array(
                table
                    .rows()
                    .iter()
                    .map(|row| cols_to_frame(row.all_cols()))
                    .collect(),
            )),
            Key::List(cols) => {
                let names = cols
                    .iter()
                    .map(|col| Ok(col.ident().ok_or(CmdError::Internal)?.to_string()))
                    .collect::<CmdResult<Vec<_>>>()?;
                Ok(Frame::Array(
                    table
                        .rows()
                        .iter()
                        .map(|row| cols_to_frame(row.cols(&names[..])))
                        .collect(),
                ))
            }
        }
    })
}

fn cols_to_frame<'a>(cols: impl Iterator<Item = &'a Column>) -> Frame {
    let mut bytes = cols
        .map(|col| &col.data()[..])
        .fold(vec![b'('], |mut acc, col| {
            acc.append(&mut col.to_vec());
            acc.push(b' ');
            acc
        });
    bytes.pop();
    bytes.push(b')');
    Frame::Bulk(Bytes::copy_from_slice(&bytes[..]))
}
