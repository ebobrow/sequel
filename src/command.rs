use bytes::Bytes;

use crate::frame::Frame;

pub enum Command {
    Get(String),
    Set(String, Bytes),
}

impl Command {
    pub fn from_frame(frame: Frame) -> Option<Command> {
        // TODO: array?
        if let Frame::String(string) = frame {
            let (cmd, rest) = string.split_once(' ')?;
            match cmd {
                "get" => Some(Command::Get(rest.to_string())),
                "set" => {
                    let (key, val) = rest.split_once(' ')?;
                    Some(Command::Set(
                        key.to_string(),
                        Bytes::copy_from_slice(val.as_bytes()),
                    ))
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
