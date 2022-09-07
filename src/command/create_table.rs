use anyhow::{anyhow, Result};

use crate::{
    connection::Frame,
    db::{ColumnHeader, Db, DefaultOpt, Table},
    parse::{ColDecl, Constraint, Token},
};

pub fn create_table(db: &Db, name: Token, col_decls: Vec<ColDecl>) -> Result<Frame> {
    let mut col_headers = Vec::new();
    for col_decl in col_decls {
        for constraint in col_decl.constraints() {
            match constraint {
                Constraint::NotNull => unimplemented!(),
                Constraint::Unique => unimplemented!(),
                Constraint::PrimaryKey => {}
                Constraint::ForeignKey => unimplemented!(),
                Constraint::Check => unimplemented!(),
                Constraint::Default => unimplemented!(),
                Constraint::CreateIndex => unimplemented!(),
            }
        }
        col_headers.push(ColumnHeader::new(
            col_decl.ident()?.to_string(),
            // TODO: CREATE TABLE foo AS SELECT bar, baz FROM other
            DefaultOpt::None,
            col_decl.ty().clone(),
            col_decl.constraints().contains(&Constraint::PrimaryKey),
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
