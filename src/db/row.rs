use bytes::Bytes;

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
