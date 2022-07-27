mod conn;
mod error;
mod frame;

pub use conn::Connection;
pub use error::{ConnError, ConnResult};
pub use frame::Frame;

#[cfg(test)]
mod tests {
    // TODO: test other stuff?

    use super::*;

    #[test]
    fn display_frame() {
        let table = Frame::Table(vec![
            vec!["Name".into(), "Age".into()],
            vec!["Elliot".into(), "16".into()],
            vec!["Aurelius".into(), "0".into()],
        ]);
        let expected = r#"Name    |Age
--------+---
Elliot  |16 
Aurelius|0  
"#;
        assert_eq!(format!("{table}"), expected);
    }
}
