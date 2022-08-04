use anyhow::{bail, Result};

use super::Token;

pub const ERROR_EOF: &'static str = "Unexpected end of file";

// TODO: This requires values for literal types like `Number`
pub fn throw_unexpected<T>(got: &Token, expected: Vec<Token>) -> Result<T> {
    let mut msg = format!("expected one of: {:?}", expected[0]);
    for ty in &expected[1..] {
        msg.push_str(&format!(", {:?}", ty)[..]);
    }
    bail!("Unexpected token: `{:#?}`; {}", got, msg);
}
