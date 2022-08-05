use std::io::{self, Cursor};

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
async fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:3000").await.unwrap();
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
            let readline = rl.readline("SQL> ").unwrap();
            tx.send(readline).unwrap();
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
                let mut cursor = Cursor::new(&buf[..n.unwrap()]);
                if let Ok(frame) = Frame::parse(&mut cursor) {
                    println!("{}", frame);
                }
                tx2.send(true).unwrap();
            }
            res = rx.changed().fuse() => {
                tx2.send(false).unwrap();
                if res.is_ok() {
                    let line = rx.borrow();
                    wr.write_u8(b':').await.unwrap();
                    wr.write_all(line.as_bytes()).await.unwrap();
                    wr.write_all(b"\r\n").await.unwrap();
                    wr.flush().await.unwrap();
                } else {
                    break;
                }
            }
        }
    }

    Ok(())
}
