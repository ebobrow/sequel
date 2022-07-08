use bytes::Bytes;

use crate::{frame::Frame, parse};

pub enum Command {
    Select { key: String, table: String },
    // Insert {
    //     key: String,
    //     value: Bytes,
    //     table: String,
    // },
}

impl Command {
    pub fn from_frame(frame: Frame) -> Option<Command> {
        // TODO: array?
        // TODO: full parser: tokenizer, etc. etc..
        if let Frame::Bulk(stream) = frame {
            // let (cmd, rest) = string.split_once(' ')?;
            // match cmd {
            //     "INSERT" => todo!(),
            //     "SELECT" => {
            //         // * FROM people
            //         let (key, table) = rest.split_once(" FROM ")?;
            //         Some(Command::Select {
            //             key: key.to_string(),
            //             table: table.to_string(),
            //         })
            //     }
            //     _ => None,
            // }
            parse::parse(stream);
            Some(Command::Select {
                key: "*".to_string(),
                table: "people".to_string(),
            })
        } else {
            None
        }
    }
}
