use anyhow::{anyhow, Result};

use crate::{
    connection::Frame,
    db::{ColumnHeader, Db, DefaultOpt, Table},
    parse::{ColDecls, Token},
};

pub fn create_table(db: &Db, name: Token, col_decls: ColDecls) -> Result<Frame> {
    let mut col_headers = Vec::new();
    for (name, ty) in col_decls {
        col_headers.push(ColumnHeader::new(
            name.ident()
                .ok_or_else(|| anyhow!("Internal error"))?
                .to_string(),
            // TODO: keywords like `PRIMARY`
            DefaultOpt::None,
            ty,
        )?);
    }
    let table = Table::try_from(col_headers)?;

    let mut db = db.lock().unwrap();
    db.insert(
        name.ident()
            .ok_or_else(|| anyhow!("Internal error"))?
            .to_string(),
        table,
    );
    Ok(Frame::Null)
}
