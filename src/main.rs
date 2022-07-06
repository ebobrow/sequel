use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use connection::{Command, Connection};
use tokio::net::{TcpListener, TcpStream};

mod connection;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    println!("Listening");

    let db = Db::default();

    loop {
        let (socket, _) = listener.accept().await?;
        let db = db.clone();

        println!("Accepted");
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: Db) {
    let mut connection = Connection::new(socket);
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match frame {
            Command::Set(key, val) => {
                let mut db = db.lock().unwrap();
                db.insert(key, val);
                Bytes::copy_from_slice("OK".as_bytes())
            }
            Command::Get(key) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(&key) {
                    value.clone()
                } else {
                    Bytes::copy_from_slice("NULL".as_bytes())
                }
            }
        };
        connection.write_frame(&response).await.unwrap();
    }
}
