use std::io::{self, stdin, stdout, Write};

use sequel::connection::{Connection, Frame};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> io::Result<()> {
    let socket = TcpStream::connect("127.0.0.1:3000").await?;
    let mut connection = Connection::new(socket);

    print!("SQL> ");
    stdout().flush()?;
    for line in stdin().lines() {
        connection.write_frame(&Frame::Cmd(line?.into())).await?;
        if let Some(response) = connection.read_frame().await.unwrap() {
            // TODO: it seems `INSERT` after `SELECT` hangs?
            println!("{}", response);
        }

        print!("SQL> ");
        stdout().flush()?;
    }
    Ok(())
}
