use std::io::Cursor;

use bytes::{Buf, Bytes};

use super::{ConnError, ConnResult};

pub enum Frame {
    Bulk(Bytes),
    Array(Vec<Frame>),
    Error(String),
    Null,
}

impl Frame {
    pub fn check(src: &mut Cursor<&[u8]>) -> ConnResult<()> {
        if !src.has_remaining() {
            Err(ConnError::Incomplete)
        } else {
            get_line(src).map(|_| ())
        }
    }

    pub fn parse(src: &mut Cursor<&[u8]>) -> ConnResult<Frame> {
        // for now
        Ok(Frame::Bulk(Bytes::copy_from_slice(get_line(src)?)))
        // Ok(Frame::String(
        //     String::from_utf8(get_line(src)?.to_vec()).unwrap(),
        // ))
    }
}

// fn get_word<'a>(src: &'a mut Cursor<&[u8]>) -> Result<&'a [u8], Error> {
//     let start = src.position() as usize;
//     let end = src.get_ref().len() - 1;

//     for i in start..end {
//         if src.get_ref()[i] == b' ' {
//             src.set_position((i + 1) as u64);
//             return Ok(&src.get_ref()[start..i]);
//         } else if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
//             src.set_position((i + 2) as u64);
//             return Ok(&src.get_ref()[start..i]);
//         }
//     }
//     Err(Error::Incomplete)
// }

fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> ConnResult<&'a [u8]> {
    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            src.set_position((i + 2) as u64);
            return Ok(&src.get_ref()[start..i]);
        }
    }

    Err(ConnError::Incomplete)
}
