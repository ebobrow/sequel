use std::io::{self, stdout, Cursor, Write};

use futures::{select, FutureExt};
use sequel::connection::Frame;
use tokio::{
    io::{stdin, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

// TODO: use `Connection`?
#[tokio::main]
async fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:3000").await.unwrap();
    let (mut rd, mut wr) = stream.split();

    let mut lines_from_stdin = BufReader::new(stdin()).lines();
    let mut buf = [0; 4096];

    print_prompt()?;
    loop {
        select! {
            n = rd.read(&mut buf).fuse() => {
                let mut cursor = Cursor::new(&buf[..n.unwrap()]);
                if let Ok(frame) = Frame::parse(&mut cursor) {
                    println!("{}", frame);
                }
                print_prompt()?;
            }
            // TODO: https://github.com/kkawakam/rustyline
            line = lines_from_stdin.next_line().fuse() => {
                let line = line?.unwrap();
                wr.write_u8(b':').await.unwrap();
                wr.write_all(line.as_bytes()).await.unwrap();
                wr.write_all(b"\r\n").await.unwrap();
                wr.flush().await.unwrap();
            }
        }
    }
}

fn print_prompt() -> io::Result<()> {
    print!("SQL> ");
    stdout().flush()?;
    Ok(())
}
