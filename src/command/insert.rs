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
            table.append(columns);
            Ok(Frame::Null)
        }
        Tokens::Omitted => {
            let mut columns = Vec::new();
            for (c, val) in table.col_headers().iter().zip(values.iter()) {
                columns.push(Column::new(val.into(), c.name().to_string()));
            }
            table.append(columns);
            Ok(Frame::Null)
        }
    })
}
