use tokio::net::TcpListener;
use std::net::SocketAddr;
use storage::{Packet, StringKey};

#[tokio::main]
pub async fn main() -> std::io::Result<()> {
    let address = SocketAddr::new("127.0.0.1".parse().unwrap(), 8080);
    let listener = TcpListener::bind(address).await?;

    println!("Listening...");

    loop {
        let (mut socket, _) = listener.accept().await?;

        let packet = match Packet::<StringKey>::read_from(&mut socket).await {
            Ok(packet) => packet,
            Err(e) => {
                dbg!(&e);
                let packet = Packet::<StringKey>::Error {value: storage::Value::String(e.message)};

                match packet.write_to(&mut socket).await {
                    Ok(_) => {},
                    Err(e) => {
                        dbg!("Could not send error to client: {:?}", e);
                    }
                }

                continue;      
            },
        };

        dbg!(&packet);

        match packet.write_to(&mut socket).await {
            Ok(_) => {},
            Err(e) => {
                dbg!("Could not send response to client: {:?}", e);
                continue;
            }
        }
    }
}
