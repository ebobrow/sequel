use crate::{
    frame::Frame,
    parse::{self, ParseError, ParseResult},
};

pub enum Command {
    Select { key: String, table: String },
    // Insert {
    //     key: String,
    //     value: Bytes,
    //     table: String,
    // },
}

impl Command {
    pub fn from_frame(frame: Frame) -> ParseResult<Command> {
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
            match parse::parse(stream) {
                Ok(_) => {
                    // TODO: actually parse
                    Ok(Command::Select {
                        key: "*".to_string(),
                        table: "people".to_string(),
                    })
                }
                Err(e) => Err(e),
            }
        } else {
            Err(ParseError::Internal)
        }
    }
}
