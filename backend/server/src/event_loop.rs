use std::io::{Error, ErrorKind};
use storage::{Packet, StringKey, Value};
use tokio::{net::TcpStream, sync::mpsc::unbounded_channel};
use tokio_stream::{
    wrappers::{TcpListenerStream, UnboundedReceiverStream},
    StreamExt,
};

use crate::connection::{Connection, ConnectionId};

#[derive(Debug)]
pub enum Event {
    Connection(TcpStream),
    Packet((ConnectionId, Packet<StringKey>)),
    ConnectionError((ConnectionId, Error)),
    ServerError(Error),
}

pub async fn poll(listener: &mut TcpListenerStream, connections: &mut Vec<Connection>) {
    let (tx, rx) = unbounded_channel();
    let rx = UnboundedReceiverStream::new(rx);

    let new_connections = listener.map(|x| transform_connection(x));
    let packets = rx.map(|(id, packet)| transform_packet(id, packet));

    let mut events = new_connections.merge(packets);

    loop {
        let event = events.next().await;

        match event {
            Some(Event::Connection(stream)) => {
                let address = match stream.peer_addr() {
                    Ok(addr) => addr,
                    Err(_) => continue,
                };

                let connection = match Connection::new(stream, address) {
                    Ok(conn) => conn,
                    Err(e) => {
                        println!("{:?}", e);
                        continue;
                    }
                };

                connection.listen(tx.clone());

                connections.push(connection);

                println!("New connection");
            }
            Some(Event::Packet((id, packet))) => {
                let connection = connections.iter().find(|x| x.id.eq(&id));

                let connection = match connection {
                    Some(c) => c,
                    None => continue,
                };

                match packet {
                    Packet::Subscribe { token, id } => {
                        let added = connection.add_subscription_point(id);

                        match added {
                            true => connection.write_packet(Packet::Ok { token }),
                            false => connection.write_packet(Packet::Error { token, value: Value::String(String::from("Already subscribed.")) })
                        }.await.unwrap();
                    }
                    Packet::Update {
                        token,
                        id,
                        new_value,
                    } => todo!(),
                    Packet::List { token, id } => todo!(),
                    Packet::Error { token: _, value: _ } => {
                        // In this case we emit a disconnect.
                        tx.send((
                            id,
                            Err(Error::new(ErrorKind::ConnectionAborted, "Client error.")),
                        ))
                        .unwrap();
                    }
                    _ => continue,
                }
            }
            Some(Event::ConnectionError((connection, e))) => {
                println!("Error {:?}", &e);
                match e.kind() {
                    ErrorKind::ConnectionReset => {
                        let idx = connections.iter().position(|x| x.id.eq(&connection));

                        match idx {
                            Some(idx) => {
                                connections.remove(idx);
                                println!("Removed connection {}", connection);
                            }
                            None => {
                                println!(
                                    "Couldn't remove the connection {}, {:?}",
                                    connection, connections
                                );
                            }
                        };
                    }
                    _ => {}
                }
            }
            Some(Event::ServerError(e)) => {
                println!("Server-error {:?}", e);
            }
            _ => {
                println!("asd");
            }
        }
    }
}

fn transform_connection(data: std::io::Result<TcpStream>) -> Event {
    match data {
        Ok(stream) => Event::Connection(stream),
        Err(e) => Event::ServerError(e),
    }
}

fn transform_packet(
    connection_id: ConnectionId,
    packet: Result<Packet<StringKey>, Error>,
) -> Event {
    match packet {
        Ok(packet) => Event::Packet((connection_id, packet)),
        Err(e) => Event::ConnectionError((connection_id, e)),
    }
}
