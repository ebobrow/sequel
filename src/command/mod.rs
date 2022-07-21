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

// TODO: make separate test file for each mod?
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
            Frame::Array(vec![Frame::Bulk(Bytes::from("Elliot 16"))])
        );
        assert_eq!(
            select(
                &db,
                Key::List(vec![Token::Identifier("name".into())]),
                Token::Identifier("people".into())
            ),
            Frame::Array(vec![Frame::Bulk(Bytes::from("Elliot"))])
        );
    }

    #[test]
    fn test_insert() {
        let db = init_db();
        insert(
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
        );
        insert(
            &db,
            Token::Identifier("people".into()),
            Tokens::Omitted,
            vec![
                LiteralValue::String("Fredward".into()),
                LiteralValue::Number(999.0),
            ],
        );
        assert_eq!(
            select(&db, Key::Glob, Token::Identifier("people".into())),
            Frame::Array(vec![
                Frame::Bulk(Bytes::from("Elliot 16")),
                Frame::Bulk(Bytes::from("Joe 60")),
                Frame::Bulk(Bytes::from("Fredward 999")),
            ])
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
