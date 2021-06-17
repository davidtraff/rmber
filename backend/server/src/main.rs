use tokio::net::TcpListener;
use std::net::SocketAddr;

mod server;
mod connection;
mod event_loop;

use server::*;

#[tokio::main]
pub async fn main() -> std::io::Result<()> {
    let address = SocketAddr::new("127.0.0.1".parse().unwrap(), 8080);
    let listener = TcpListener::bind(address).await?;
    let mut server = Server::new(listener);

    server.run().await;

    Ok(())
}
