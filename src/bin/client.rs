use std::io::{self, stdin, stdout, Write};

use sequel::connection::{Connection, Frame};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> io::Result<()> {
    let socket = TcpStream::connect("127.0.0.1:3000").await?;
    let mut connection = Connection::new(socket);

    loop {
        let mut s = String::new();
        print!("SQL> ");
        stdout().flush()?;
        stdin().read_line(&mut s)?;
        if let Some('\n') = s.chars().next_back() {
            s.pop();
        }
        if let Some('\r') = s.chars().next_back() {
            s.pop();
        }
        connection.write_frame(&Frame::Cmd(s.into())).await?;
        if let Some(response) = connection.read_frame().await.unwrap() {
            // TODO: it seems `INSERT` after `SELECT` hangs?
            println!("{}", response);
        }
    }
}
