use std::io::Cursor;

use anyhow::Result;
use futures::{select, FutureExt};
use rustyline::{Config, EditMode, Editor};
use sequel::connection::Frame;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::watch,
};

// TODO: use `Connection`?
#[tokio::main]
async fn main() -> Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:3000").await?;
    let (mut rd, mut wr) = stream.split();

    let (tx, mut rx) = watch::channel(String::new());
    // TODO: this is pretty bad
    let (tx2, mut rx2) = watch::channel(false);
    tokio::spawn(async move {
        let mut rl = Editor::<()>::with_config(
            Config::builder()
                .auto_add_history(true)
                .edit_mode(EditMode::Emacs)
                .build(),
        )
        .unwrap();

        loop {
            if let Ok(readline) = rl.readline("SQL> ") {
                tx.send(readline).unwrap();
            } else {
                break;
            }
            loop {
                rx2.changed().await.unwrap();
                if *rx2.borrow() {
                    break;
                }
            }
        }
    });

    let mut buf = [0; 4096];

    loop {
        select! {
            n = rd.read(&mut buf).fuse() => {
                let mut cursor = Cursor::new(&buf[..n?]);
                if let Ok(frame) = Frame::parse(&mut cursor) {
                    println!("{}", frame);
                }
                tx2.send(true)?;
            }
            res = rx.changed().fuse() => {
                if res.is_ok() {
                    tx2.send(false)?;
                    let line = rx.borrow();
                    wr.write_u8(b':').await?;
                    wr.write_all(line.as_bytes()).await?;
                    wr.write_all(b"\r\n").await?;
                    wr.flush().await?;
                } else {
                    println!("bye");
                    break;
                }
            }
        }
    }

    Ok(())
}
