use std::io::{self, Cursor};

use bytes::{Buf, Bytes, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

#[derive(Debug)]
pub enum Error {
    Incomplete,
    Other(crate::Error),
}
impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Incomplete => "stream ended early".fmt(fmt),
            Error::Other(err) => err.fmt(fmt),
        }
    }
}

// TODO: its own file
pub enum Command {
    Get(String),
    Set(String, Bytes),
}

impl Command {
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        if !src.has_remaining() {
            return Err(Error::Incomplete);
        }
        get_line(src).map(|_| ())
    }

    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Command, Error> {
        match get_word(src)? {
            b"get" => {
                let key = get_word(src)?;
                Ok(Command::Get(String::from_utf8(key.to_vec()).unwrap()))
            }
            b"set" => {
                let key = get_word(src)?.to_vec();
                let val = get_word(src)?;
                Ok(Command::Set(
                    String::from_utf8(key).unwrap(),
                    Bytes::copy_from_slice(val),
                ))
            }
            _ => Err(Error::Other("unsupported command".into())),
        }
    }
}

pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(4096),
        }
    }

    // TODO: anyhow or something
    // TODO: actual frames
    pub async fn read_frame(&mut self) -> crate::Result<Option<Command>> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    fn parse_frame(&mut self) -> crate::Result<Option<Command>> {
        let mut buf = Cursor::new(&self.buffer[..]);
        match Command::check(&mut buf) {
            Ok(_) => {
                let len = buf.position() as usize;
                buf.set_position(0);
                let frame = Command::parse(&mut buf)?;
                self.buffer.advance(len);
                Ok(Some(frame))
            }
            Err(Error::Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn write_frame(&mut self, frame: &Bytes) -> io::Result<()> {
        self.stream.write_all(frame).await?;
        self.stream.write_all(b"\r\n").await?;
        self.stream.flush().await
    }
}

fn get_word<'a>(src: &'a mut Cursor<&[u8]>) -> Result<&'a [u8], Error> {
    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b' ' {
            src.set_position((i + 1) as u64);
            return Ok(&src.get_ref()[start..i]);
        } else if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            src.set_position((i + 2) as u64);
            return Ok(&src.get_ref()[start..i]);
        }
    }
    Err(Error::Incomplete)
}

fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            src.set_position((i + 2) as u64);
            return Ok(&src.get_ref()[start..i]);
        }
    }

    Err(Error::Incomplete)
}
