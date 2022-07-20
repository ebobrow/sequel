use bytes::Bytes;

#[derive(Eq)]
pub struct Row {
    // TODO: I don't like that we store this on every row
    primary_col: String,
    cols: Vec<Column>,
}

impl Row {
    pub fn new(primary_col: String, cols: Vec<Column>) -> Row {
        Row { primary_col, cols }
    }

    pub fn all_cols(&self) -> impl Iterator<Item = &Column> {
        self.cols.iter()
    }

    pub fn cols<'a>(&'a self, names: &'a Vec<String>) -> impl Iterator<Item = &'a Column> {
        self.cols
            .iter()
            .filter(|col| names.iter().any(|name| name == col.name()))
    }

    // TODO: Result?
    pub fn primary_col(&self) -> &Column {
        self.cols
            .iter()
            .find(|col| col.name == self.primary_col)
            .unwrap()
    }
}

impl PartialEq for Row {
    fn eq(&self, other: &Self) -> bool {
        self.primary_col() == other.primary_col()
    }
}

impl PartialOrd for Row {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.primary_col().partial_cmp(other.primary_col())
    }
}

impl Ord for Row {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.primary_col().cmp(other.primary_col())
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

#[derive(Eq)]
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
