use anyhow::{anyhow, Result};
use bytes::Bytes;

use crate::{
    connection::Frame,
    db::{Db, Table},
    parse::{self, Command, Token},
};

use self::{create_table::create_table, insert::insert, select::select};

mod create_table;
mod insert;
mod select;

// Basicaly visitor pattern--rename?
pub fn run_cmd(db: &Db, stream: Bytes) -> Frame {
    let res = match parse::parse(stream) {
        Ok(Command::Select { key, table }) => select(db, key, table),
        Ok(Command::Insert { table, cols, rows }) => insert(db, table, cols, rows),
        Ok(Command::CreateTable { name, def }) => create_table(db, name, def),
        Err(e) => return Frame::Error(format!("{:?}", e)),
    };
    res.unwrap_or_else(|e| Frame::Error(format!("{:?}", e)))
}

fn on_table<F>(db: &Db, table: Token, f: F) -> Result<Frame>
where
    F: FnOnce(&Table) -> Result<Frame>,
{
    let db = db.lock().unwrap();
    let table_name = table.ident().ok_or_else(|| anyhow!("Internal error"))?;
    let table = db
        .get(table_name)
        .ok_or_else(|| anyhow!("Table \"{}\" not found", table_name))?;
    f(table)
}

fn on_table_mut<F>(db: &Db, table: Token, f: F) -> Result<Frame>
where
    F: FnOnce(&mut Table) -> Result<Frame>,
{
    let mut db = db.lock().unwrap();
    let table_name = table.ident().ok_or_else(|| anyhow!("Internal error"))?;
    let table = db
        .get_mut(table_name)
        .ok_or_else(|| anyhow!("Table \"{}\" not found", table_name))?;
    f(table)
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;
    use std::{
        collections::HashMap,
        fmt::Debug,
        sync::{Arc, Mutex},
    };

    use crate::{
        db::{Column, ColumnHeader, DefaultOpt, Table},
        parse::{ColDecl, Key, LiteralValue, TableDef, Token, Tokens, Ty},
    };

    use super::*;

    #[test]
    fn test_select() {
        let db = init_db();
        assert_ok(
            select(&db, Key::Glob, Token::Identifier("people".into())),
            Frame::Table(vec![
                vec!["name".into(), "age".into(), "ID".into()],
                vec!["Elliot".into(), "16".into(), "0".into()],
            ]),
        );
        assert_ok(
            select(
                &db,
                Key::List(vec![Token::Identifier("name".into())]),
                Token::Identifier("people".into()),
            ),
            Frame::Table(vec![vec!["name".into()], vec!["Elliot".into()]]),
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
            vec![vec![
                LiteralValue::String("Joe".into()),
                LiteralValue::Number(OrderedFloat(60.0)),
            ]],
        )
        .is_ok());
        assert!(insert(
            &db,
            Token::Identifier("people".into()),
            Tokens::Omitted,
            vec![vec![
                LiteralValue::String("Fredward".into()),
                LiteralValue::Number(OrderedFloat(999.0)),
            ]],
        )
        .is_ok());
        assert_ok(
            select(&db, Key::Glob, Token::Identifier("people".into())),
            Frame::Table(vec![
                vec!["name".into(), "age".into(), "ID".into()],
                vec!["Elliot".into(), "16".into(), "0".into()],
                vec!["Joe".into(), "60".into(), "1".into()],
                vec!["Fredward".into(), "999".into(), "2".into()],
            ]),
        );
    }

    #[test]
    fn insert_wrong_num_cols() {
        let db = init_db();
        assert!(insert(
            &db,
            Token::Identifier("people".into()),
            Tokens::Omitted,
            vec![vec![LiteralValue::String("Elliot".into())]],
        )
        .is_ok());
        assert_ok(
            select(&db, Key::Glob, Token::Identifier("people".into())),
            Frame::Table(vec![
                vec!["name".into(), "age".into(), "ID".into()],
                vec!["Elliot".into(), "16".into(), "0".into()],
                vec!["Elliot".into(), Bytes::new(), "1".into()],
            ]),
        );

        assert_err(
            insert(
                &db,
                Token::Identifier("people".into()),
                Tokens::Omitted,
                vec![vec![
                    LiteralValue::Number(OrderedFloat(1.0)),
                    LiteralValue::Number(OrderedFloat(2.0)),
                    LiteralValue::Number(OrderedFloat(3.0)),
                    LiteralValue::Number(OrderedFloat(4.0)),
                ]],
            ),
            "too many values supplied",
        );
    }

    #[test]
    fn default_opts() {
        let db: Db = Arc::new(Mutex::new(HashMap::from([(
            "table".into(),
            Table::try_from(vec![
                ColumnHeader::new("three".into())
                    .def(DefaultOpt::Some(LiteralValue::Number(OrderedFloat(3.0))))
                    .ty(Ty::Number)
                    .build()
                    .unwrap(),
                ColumnHeader::new("inc".into())
                    .def(DefaultOpt::Incrementing(11))
                    .ty(Ty::Number)
                    .build()
                    .unwrap(),
            ])
            .unwrap(),
        )])));
        assert!(insert(
            &db,
            Token::Identifier("table".into()),
            Tokens::Omitted,
            vec![vec![]]
        )
        .is_ok());
        assert!(insert(
            &db,
            Token::Identifier("table".into()),
            Tokens::List(vec![Token::Identifier("three".into())]),
            vec![vec![LiteralValue::Number(OrderedFloat(4.0))]]
        )
        .is_ok());

        assert_ok(
            select(
                &db,
                Key::List(vec![
                    Token::Identifier("three".into()),
                    Token::Identifier("inc".into()),
                ]),
                Token::Identifier("table".into()),
            ),
            Frame::Table(vec![
                vec!["three".into(), "inc".into()],
                vec!["3".into(), "11".into()],
                vec!["4".into(), "12".into()],
            ]),
        );
    }

    #[test]
    fn test_create_table() {
        let db = Db::default();
        assert!(create_table(
            &db,
            Token::Identifier("people".to_string()),
            TableDef::Cols(vec![
                ColDecl::new(
                    Token::Identifier("name".to_string()),
                    Ty::String,
                    Vec::new()
                ),
                ColDecl::new(Token::Identifier("age".to_string()), Ty::Number, Vec::new())
            ])
        )
        .is_ok());

        assert!(create_table(
            &db,
            Token::Identifier("names".to_string()),
            TableDef::As(Box::new(Command::Select {
                key: Key::List(vec![Token::Identifier("name".to_string())]),
                table: Token::Identifier("people".to_string())
            }))
        )
        .is_ok());

        let db = db.lock().unwrap();
        let people = db.get("people").unwrap();
        assert_table_def_equals(people, &[("name", Ty::String), ("age", Ty::Number)]);

        let names = db.get("names").unwrap();
        assert_table_def_equals(names, &[("name", Ty::String)]);
    }

    fn init_db() -> Db {
        let mut table = Table::try_from(vec![
            ColumnHeader::new("name".into())
                .ty(Ty::String)
                .build()
                .unwrap(),
            ColumnHeader::new("age".into())
                .ty(Ty::Number)
                .build()
                .unwrap(),
        ])
        .unwrap();
        table
            .append(vec![
                Column::new(LiteralValue::String("Elliot".into()), "name".into()),
                Column::new(LiteralValue::Number(OrderedFloat(16.0)), "age".into()),
            ])
            .unwrap();
        Arc::new(Mutex::new(HashMap::from([("people".into(), table)])))
    }

    fn assert_ok<T: Debug + PartialEq>(res: Result<T>, expected: T) {
        assert!(res.is_ok());
        match res {
            Ok(res) => assert_eq!(res, expected),
            Err(_) => unreachable!(),
        }
    }

    fn assert_err<T>(res: Result<T>, expected: &str) {
        assert!(res.is_err());
        match res {
            Ok(_) => unreachable!(),
            Err(e) => assert_eq!(e.to_string(), expected),
        }
    }

    fn assert_table_def_equals(table: &Table, expected: &[(&str, Ty)]) {
        assert!(table.col_headers().iter().zip(expected).all(
            |(col_header, (expected_name, expected_ty))| {
                &col_header.name() == expected_name && col_header.ty() == expected_ty
            }
        ))
    }
}
