use std::{fmt::Display, io::Cursor};

use bytes::{Buf, Bytes};

use super::{ConnError, ConnResult};

#[derive(Debug, PartialEq)]
pub enum Frame {
    // Starts with `:`
    Cmd(Bytes),

    // `*1|2^3|4*` -> [[1,2],[3,4]]
    Table(Vec<Vec<Bytes>>),

    // Starts with `-`
    Error(String),

    // -1
    Null,
}

impl Frame {
    pub fn check(src: &mut Cursor<&[u8]>) -> ConnResult<()> {
        match get_u8(src)? {
            b':' => {
                get_line(src)?;
                Ok(())
            }
            b'*' => {
                get_until(src, b'*')?;
                Ok(())
            }
            b'-' => {
                get_line(src)?;
                Ok(())
            }
            c => Err(ConnError::Protocol(c)),
        }
    }

    pub fn parse(src: &mut Cursor<&[u8]>) -> ConnResult<Frame> {
        match get_u8(src)? {
            b':' => {
                let line = get_line(src)?;
                Ok(Frame::Cmd(Bytes::copy_from_slice(line)))
            }
            b'*' => {
                let table = std::str::from_utf8(get_until(src, b'*')?).unwrap();
                let mut rows = Vec::new();
                for row in table.split('^') {
                    let mut items = Vec::new();
                    for item in row.split('|') {
                        items.push(Bytes::copy_from_slice(item.as_bytes()));
                    }
                    rows.push(items);
                }
                Ok(Frame::Table(rows))
            }
            b'-' => {
                let line = get_line(src)?;
                if line == [b'1'] {
                    Ok(Frame::Null)
                } else {
                    Ok(Frame::Error(
                        String::from_utf8(line.to_vec())
                            .map_err(|_| ConnError::Protocol(line[0]))?,
                    ))
                }
            }
            c => Err(ConnError::Protocol(c)),
        }
    }
}

impl Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Frame::Cmd(cmd) => write!(f, "{}", std::str::from_utf8(cmd).unwrap()),
            Frame::Table(rows) => {
                let mut lengths = Vec::new();
                for i in 0..rows[0].len() {
                    lengths.push(rows.iter().map(|row| row[i].len()).max().unwrap());
                }
                let stringify_row = |row: &[Bytes]| -> String {
                    row.iter()
                        .enumerate()
                        .map(|(i, col)| {
                            let mut stringified = String::from_utf8(col.to_vec()).unwrap();
                            let padding = lengths[i] - stringified.len();
                            if i == 0 {
                                stringified = format!(" {stringified}");
                            }
                            stringified + &" ".repeat(padding)
                        })
                        .collect::<Vec<_>>()
                        .join(" | ")
                };

                writeln!(f, "{}", stringify_row(&rows[0]))?;
                writeln!(
                    f,
                    "{}",
                    lengths
                        .iter()
                        .map(|length| "-".repeat(*length + 2))
                        .collect::<Vec<_>>()
                        .join("+")
                )?;
                for row in &rows[1..] {
                    writeln!(f, "{}", stringify_row(row))?;
                }

                Ok(())
            }
            Frame::Error(e) => write!(f, "Error: {}", e),
            Frame::Null => write!(f, "NULL"),
        }
    }
}

fn get_u8(src: &mut Cursor<&[u8]>) -> ConnResult<u8> {
    if !src.has_remaining() {
        return Err(ConnError::Incomplete);
    }

    Ok(src.get_u8())
}

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

fn get_until<'a>(src: &mut Cursor<&'a [u8]>, c: u8) -> ConnResult<&'a [u8]> {
    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == c {
            src.set_position(i as u64);
            return Ok(&src.get_ref()[start..i]);
        }
    }

    Err(ConnError::Incomplete)
}
