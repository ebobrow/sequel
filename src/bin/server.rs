use anyhow::Result;
use sequel::{
    connection::{Connection, Frame},
    run_cmd, Db,
};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    println!("Listening");

    let db = Db::default();

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
