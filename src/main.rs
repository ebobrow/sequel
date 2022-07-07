use std::{
    collections::{BTreeMap, HashMap},
    io,
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use command::Command;
use connection::Connection;
use frame::Frame;
use tokio::net::{TcpListener, TcpStream};

mod command;
mod connection;
mod frame;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

// TODO: `Page` type
type Db = Arc<Mutex<HashMap<String, BTreeMap<String, Bytes>>>>;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    println!("Listening");

    // let db = Db::default();
    let db = Arc::new(Mutex::new(HashMap::from([(
        String::from("people"),
        BTreeMap::from([
            (String::from("Elliot"), Bytes::from_static(b"16")),
            (String::from("Joe"), Bytes::from_static(b"69")),
        ]),
    )])));

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
        let response = match Command::from_frame(frame).unwrap() {
            // Command::Set(key, val) => {
            //     let mut db = db.lock().unwrap();
            //     db.insert(key, val);
            //     Frame::String("OK".to_string())
            // }
            Command::Select { key, table } => {
                let db = db.lock().unwrap();
                if let Some(table) = db.get(&table) {
                    match &key[..] {
                        "*" => {
                            // TODO: array type?
                            let mut ret = String::new();
                            for (key, val) in table {
                                ret.push_str(&format!("{} {:?}\n", key, val)[..]);
                            }
                            Frame::String(ret)
                        }
                        _ => {
                            if let Some((k, v)) = table.iter().find(|(k, _)| **k == key) {
                                Frame::String(format!("{} {:?}", k, v))
                            } else {
                                Frame::Null
                            }
                        }
                    }
                } else {
                    // TODO: Frame::Error?
                    Frame::Null
                }
            }
        };
        connection.write_frame(&response).await.unwrap();
    }
}
