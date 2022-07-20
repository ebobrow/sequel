use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use command::run_cmd;
use connection::{Connection, Frame};
use db::{Column, ColumnHeader, Db, Table};
use tokio::net::{TcpListener, TcpStream};

mod command;
mod connection;
mod db;
mod parse;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    println!("Listening");

    let db = {
        let mut table = Table::try_from(vec![
            ColumnHeader::new("name".into()),
            ColumnHeader::new("age".into()),
        ])
        .unwrap();
        table.append(vec![
            Column::new(Bytes::from("Elliot"), "name".into()),
            Column::new(Bytes::from("16"), "age".into()),
        ]);
        Arc::new(Mutex::new(HashMap::from([("people".into(), table)])))
    };

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
    // and change from `bin` to `lib` and have client and server binaries?
    while let Some(Frame::Bulk(stream)) = connection.read_frame().await.unwrap() {
        let response = run_cmd(&db, stream);
        connection.write_frame(&response).await.unwrap();
    }
}
