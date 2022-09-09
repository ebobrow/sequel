use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub use self::{
    column_header::{ColumnHeader, DefaultOpt},
    row::{Column, Row},
    table::Table,
};

mod column_header;
mod row;
mod table;

// TODO: file persistence
pub type Db = Arc<Mutex<HashMap<String, Table>>>;
