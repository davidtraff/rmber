use protocol::{Packet, StringKey, Value};
use schema::Schema;
use std::io::{Error, ErrorKind};
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

pub async fn poll(
    listener: &mut TcpListenerStream,
    connections: &mut Vec<Connection>,
    current_schema: &mut Schema,
) {
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
                    Packet::Subscribe { id } => {
                        let set = connection.subscription_set();

                        let ok = match set.insert_point(id.as_str()) {
                            Ok(_) => connection.write_packet(Packet::Ok {}),
                            Err(e) => connection.write_packet(Packet::Error {
                                value: Value::String(e.to_string()),
                            }),
                        }
                        .await;

                        match ok {
                            Ok(_) => {}
                            Err(e) => {
                                println!("Error when sending response to client: {}", e);
                            }
                        }
                    }
                    Packet::RegisterSchema { schema } => {
                        let schema = match schema {
                            Value::String(s) => s,
                            _ => {
                                connection.write_packet(Packet::Error { value: Value::String(String::from("Invalid schema-type")) }).await.unwrap();
                                continue;
                            }
                        };

                        connection.set_schema(schema);

                        match generate_schema(connections) {
                            Ok(mut new_schema) => {
                                std::mem::swap(current_schema, &mut new_schema);

                                connection.write_packet(Packet::Ok {})
                            },
                            Err(e) => {
                                connection.write_packet(Packet::Error { value: Value::String(e) })
                            }
                        }.await.unwrap();
                    }
                    Packet::Update {
                        id: _,
                        new_value: _,
                    } => todo!(),
                    Packet::List { id: _ } => todo!(),
                    Packet::Error { value: _ } => {
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

fn generate_schema(connections: &Vec<Connection>) -> Result<Schema, String> {
    let schemas= connections.iter()
        .map(|c| c.get_schema())
        .filter(|schema| schema.is_some())
        .map(|schema| schema.as_ref().unwrap().clone())
        .collect::<Vec<_>>()
        .join("\r\n");

    let ns = match schema::parse(&schemas) {
        Ok(ns) => ns,
        Err(e) => return Err(e.to_string())
    };
    
    Ok(Schema::new(ns))
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
