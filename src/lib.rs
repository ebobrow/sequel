// TODOs:
// - [ ] Get rid of `unwrap` and `expect` where possible

mod command;
pub mod connection;
// TODO: this won't have to be public after `CREATE TABLE` command
pub mod db;
mod parse;

pub use command::run_cmd;
