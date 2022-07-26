use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub use self::{
    row::{Column, ColumnHeader, Row},
    table::Table,
};

mod row;
mod table;

// TODO: Page how
// TODO: file persistence
pub type Db = Arc<Mutex<HashMap<String, Table>>>;

// TODO: tests
