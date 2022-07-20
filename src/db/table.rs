use std::collections::BTreeSet;

use super::row::{ColumnHeader, Row};

pub struct Table {
    col_headers: Vec<ColumnHeader>,
    rows: BTreeSet<Row>,
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
                })
            }
            1 => Ok(Table {
                col_headers: cols,
                rows: BTreeSet::new(),
            }),
            n => Err(format!("Expected 1 primary key, found {}", n)),
        }
    }
}

impl Table {
    // TODO: result instead of option?
    // pub fn primary_key_of(&self, row: &Row) -> Option<&Column> {
    //     if let Some(row) = self.rows.iter().find(|&r| r == row) {
    //         self.col_headers
    //             .iter()
    //             .find(|col| col.is_primary_key)
    //             .map(|col| row.cols.iter().find(|c| c.name == col.name))
    //             .flatten()
    //     } else {
    //         None
    //     }
    // }

    // pub fn primary_key(&self) -> &ColumnHeader {
    //     self.col_headers
    //         .iter()
    //         .find(|col| col.is_primary_key)
    //         .unwrap()
    // }

    pub fn rows(&self) -> &BTreeSet<Row> {
        &self.rows
    }
}
