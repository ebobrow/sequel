use crate::{
    connection::Frame,
    db::{Column, Db},
    parse::{Key, LiteralValue, Token},
};

// TODO: CmdError, CmdResult
pub fn insert(db: &Db, table: Token, cols: Key, values: Vec<LiteralValue>) -> Frame {
    // TODO: this is all duplicate code, can move into `command/mod.rs`
    let mut db = db.lock().unwrap();
    let table_name = table.ident().unwrap();
    if let Some(table) = db.get_mut(table_name) {
        if let Key::List(cols) = cols {
            let mut columns = Vec::new();
            for (c, val) in cols.iter().zip(values.iter()) {
                columns.push(Column::new(val.into(), c.ident().unwrap().to_string()));
            }
            table.append(columns);
            Frame::Null
        } else {
            // TODO: error handling bad
            Frame::Error("* bad".into())
        }
    } else {
        Frame::Error(format!("Table \"{}\" not found", table_name))
    }
}
