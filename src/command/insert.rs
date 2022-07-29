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

pub fn insert(
    db: &Db,
    table: Token,
    cols: Tokens,
    rows: Vec<Vec<LiteralValue>>,
) -> CmdResult<Frame> {
    on_table_mut(db, table, |table| match cols {
        Tokens::List(cols) => {
            for values in rows {
                let mut columns = Vec::new();
                for (c, val) in cols.iter().zip(values.iter()) {
                    columns.push(Column::new(
                        val.into(),
                        c.ident().ok_or(CmdError::Internal)?.to_string(),
                    ));
                }
                // TODO: This doesn't work with `INSERT INTO people (name, ID) VALUES ("Elliot", 0)`
                // and `INSERT INTO people (name, age, ID) VALUES ("Elliot", 16)`
                check_row_length(
                    &mut columns,
                    cols.iter()
                        .map(|col| col.ident())
                        .collect::<Option<Vec<_>>>()
                        .ok_or(CmdError::Internal)?,
                )?;
                table.append(columns)?;
            }
            Ok(Frame::Null)
        }
        Tokens::Omitted => {
            for values in rows {
                let mut columns = Vec::new();
                for (c, val) in table.col_headers().iter().zip(values.iter()) {
                    columns.push(Column::new(val.into(), c.name().to_string()));
                }
                check_row_length(
                    &mut columns,
                    table
                        .non_primary_keys()
                        .into_iter()
                        .map(|col| col.name())
                        .collect(),
                )?;
                table.append(columns)?;
            }
            Ok(Frame::Null)
        }
    })
}

fn check_row_length(input: &mut Vec<Column>, expected: Vec<impl ToString>) -> CmdResult<()> {
    if input.len() < expected.len() {
        for omitted_col in &expected[input.len()..] {
            input.push(Column::new(Bytes::new(), omitted_col.to_string()));
        }
    } else if input.len() > expected.len() {
        return Err(CmdError::User("too many values supplied".into()));
    }
    Ok(())
}
