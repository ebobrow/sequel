use anyhow::{anyhow, Result};

use crate::{
    connection::Frame,
    db::{ColumnHeader, Db, DefaultOpt, Table},
    parse::{ColDecl, Constraint, Expr, Token},
};

pub fn create_table(db: &Db, name: Token, col_decls: Vec<ColDecl>) -> Result<Frame> {
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
        // TODO: CREATE TABLE foo AS SELECT bar, baz FROM other
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
