use anyhow::{anyhow, Result};
use bytes::Bytes;

use crate::{
    connection::Frame,
    db::Db,
    parse::{Key, Token},
};

use super::on_table;

pub fn select(db: &Db, key: Key, table: Token) -> Result<Frame> {
    on_table(db, table, |table| match key {
        Key::Glob => {
            let headers: Vec<_> = table
                .col_headers()
                .iter()
                .map(|header| Bytes::copy_from_slice(header.name().as_bytes()))
                .collect();
            let header_names: Vec<_> = table
                .col_headers()
                .iter()
                .map(|header| header.name().to_string())
                .collect();
            let mut contents: Vec<Vec<_>> = table
                .rows()
                .iter()
                .map(|row| row.cols(&header_names[..]))
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| anyhow!("Unknown column names"))?;
            contents.insert(0, headers);
            Ok(Frame::Table(contents))
        }
        Key::List(cols) => {
            let names = cols
                .iter()
                .map(|col| {
                    Ok(col
                        .ident()
                        .ok_or_else(|| anyhow!("Internal error"))?
                        .to_string())
                })
                .collect::<Result<Vec<_>>>()?;
            let mut contents: Vec<Vec<_>> = table
                .rows()
                .iter()
                .map(|row| row.cols(&names[..]))
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| anyhow!("Unknown column names"))?;
            contents.insert(0, names.into_iter().map(Bytes::from).collect());
            Ok(Frame::Table(contents))
        }
    })
}
