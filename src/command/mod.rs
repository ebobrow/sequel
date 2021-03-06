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
        Ok(Expr::Insert { table, cols, rows }) => insert(db, table, cols, rows),
        Err(e) => return Frame::Error(format!("{:?}", e)),
    };
    res.unwrap_or_else(|e| Frame::Error(format!("{:?}", e)))
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
        db::{Column, ColumnHeader, DefaultOpt, Table},
        parse::{Key, LiteralValue, Token, Tokens},
    };

    use super::*;

    #[test]
    fn test_select() {
        let db = init_db();
        assert_eq!(
            select(&db, Key::Glob, Token::Identifier("people".into())),
            Ok(Frame::Table(vec![
                vec!["name".into(), "age".into(), "ID".into()],
                vec!["Elliot".into(), "16".into(), "0".into()],
            ]))
        );
        assert_eq!(
            select(
                &db,
                Key::List(vec![Token::Identifier("name".into())]),
                Token::Identifier("people".into())
            ),
            Ok(Frame::Table(vec![
                vec!["name".into()],
                vec!["Elliot".into()],
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
            vec![vec![
                LiteralValue::String("Joe".into()),
                LiteralValue::Number(60.0),
            ]],
        )
        .is_ok());
        assert!(insert(
            &db,
            Token::Identifier("people".into()),
            Tokens::Omitted,
            vec![vec![
                LiteralValue::String("Fredward".into()),
                LiteralValue::Number(999.0),
            ]],
        )
        .is_ok());
        assert_eq!(
            select(&db, Key::Glob, Token::Identifier("people".into())),
            Ok(Frame::Table(vec![
                vec!["name".into(), "age".into(), "ID".into()],
                vec!["Elliot".into(), "16".into(), "0".into()],
                vec!["Joe".into(), "60".into(), "1".into()],
                vec!["Fredward".into(), "999".into(), "2".into()],
            ]))
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
        assert_eq!(
            select(&db, Key::Glob, Token::Identifier("people".into())),
            Ok(Frame::Table(vec![
                vec!["name".into(), "age".into(), "ID".into()],
                vec!["Elliot".into(), "16".into(), "0".into()],
                vec!["Elliot".into(), Bytes::new(), "1".into()],
            ]))
        );

        assert_eq!(
            insert(
                &db,
                Token::Identifier("people".into()),
                Tokens::Omitted,
                vec![vec![
                    LiteralValue::Number(1.0),
                    LiteralValue::Number(2.0),
                    LiteralValue::Number(3.0),
                    LiteralValue::Number(4.0)
                ]]
            ),
            Err(CmdError::User("too many values supplied".into()))
        );
    }

    #[test]
    fn default_opts() {
        let db: Db = Arc::new(Mutex::new(HashMap::from([(
            "table".into(),
            Table::try_from(vec![
                ColumnHeader::new("three".into(), DefaultOpt::Some("3".into())),
                ColumnHeader::new("inc".into(), DefaultOpt::Incrementing(11)),
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
            vec![vec![LiteralValue::String("not 3".into())]]
        )
        .is_ok());

        assert_eq!(
            select(
                &db,
                Key::List(vec![
                    Token::Identifier("three".into()),
                    Token::Identifier("inc".into())
                ]),
                Token::Identifier("table".into())
            ),
            Ok(Frame::Table(vec![
                vec!["three".into(), "inc".into()],
                vec!["3".into(), "11".into()],
                vec!["not 3".into(), "12".into()],
            ]))
        );
    }

    fn init_db() -> Db {
        let mut table = Table::try_from(vec![
            ColumnHeader::new("name".into(), DefaultOpt::None),
            ColumnHeader::new("age".into(), DefaultOpt::None),
        ])
        .unwrap();
        table
            .append(vec![
                Column::new("Elliot".into(), "name".into()),
                Column::new("16".into(), "age".into()),
            ])
            .unwrap();
        Arc::new(Mutex::new(HashMap::from([("people".into(), table)])))
    }
}
