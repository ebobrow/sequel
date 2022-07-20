use bytes::Bytes;

use crate::{
    connection::Frame,
    db::Db,
    parse::{self, Expr, Key},
};

// TODO: this feels weird here. move to parser or conn or db?
pub fn run_cmd(db: &Db, stream: Bytes) -> Frame {
    match parse::parse(stream) {
        Ok(Expr::Select { key, table }) => {
            let db = db.lock().unwrap();
            let table_name = table.ident().unwrap();
            if let Some(table) = db.get(table_name) {
                match key {
                    Key::Glob => {
                        // TODO: extract to function on table
                        let mut values = Vec::new();
                        for row in table.rows() {
                            let mut s = String::new();
                            for col in row.cols() {
                                s.push_str(std::str::from_utf8(col.data()).unwrap());
                            }
                            // TODO: don't go from Bytes back to String
                            values.push(Frame::Bulk(Bytes::copy_from_slice(s.as_bytes())));
                        }
                        Frame::Array(values)
                    }
                    Key::List(cols) => {
                        let mut values = Vec::new();
                        for row in table.rows() {
                            let mut s = String::new();
                            for col in row.cols() {
                                if cols
                                    .iter()
                                    .find(|c| &c.ident().unwrap()[..] == col.name())
                                    .is_some()
                                {
                                    s.push_str(std::str::from_utf8(col.data()).unwrap());
                                }
                            }
                            // TODO: don't go from Bytes back to String
                            values.push(Frame::Bulk(Bytes::copy_from_slice(s.as_bytes())));
                        }
                        Frame::Array(values)
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
