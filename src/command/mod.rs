use bytes::Bytes;

use crate::{
    connection::Frame,
    db::{Db, Table},
    parse::{self, Expr, Token},
};

use self::{
    error::{CmdError, CmdResult},
    insert::insert,
    select::select,
};

mod error;
mod insert;
mod select;

// Basicaly visitor pattern--rename?
pub fn run_cmd(db: &Db, stream: Bytes) -> Frame {
    let res = match parse::parse(stream) {
        Ok(Expr::Select { key, table }) => select(db, key, table),
        Ok(Expr::Insert {
            table,
            cols,
            values,
        }) => insert(db, table, cols, values),
        Err(e) => Err(CmdError::Other(format!("{:?}", e))),
    };
    match res {
        Ok(frame) => frame,
        Err(e) => Frame::Error(format!("Error:\n{:?}", e)),
    }
}

fn on_table<F>(db: &Db, table: Token, f: F) -> CmdResult<Frame>
where
    F: FnOnce(&Table) -> CmdResult<Frame>,
{
    let db = db.lock().unwrap();
    let table_name = table.ident().ok_or(CmdError::Internal)?;
    let table = db
        .get(table_name)
        .ok_or_else(|| CmdError::TableNotFound(table_name.to_string()))?;
    f(table)
}

fn on_table_mut<F>(db: &Db, table: Token, f: F) -> CmdResult<Frame>
where
    F: FnOnce(&mut Table) -> CmdResult<Frame>,
{
    let mut db = db.lock().unwrap();
    let table_name = table.ident().ok_or(CmdError::Internal)?;
    let table = db
        .get_mut(table_name)
        .ok_or_else(|| CmdError::TableNotFound(table_name.to_string()))?;
    f(table)
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use crate::{
        db::{Column, ColumnHeader, Table},
        parse::{Key, LiteralValue, Token, Tokens},
    };

    use super::*;

    #[test]
    fn test_select() {
        let db = init_db();
        assert_eq!(
            select(&db, Key::Glob, Token::Identifier("people".into())),
            Ok(Frame::Table(vec![
                vec![Bytes::from("name"), Bytes::from("age"), Bytes::from("ID")],
                vec![Bytes::from("Elliot"), Bytes::from("16")]
            ]))
        );
        assert_eq!(
            select(
                &db,
                Key::List(vec![Token::Identifier("name".into())]),
                Token::Identifier("people".into())
            ),
            Ok(Frame::Table(vec![
                vec![Bytes::from("name")],
                vec![Bytes::from("Elliot")]
            ]))
        );
    }

    #[test]
    fn test_insert() {
        let db = init_db();
        assert!(insert(
            &db,
            Token::Identifier("people".into()),
            Tokens::List(vec![
                Token::Identifier("name".into()),
                Token::Identifier("age".into()),
            ]),
            vec![
                LiteralValue::String("Joe".into()),
                LiteralValue::Number(60.0),
            ],
        )
        .is_ok());
        assert!(insert(
            &db,
            Token::Identifier("people".into()),
            Tokens::Omitted,
            vec![
                LiteralValue::String("Fredward".into()),
                LiteralValue::Number(999.0),
            ],
        )
        .is_ok());
        assert_eq!(
            select(&db, Key::Glob, Token::Identifier("people".into())),
            Ok(Frame::Table(vec![
                vec![Bytes::from("name"), Bytes::from("age"), Bytes::from("ID")],
                vec![Bytes::from("Elliot"), Bytes::from("16")],
                vec![Bytes::from("Joe"), Bytes::from("60")],
                vec![Bytes::from("Fredward"), Bytes::from("999")]
            ]))
        );
    }

    fn init_db() -> Db {
        let mut table = Table::try_from(vec![
            ColumnHeader::new("name".into()),
            ColumnHeader::new("age".into()),
        ])
        .unwrap();
        table.append(vec![
            Column::new(Bytes::from("Elliot"), "name".into()),
            Column::new(Bytes::from("16"), "age".into()),
        ]);
        Arc::new(Mutex::new(HashMap::from([("people".into(), table)])))
    }
}
