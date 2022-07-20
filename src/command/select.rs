use bytes::Bytes;

use crate::{
    connection::Frame,
    db::{Column, Db},
    parse::{Key, Token},
};

pub fn select(db: &Db, key: Key, table: Token) -> Frame {
    let db = db.lock().unwrap();
    let table_name = table.ident().unwrap();
    if let Some(table) = db.get(table_name) {
        match key {
            // TODO: header on top like "ID | name | age"
            Key::Glob => Frame::Array(
                table
                    .rows()
                    .iter()
                    .map(|row| cols_to_frame(row.all_cols()))
                    .collect(),
            ),
            Key::List(cols) => {
                let names: Vec<_> = cols
                    .iter()
                    .map(|col| col.ident().unwrap().to_string())
                    .collect();
                Frame::Array(
                    table
                        .rows()
                        .iter()
                        .map(|row| cols_to_frame(row.cols(&names[..])))
                        .collect(),
                )
            }
        }
    } else {
        Frame::Error(format!("Table \"{}\" not found", table_name))
    }
}

fn cols_to_frame<'a>(cols: impl Iterator<Item = &'a Column>) -> Frame {
    let bytes = cols
        .map(|col| &col.data()[..])
        .fold(Vec::new(), |mut acc, col| {
            acc.push(b' ');
            acc.append(&mut col.to_vec());
            acc
        });
    Frame::Bulk(Bytes::from(bytes))
}
