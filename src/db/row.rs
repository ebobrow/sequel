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

    pub fn cols(&self, names: &[String]) -> Vec<Bytes> {
        let mut cols = Vec::new();
        let all_col_names = self
            .cols
            .clone()
            .into_iter()
            .chain([self.primary_key_col.clone()]);
        for name in names {
            cols.push(
                all_col_names
                    .clone()
                    .find(|col| col.name() == name)
                    // TODO: error--feels weird having separate db and command errors. consolidate
                    // or remove both and just use strings?
                    .unwrap()
                    .data,
            );
        }
        cols
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
    Some(Bytes),
    Incrementing(u8),
}

#[derive(Clone)]
pub struct ColumnHeader {
    name: String,
    is_primary_key: bool,
    is_hidden: bool,
    default: DefaultOpt,
}

impl ColumnHeader {
    pub fn new(name: String, default: DefaultOpt) -> ColumnHeader {
        ColumnHeader {
            name,
            is_primary_key: false,
            is_hidden: false,
            default,
        }
    }

    pub fn new_prinary(name: String, default: DefaultOpt) -> ColumnHeader {
        ColumnHeader {
            name,
            is_primary_key: true,
            is_hidden: false,
            default,
        }
    }

    pub fn new_hidden() -> ColumnHeader {
        ColumnHeader {
            name: "ID".into(),
            is_primary_key: true,
            is_hidden: true,
            default: DefaultOpt::Incrementing(0),
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
}

#[derive(Eq, Clone, Debug)]
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
