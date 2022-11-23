use anyhow::{anyhow, bail, Ok, Result};

use crate::{
    connection::Frame,
    db::{ColumnHeader, Db, DefaultOpt, Table},
    parse::{ColDecl, Command, Constraint, Expr, Key, TableDef, Token},
};

pub fn create_table(db: &Db, name: Token, def: TableDef) -> Result<Frame> {
    match def {
        TableDef::Cols(col_decls) => from_col_decls(db, name, col_decls),
        TableDef::As(cmd) => from_other(db, name, *cmd),
    }
}

fn from_col_decls(db: &Db, name: Token, col_decls: Vec<ColDecl>) -> Result<Frame> {
    let mut col_headers = Vec::new();
    for col_decl in col_decls {
        for constraint in col_decl.constraints() {
            match constraint {
                Constraint::NotNull => {}
                Constraint::Unique => {}
                Constraint::PrimaryKey => {}
                Constraint::ForeignKey => unimplemented!(),
                Constraint::Check(_) => {}
                Constraint::Default(_) => {}
                Constraint::CreateIndex => unimplemented!(),
            }
        }
        col_headers.push(
            ColumnHeader::new(col_decl.ident()?.to_string())
                .ty(col_decl.ty().clone())
                .primary_key(col_decl.constraints().contains(&Constraint::PrimaryKey))
                .unique(col_decl.constraints().contains(&Constraint::Unique))
                .not_null(col_decl.constraints().contains(&Constraint::NotNull))
                .def(extract_default(col_decl.constraints()))
                .check(extract_check(col_decl.constraints()))
                .build()?,
        );
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

fn from_other(db: &Db, name: Token, command: Command) -> Result<Frame> {
    if let Command::Select { key, table } = command {
        let mut db = db.lock().unwrap();
        let table_name = table.ident().ok_or_else(|| anyhow!("Internal error"))?;
        let table = db
            .get(table_name)
            .ok_or_else(|| anyhow!("Table \"{}\" not found", table_name))?;
        let names = match &key {
            Key::Glob => None,
            Key::List(names) => Some(
                names
                    .iter()
                    .map(|col| {
                        Ok(col
                            .ident()
                            .ok_or_else(|| anyhow!("Internal error"))?
                            .to_string())
                    })
                    .collect::<Result<Vec<_>>>()?,
            ),
        };
        let headers = match &names {
            None => table.col_headers().to_vec(),
            Some(names) => table
                .col_headers()
                .iter()
                .filter(|header| names.contains(&header.name().to_owned()))
                .cloned()
                .collect(),
        };
        let mut new_table = Table::try_from(headers)?;
        for row in table.rows() {
            let cols = match &names {
                None => row.all_cols(),
                Some(names) => row
                    .all_cols()
                    .into_iter()
                    .filter(|col| names.contains(&col.name().to_owned()))
                    .collect(),
            };
            new_table.append(cols)?;
        }

        db.insert(
            name.ident()
                .ok_or_else(|| anyhow!("Internal error"))?
                .to_string(),
            new_table,
        );
        Ok(Frame::Null)
    } else {
        bail!("expected `SELECT`");
    }
}

fn extract_default(constraints: &[Constraint]) -> DefaultOpt {
    constraints
        .iter()
        .find_map(|constraint| {
            if let Constraint::Default(lit) = constraint {
                // TODO: will it ever be `DefaultOpt::Increment(_)`?
                // also stuff like `GETDATE()`
                Some(DefaultOpt::Some(lit.clone()))
            } else {
                None
            }
        })
        .unwrap_or(DefaultOpt::None)
}

fn extract_check(constraints: &[Constraint]) -> Option<Expr> {
    constraints.iter().find_map(|constraint| {
        if let Constraint::Check(expr) = constraint {
            Some(expr.clone())
        } else {
            None
        }
    })
}
