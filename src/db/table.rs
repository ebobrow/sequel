use ordered_float::OrderedFloat;
use std::collections::BTreeSet;

use anyhow::{anyhow, bail};

use crate::{parse::LiteralValue, Ty};

use super::{row::Row, Column, ColumnHeader};

pub struct Table {
    col_headers: Vec<ColumnHeader>,
    rows: BTreeSet<Row>,
}

impl TryFrom<Vec<ColumnHeader>> for Table {
    type Error = anyhow::Error;

    fn try_from(cols: Vec<ColumnHeader>) -> Result<Self, Self::Error> {
        // Don't allow duplicate column names
        let mut sorted = cols.clone();
        sorted.sort_by_key(|col| col.name().to_string());
        for i in 0..sorted.len() - 1 {
            if sorted[i].name() == sorted[i + 1].name() {
                bail!("Cannot have duplicate columns");
            }
        }
        match cols.iter().filter(|col| col.is_primary()).count() {
            0 => {
                // If no primary key, create hidden auto incrementing
                let mut col_headers = cols;
                col_headers.push(ColumnHeader::new_hidden());
                Ok(Table {
                    col_headers,
                    rows: BTreeSet::new(),
                })
            }
            1 => Ok(Table {
                col_headers: cols,
                rows: BTreeSet::new(),
            }),
            n => bail!("Expected 1 primary key, found {}", n),
        }
    }
}

impl Table {
    pub fn rows(&self) -> &BTreeSet<Row> {
        // TODO: don't include hidden?
        &self.rows
    }

    pub fn append(&mut self, cols: Vec<Column>) -> anyhow::Result<()> {
        for col in &cols {
            let header = self
                .col_headers
                .iter()
                .find(|header| header.name() == col.name())
                .ok_or_else(|| anyhow!("Column {} not found", col.name()))?;

            // Check null
            if let LiteralValue::Null = col.data() {
                if header.not_null() {
                    bail!("Column {} non-nullable", header.name());
                }
                continue;
            }

            // Check unique
            if header.unique()
                && self
                    .rows()
                    .iter()
                    .any(|row| row.cols(&[header.name().to_string()]).unwrap()[0] == *col.data())
            {
                bail!("Col {} must be unique", header.name());
            }

            // Check type
            match header.ty() {
                Ty::String => {
                    if !matches!(col.data(), LiteralValue::String(_)) {
                        bail!("Expected string")
                    }
                }
                Ty::Number => {
                    if !matches!(col.data(), LiteralValue::Number(_)) {
                        bail!("Expected number")
                    }
                }
                Ty::Bool => {
                    if !matches!(col.data(), LiteralValue::Bool(_)) {
                        bail!("Expected bool")
                    }
                }
            }

            // `CHECK` condition
            if let Some(expr) = header.check() {
                match expr.eval(&cols)? {
                    LiteralValue::Bool(b) => {
                        if !b {
                            bail!("Check condition on {} failed", header.name())
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
        let (primary_col, cols): (Vec<_>, Vec<_>) = cols
            .into_iter()
            .partition(|col| col.name() == self.primary_key().name());
        match &primary_col[..] {
            [] => {
                let val =
                    LiteralValue::Number(OrderedFloat(self.primary_key_mut().inc().ok_or_else(
                        || anyhow!("Must specify primary key if it doesn't have default"),
                    )? as f64));
                self.rows
                    .insert(Row::new(Column::new(val, "ID".into()), cols));
            }
            [primary_col] => {
                self.rows.insert(Row::new(primary_col.clone(), cols));
            }
            _ => panic!(),
        };
        Ok(())
    }

    fn primary_key(&self) -> &ColumnHeader {
        self.col_headers
            .iter()
            .find(|col| col.is_primary())
            .unwrap()
    }

    fn primary_key_mut(&mut self) -> &mut ColumnHeader {
        self.col_headers
            .iter_mut()
            .find(|col| col.is_primary())
            .unwrap()
    }

    pub fn col_headers(&self) -> &[ColumnHeader] {
        self.col_headers.as_ref()
    }

    pub fn col_headers_mut(&mut self) -> &mut [ColumnHeader] {
        self.col_headers.as_mut()
    }

    pub fn visible_keys(&self) -> impl Iterator<Item = &ColumnHeader> {
        self.col_headers().iter().filter(|col| !col.is_hidden())
    }

    pub fn visible_keys_mut(&mut self) -> impl Iterator<Item = &mut ColumnHeader> {
        self.col_headers_mut()
            .iter_mut()
            .filter(|col| !col.is_hidden())
    }
}
