use std::cmp::Ordering;

use anyhow::{anyhow, Result};
use bytes::Bytes;

use crate::{
    connection::Frame,
    db::{Column, ColumnHeader, Db, DefaultOpt},
    parse::{LiteralValue, Token, Tokens},
};

use super::on_table_mut;

pub fn insert(db: &Db, table: Token, cols: Tokens, rows: Vec<Vec<LiteralValue>>) -> Result<Frame> {
    on_table_mut(db, table, |table| match cols {
        Tokens::List(specified_cols) => {
            let specified_col_names = specified_cols
                .iter()
                .map(|col| col.ident())
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| anyhow!("Internal error"))?;
            let unknown_cols: Vec<_> = specified_col_names
                .iter()
                .filter(|col| !table.visible_keys().any(|header| header.name() == **col))
                .collect();
            if !unknown_cols.is_empty() {
                return Err(anyhow!("Unknown columns: {:?}", unknown_cols,));
            }
            for values in rows {
                let mut columns = Vec::new();
                for (name, val) in specified_col_names.iter().zip(values.iter()) {
                    columns.push(Column::new(val.into(), name.to_string()));
                }
                match columns.len().cmp(&specified_cols.len()) {
                    Ordering::Less => {
                        for mut omitted_col in specified_col_names
                            .iter()
                            .map(|name| {
                                table
                                    .col_headers()
                                    .iter()
                                    .find(|header| header.name() == *name)
                                    .unwrap()
                                    .clone()
                            })
                            .skip(columns.len())
                        {
                            columns.push(get_default(&mut omitted_col)?);
                        }
                    }
                    Ordering::Greater => {
                        return Err(anyhow!("too many values supplied"));
                    }
                    Ordering::Equal => {}
                };
                for default_col in table.col_headers_mut().iter_mut().filter(|col| {
                    col.default() != &DefaultOpt::None
                        && !specified_col_names.contains(&&col.name().to_string())
                }) {
                    columns.push(get_default(default_col)?);
                }
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
                match columns.len().cmp(&table.visible_keys().count()) {
                    Ordering::Less => {
                        for omitted_col in table.visible_keys_mut().skip(columns.len()) {
                            columns.push(get_default(omitted_col)?);
                        }
                    }
                    Ordering::Greater => {
                        return Err(anyhow!("too many values supplied"));
                    }
                    Ordering::Equal => {}
                };
                table.append(columns)?;
            }
            Ok(Frame::Null)
        }
    })
}

fn get_default(header: &mut ColumnHeader) -> Result<Column> {
    let val = match header.default() {
        DefaultOpt::None => Bytes::new(),
        DefaultOpt::Some(val) => val.into(),
        DefaultOpt::Incrementing(_) => Bytes::from(
            header
                .inc()
                .ok_or_else(|| anyhow!("Internal error"))?
                .to_string(),
        ),
    };
    Ok(Column::new(val, header.name().to_string()))
}
