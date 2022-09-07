use anyhow::{anyhow, Result};
use bytes::Bytes;

use crate::parse::{LiteralValue, Ty};

#[derive(Eq, Debug)]
pub struct Row {
    primary_key_col: Column,
    cols: Vec<Column>,
}

impl Row {
    pub fn new(primary_key_col: Column, cols: Vec<Column>) -> Row {
        Row {
            primary_key_col,
            cols,
        }
    }

    pub fn cols(&self, names: &[String]) -> Option<Vec<Bytes>> {
        let mut cols = Vec::new();
        let all_col_names = self
            .cols
            .clone()
            .into_iter()
            .chain([self.primary_key_col.clone()]);
        for name in names {
            cols.push(all_col_names.clone().find(|col| col.name() == name)?.data);
        }
        Some(cols)
    }
}

impl PartialEq for Row {
    fn eq(&self, other: &Self) -> bool {
        self.primary_key_col == other.primary_key_col
    }
}

impl PartialOrd for Row {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.primary_key_col.partial_cmp(&other.primary_key_col)
    }
}

impl Ord for Row {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.primary_key_col.cmp(&other.primary_key_col)
    }
}

#[derive(Clone, PartialEq)]
pub enum DefaultOpt {
    None,
    Some(LiteralValue),
    Incrementing(u8),
}

#[derive(Clone)]
pub struct ColumnHeader {
    name: String,
    is_primary_key: bool,
    is_hidden: bool,
    default: DefaultOpt,
    ty: Ty,
}

impl ColumnHeader {
    pub fn new(
        name: String,
        default: DefaultOpt,
        ty: Ty,
        is_primary_key: bool,
    ) -> Result<ColumnHeader> {
        Self::new_with_check(name, default, ty, is_primary_key)
    }

    fn new_with_check(
        name: String,
        default: DefaultOpt,
        ty: Ty,
        is_primary_key: bool,
    ) -> Result<ColumnHeader> {
        let print_err = |def_ty| {
            anyhow!(
                "Default type doesn't match declared type; expected {}, got {:?}",
                def_ty,
                ty
            )
        };
        match &default {
            DefaultOpt::None => {}
            DefaultOpt::Some(val) => match val {
                LiteralValue::String(_) => {
                    if ty != Ty::String {
                        return Err(print_err("String"));
                    }
                }
                LiteralValue::Number(_) => {
                    if ty != Ty::Number {
                        return Err(print_err("Number"));
                    }
                }
            },
            DefaultOpt::Incrementing(_) => {
                if ty != Ty::Number {
                    return Err(print_err("Number"));
                }
            }
        };
        Ok(ColumnHeader {
            name,
            is_primary_key,
            is_hidden: false,
            default,
            ty,
        })
    }

    pub fn new_hidden() -> ColumnHeader {
        ColumnHeader {
            name: "ID".into(),
            is_primary_key: true,
            is_hidden: true,
            default: DefaultOpt::Incrementing(0),
            ty: Ty::Number,
        }
    }

    pub fn is_primary(&self) -> bool {
        self.is_primary_key
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn default(&self) -> &DefaultOpt {
        &self.default
    }

    pub fn inc(&mut self) -> Option<u8> {
        if let DefaultOpt::Incrementing(i) = self.default {
            self.default = DefaultOpt::Incrementing(i + 1);
            Some(i)
        } else {
            None
        }
    }

    pub fn is_hidden(&self) -> bool {
        self.is_hidden
    }

    pub fn ty(&self) -> &Ty {
        &self.ty
    }
}

#[derive(Eq, Clone, Debug)]
pub struct Column {
    // TODO: Should this be `LiteralValue`? (doens't implement `Eq`)
    data: Bytes,
    name: String, // Should correspond with name in `ColumnHeader`
}

impl Column {
    pub fn new(data: Bytes, name: String) -> Column {
        Column { data, name }
    }

    pub fn data(&self) -> &Bytes {
        &self.data
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl PartialEq for Column {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl PartialOrd for Column {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.data.partial_cmp(&other.data)
    }
}

impl Ord for Column {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.data.cmp(&other.data)
    }
}
