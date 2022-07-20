use bytes::Bytes;

use crate::{
    connection::Frame,
    db::Db,
    parse::{self, Expr},
};

use self::{insert::insert, select::select};

mod insert;
mod select;

// Basicaly visitor pattern--rename?
pub fn run_cmd(db: &Db, stream: Bytes) -> Frame {
    match parse::parse(stream) {
        Ok(Expr::Select { key, table }) => select(db, key, table),
        Ok(Expr::Insert {
            table,
            cols,
            values,
        }) => insert(db, table, cols, values),
        Err(e) => Frame::Error(format!("Error:\n{:?}", e)),
    }
}

// TODO: tests
