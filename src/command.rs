use bytes::Bytes;

use crate::{
    frame::Frame,
    parse::{self, Expr, Key},
    Db,
};

pub fn run_cmd(db: &Db, stream: Bytes) -> Frame {
    match parse::parse(stream) {
        Ok(Expr::Select { key, table }) => {
            let db = db.lock().unwrap();
            let table_name = std::str::from_utf8(table.lexeme()).unwrap();
            if let Some(table) = db.get(table_name) {
                match key {
                    Key::Glob => {
                        Frame::Array(table.values().map(|v| Frame::Bulk(v.clone())).collect())
                    }
                    Key::List(cols) => {
                        if let Some((_, v)) = table.iter().find(|(k, _)| {
                            cols.iter()
                                .find(|col| {
                                    // TODO: this isn't actually how it works because we don't
                                    // have `Page` datatype or anything
                                    std::str::from_utf8(col.lexeme()).unwrap() == &k[..]
                                })
                                .is_some()
                        }) {
                            Frame::Bulk(v.clone())
                        } else {
                            Frame::Null
                        }
                    }
                }
            } else {
                Frame::Error(format!("Table \"{}\" not found", table_name))
            }
        }
        Ok(Expr::Insert {
            table,
            cols,
            values,
        }) => {
            // TODO: again need `Page` before we can really do this
            todo!()
        }
        Err(e) => Frame::Error(format!("Error:\n{:?}", e)),
    }
}
