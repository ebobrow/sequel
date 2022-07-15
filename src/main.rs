use std::{
    collections::{BTreeMap, HashMap},
    io,
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use command::run_cmd;
use connection::Connection;
use frame::Frame;
use tokio::net::{TcpListener, TcpStream};

mod command;
mod connection;
mod error;
mod frame;
mod parse;

pub type Db = Arc<Mutex<HashMap<String, BTreeMap<String, Bytes>>>>;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    println!("Listening");

    // let db = Db::default();
    let db = Arc::new(Mutex::new(HashMap::from([(
        String::from("people"),
        BTreeMap::from([
            (String::from("Elliot"), Bytes::from_static(b"Elliot 16")),
            (String::from("Joe"), Bytes::from_static(b"Joe 69")),
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
    // TODO: write some sort of client so that we don't send whole SQL expressions through TCP
    while let Some(Frame::Bulk(stream)) = connection.read_frame().await.unwrap() {
        let response = run_cmd(&db, stream);
        connection.write_frame(&response).await.unwrap();
    }
}
