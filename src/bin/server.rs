// TODO: move this to src/server.rs and only export `server::run`
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use sequel::{
    connection::{Connection, Frame},
    db::{ColumnHeader, Db, DefaultOpt, Table},
    run_cmd, LiteralValue, Ty,
};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    println!("Listening");

    let db = Arc::new(Mutex::new(HashMap::from([(
        "people".into(),
        Table::try_from(vec![
            ColumnHeader::new("name".into(), DefaultOpt::None, Ty::String).unwrap(),
            ColumnHeader::new("age".into(), DefaultOpt::None, Ty::Number).unwrap(),
            ColumnHeader::new("test".into(), DefaultOpt::Incrementing(0), Ty::Number).unwrap(),
            ColumnHeader::new(
                "three".into(),
                DefaultOpt::Some(LiteralValue::Number(3.0)),
                Ty::Number,
            )
            .unwrap(),
        ])?,
    )])));

    loop {
        let (socket, _) = listener.accept().await?;
        let db = db.clone();

        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: Db) {
    println!("Accepted");
    let mut connection = Connection::new(socket);
    while let Some(Frame::Cmd(cmd)) = connection.read_frame().await.unwrap() {
        let response = run_cmd(&db, cmd);
        connection.write_frame(&response).await.unwrap();
    }
    println!("Client disconnected");
}
