use bytes::Bytes;

#[derive(PartialEq)]
pub struct Row {
    cols: Vec<Column>,
}

impl Row {
    pub fn new(cols: Vec<Column>) -> Row {
        Row { cols }
    }

    pub fn cols(&self) -> &[Column] {
        self.cols.as_ref()
    }
}

// TODO: designate incrementing, default
#[derive(Clone)]
pub struct ColumnHeader {
    name: String,
    is_primary_key: bool,
}

impl ColumnHeader {
    pub fn new(name: String) -> ColumnHeader {
        ColumnHeader {
            name,
            is_primary_key: false,
        }
    }

    pub fn new_prinary(name: String) -> ColumnHeader {
        ColumnHeader {
            name,
            is_primary_key: true,
        }
    }

    pub fn is_primary(&self) -> bool {
        self.is_primary_key
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

// TODO: what goes here
#[derive(PartialEq)]
pub struct Column {
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
