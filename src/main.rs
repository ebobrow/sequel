use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
};

use command::run_cmd;
use connection::Connection;
use data::{Db, Table};
use frame::Frame;
use tokio::net::{TcpListener, TcpStream};

use crate::data::ColumnHeader;

mod command;
mod connection;
mod data;
mod error;
mod frame;
mod parse;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    println!("Listening");

    let db: Db = Arc::new(Mutex::new(HashMap::from([(
        "people".into(),
        Table::try_from(vec![
            ColumnHeader::new("name".into()),
            ColumnHeader::new("age".into()),
        ])
        .unwrap(),
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

// TODO: `conn` mod or whatever name and move `Error`
async fn process(socket: TcpStream, db: Db) {
    let mut connection = Connection::new(socket);
    // TODO: write some sort of client so that we don't send whole SQL expressions through TCP
    while let Some(Frame::Bulk(stream)) = connection.read_frame().await.unwrap() {
        let response = run_cmd(&db, stream);
        connection.write_frame(&response).await.unwrap();
    }
}
