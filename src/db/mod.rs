use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub use self::{
    row::{Column, ColumnHeader, DefaultOpt, Row},
    table::Table,
};

mod row;
mod table;

// TODO: file persistence
pub type Db = Arc<Mutex<HashMap<String, Table>>>;
