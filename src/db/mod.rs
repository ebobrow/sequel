use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub use self::{
    error::DbError,
    row::{Column, ColumnHeader, DefaultOpt, Row},
    table::Table,
};

mod error;
mod row;
mod table;

// TODO: Page how
// TODO: file persistence
pub type Db = Arc<Mutex<HashMap<String, Table>>>;

// TODO: tests
