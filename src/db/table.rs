use std::collections::BTreeSet;

use bytes::Bytes;

use super::{
    row::{ColumnHeader, Row},
    Column,
};

pub struct Table {
    col_headers: Vec<ColumnHeader>,
    rows: BTreeSet<Row>,
    inc: Option<u8>,
}

impl TryFrom<Vec<ColumnHeader>> for Table {
    // TODO: Error type
    type Error = String;

    fn try_from(cols: Vec<ColumnHeader>) -> Result<Self, Self::Error> {
        // Don't allow duplicate column names
        let mut sorted = cols.clone();
        sorted.sort_by_key(|col| col.name().to_string());
        for i in 0..sorted.len() - 1 {
            if sorted[i].name() == sorted[i + 1].name() {
                return Err("Cannot have duplicate columns".into());
            }
        }
        match cols.iter().filter(|col| col.is_primary()).count() {
            0 => {
                // If no primary key, create hidden auto incrementing
                let mut col_headers = cols;
                col_headers.push(ColumnHeader::new_prinary("ID".into()));
                Ok(Table {
                    col_headers,
                    rows: BTreeSet::new(),
                    inc: Some(0),
                })
            }
            1 => Ok(Table {
                col_headers: cols,
                rows: BTreeSet::new(),
                inc: None,
            }),
            n => Err(format!("Expected 1 primary key, found {}", n)),
        }
    }
}

impl Table {
    pub fn rows(&self) -> &BTreeSet<Row> {
        &self.rows
    }

    pub fn append(&mut self, cols: Vec<Column>) {
        let (primary_col, cols): (Vec<_>, Vec<_>) = cols
            .into_iter()
            .partition(|col| col.name() == self.primary_key_name());
        // TODO: errors
        // TODO: null value for missing cols (some sort of `Value` enum or reuse `LiteralValue`)
        match &primary_col[..] {
            [] => {
                self.rows.insert(Row::new(
                    Column::new(Bytes::from(self.inc.unwrap().to_string()), "ID".into()),
                    cols,
                ));
                self.inc = self.inc.map(|i| i + 1);
            }
            [primary_col] => {
                self.rows.insert(Row::new(primary_col.clone(), cols));
            }
            _ => panic!(),
        };
    }

    // TODO: store this as field to avoid so many iterations?
    fn primary_key_name(&self) -> String {
        self.col_headers
            .iter()
            .find(|col| col.is_primary())
            .unwrap()
            .name()
            .into()
    }

    pub fn col_headers(&self) -> &[ColumnHeader] {
        self.col_headers.as_ref()
    }
}
