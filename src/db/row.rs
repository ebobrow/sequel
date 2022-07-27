use bytes::Bytes;

#[derive(Eq)]
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

    pub fn all_cols(&self) -> impl Iterator<Item = &Column> {
        self.cols.iter().chain([&self.primary_key_col])
    }

    pub fn cols<'a>(&'a self, names: &'a [String]) -> impl Iterator<Item = &'a Column> {
        self.all_cols()
            .filter(|col| names.iter().any(|name| name == col.name()))
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

#[derive(Eq, Clone)]
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
