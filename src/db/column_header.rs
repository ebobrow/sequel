use anyhow::{anyhow, Result};

use crate::{
    parse::{Expr, LiteralValue},
    Ty,
};

#[derive(Clone, PartialEq)]
pub enum DefaultOpt {
    None,
    Some(LiteralValue),
    Incrementing(u8),
}

// TODO: macro for this
pub struct ColumnHeaderBuilder {
    name: String,
    is_primary_key: bool,
    is_hidden: bool,
    not_null: bool,
    unique: bool,
    default: DefaultOpt,
    check: Option<Expr>,
    ty: Option<Ty>,
}

impl ColumnHeaderBuilder {
    fn new(name: String) -> Self {
        ColumnHeaderBuilder {
            name,
            is_primary_key: false,
            is_hidden: false,
            not_null: false,
            unique: false,
            default: DefaultOpt::None,
            check: None,
            ty: None,
        }
    }

    pub fn primary_key(mut self, is_primary_key: bool) -> Self {
        self.is_primary_key = is_primary_key;
        self
    }

    pub fn hidden(mut self, is_hidden: bool) -> Self {
        self.is_hidden = is_hidden;
        self
    }

    pub fn not_null(mut self, is_not_null: bool) -> Self {
        self.not_null = is_not_null;
        self
    }

    pub fn unique(mut self, unique: bool) -> Self {
        self.unique = unique;
        self
    }

    pub fn def(mut self, def: DefaultOpt) -> Self {
        self.default = def;
        self
    }

    pub fn check(mut self, check: Option<Expr>) -> Self {
        self.check = check;
        self
    }

    pub fn ty(mut self, ty: Ty) -> Self {
        self.ty = Some(ty);
        self
    }

    pub fn build(self) -> Result<ColumnHeader> {
        if let Some(ty) = self.ty {
            let print_err = |def_ty| {
                anyhow!(
                    "Default type doesn't match declared type; expected {}, got {:?}",
                    def_ty,
                    ty
                )
            };
            match &self.default {
                DefaultOpt::None => {}
                DefaultOpt::Some(val) => match val {
                    LiteralValue::String(_) => {
                        if ty != Ty::String {
                            return Err(print_err("String"));
                        }
                    }
                    LiteralValue::Number(_) => {
                        if ty != Ty::Number {
                            return Err(print_err("Number"));
                        }
                    }
                    LiteralValue::Bool(_) => {
                        if ty != Ty::Bool {
                            return Err(print_err("Number"));
                        }
                    }
                    LiteralValue::Null => unreachable!(),
                },
                DefaultOpt::Incrementing(_) => {
                    if ty != Ty::Number {
                        return Err(print_err("Number"));
                    }
                }
            };
            Ok(ColumnHeader {
                name: self.name,
                is_primary_key: self.is_primary_key,
                is_hidden: self.is_hidden,
                default: self.default,
                not_null: self.not_null,
                unique: self.unique,
                check: self.check,
                ty,
            })
        } else {
            Err(anyhow!("Type not specified"))
        }
    }
}

#[derive(Clone)]
pub struct ColumnHeader {
    name: String,
    is_primary_key: bool,
    is_hidden: bool,
    not_null: bool,
    unique: bool,
    default: DefaultOpt,
    check: Option<Expr>,
    ty: Ty,
}

impl ColumnHeader {
    pub fn new(name: String) -> ColumnHeaderBuilder {
        ColumnHeaderBuilder::new(name)
    }

    pub fn new_hidden() -> ColumnHeader {
        ColumnHeader {
            name: "ID".into(),
            is_primary_key: true,
            is_hidden: true,
            not_null: true,
            unique: true,
            default: DefaultOpt::Incrementing(0),
            check: None,
            ty: Ty::Number,
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

    pub fn ty(&self) -> &Ty {
        &self.ty
    }

    pub fn not_null(&self) -> bool {
        self.not_null
    }

    pub fn unique(&self) -> bool {
        self.unique
    }

    pub fn check(&self) -> Option<&Expr> {
        self.check.as_ref()
    }
}
