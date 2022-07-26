use std::io::{self, Cursor};

use bytes::{Buf, Bytes, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

use super::{ConnError, ConnResult, Frame};

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

    pub async fn read_frame(&mut self) -> ConnResult<Option<Frame>> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(ConnError::Reset);
                }
            }
        }
    }

    fn parse_frame(&mut self) -> ConnResult<Option<Frame>> {
        let mut buf = Cursor::new(&self.buffer[..]);
        match Frame::check(&mut buf) {
            Ok(_) => {
                let len = buf.position() as usize;
                buf.set_position(0);
                let frame = Frame::parse(&mut buf)?;
                self.buffer.advance(len);
                Ok(Some(frame))
            }
            Err(ConnError::Incomplete) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn write_frame(&mut self, frame: &Frame) -> io::Result<()> {
        match frame {
            Frame::Cmd(cmd) => {
                self.stream.write_u8(b':').await?;
                self.stream.write_all(cmd).await?;
            }
            Frame::Table(rows) => {
                self.stream.write_u8(b'*').await?;
                if let Some(first_row) = rows.first() {
                    self.write_table_row(first_row).await?;
                    for row in &rows[1..] {
                        self.stream.write_u8(b'^').await?;
                        self.write_table_row(row).await?;
                    }
                }
                self.stream.write_u8(b'*').await?;
            }
            Frame::Error(e) => {
                eprintln!("{}", e);
                self.stream.write_all(e.as_bytes()).await?;
            }
            Frame::Null => self.stream.write_all(b"-1").await?,
        }
        self.stream.write_all(b"\r\n").await?;
        self.stream.flush().await?;

        Ok(())
    }

    async fn write_table_row(&mut self, row: &[Bytes]) -> io::Result<()> {
        if let Some(first_item) = row.first() {
            self.stream.write_all(first_item).await?;
            for item in &row[1..] {
                self.stream.write_u8(b'|').await?;
                self.stream.write_all(item).await?;
            }
        }

        Ok(())
    }
}
