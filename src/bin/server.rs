// TODO: move this to src/server.rs and only export `server::run`
use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
};

use sequel::{
    connection::{Connection, Frame},
    db::{ColumnHeader, Db, DefaultOpt, Table},
    run_cmd,
};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    println!("Listening");

    let db = Arc::new(Mutex::new(HashMap::from([(
        "people".into(),
        Table::try_from(vec![
            ColumnHeader::new("name".into(), DefaultOpt::None),
            ColumnHeader::new("age".into(), DefaultOpt::None),
            ColumnHeader::new("test".into(), DefaultOpt::Incrementing(0)),
            ColumnHeader::new("three".into(), DefaultOpt::Some("3".into())),
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

async fn process(socket: TcpStream, db: Db) {
    let mut connection = Connection::new(socket);
    while let Some(Frame::Cmd(cmd)) = connection.read_frame().await.unwrap() {
        let response = run_cmd(&db, cmd);
        connection.write_frame(&response).await.unwrap();
    }
}
