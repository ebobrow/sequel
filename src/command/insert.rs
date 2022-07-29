use bytes::Bytes;

use crate::{
    connection::Frame,
    db::{Column, Db},
    parse::{LiteralValue, Token, Tokens},
};

use super::{
    error::{CmdError, CmdResult},
    on_table_mut,
};

// TODO: insert multiple columns
pub fn insert(db: &Db, table: Token, cols: Tokens, values: Vec<LiteralValue>) -> CmdResult<Frame> {
    on_table_mut(db, table, |table| match cols {
        Tokens::List(cols) => {
            let mut columns = Vec::new();
            for (c, val) in cols.iter().zip(values.iter()) {
                columns.push(Column::new(
                    val.into(),
                    c.ident().ok_or(CmdError::Internal)?.to_string(),
                ));
            }
            if columns.len() < cols.len() {
                for omitted_col in &cols[columns.len()..] {
                    columns.push(Column::new(
                        Bytes::new(),
                        omitted_col.ident().ok_or(CmdError::Internal)?.to_string(),
                    ))
                }
            } else if columns.len() > cols.len() {
                return Err(CmdError::User("too many values supplied".into()));
            }
            table.append(columns);
            Ok(Frame::Null)
        }
        Tokens::Omitted => {
            let mut columns = Vec::new();
            for (c, val) in table.col_headers().iter().zip(values.iter()) {
                columns.push(Column::new(val.into(), c.name().to_string()));
            }
            let non_primary_keys: Vec<_> = table.non_primary_keys().collect();
            if columns.len() < non_primary_keys.len() {
                for omitted_col in &non_primary_keys[columns.len()..] {
                    columns.push(Column::new(Bytes::new(), omitted_col.name().to_string()))
                }
            } else if columns.len() > non_primary_keys.len() {
                return Err(CmdError::User("too many values supplied".into()));
            }
            table.append(columns);
            Ok(Frame::Null)
        }
    })
}
