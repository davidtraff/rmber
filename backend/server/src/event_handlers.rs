use std::io::{Error, ErrorKind};

use protocol::{Packet, Value};
use schema::Schema;

use crate::{
    connection::Connection,
    server::{ConnectionErrorEvent, ConnectionEvent, EventContext, PacketEvent, ServerErrorEvent},
};

pub fn handle_new_connection(ctx: EventContext, connection: ConnectionEvent) {
    let address = match connection.peer_addr() {
        Ok(addr) => addr,
        Err(_) => return,
    };

    let connection = match Connection::new(connection, address) {
        Ok(conn) => conn,
        Err(e) => {
            println!("{:?}", e);
            return;
        }
    };

    connection.listen(ctx.get_packet_tx());
    println!("New connection {}", &connection.id);

    ctx.add_connection(connection);
}

pub async fn handle_packet(ctx: EventContext<'_>, (id, packet): PacketEvent) {
    let connections = ctx.connections();
    let connection = connections.iter().find(|c| c.id.eq(&id));
    let connection = match connection {
        Some(c) => c,
        None => return,
    };

    match packet {
        Packet::Subscribe { id } => {
            let mut set = connection.subscription_set();

            match set.insert_point(id.as_str()) {
                Ok(_) => connection.send_ok().await,
                Err(e) => connection.send_err(&e.to_string()).await,
            };
        }
        Packet::RegisterSchema { schema } => {
            let schema = match schema {
                Value::String(s) => s,
                _ => {
                    connection
                        .send_err("Invalid schema-type. Expected String")
                        .await;
                    return;
                }
            };

            connection.set_schema(schema);

            match generate_schema(&*ctx.connections()) {
                Ok(new_schema) => {
                    dbg!(&new_schema);
                    ctx.replace_schema(new_schema);

                    connection.send_ok().await;
                }
                Err(e) => {
                    connection.send_err(&e).await;
                }
            };
        }
        Packet::Update {
            id: _,
            new_value: _,
        } => todo!(),
        Packet::Error { value: _ } => {
            // In this case we emit a disconnect.
            ctx.emit_packet_error(
                &id,
                Error::new(ErrorKind::ConnectionAborted, "Client error."),
            );
        }
        _ => {}
    }
}

fn generate_schema(connections: &Vec<Connection>) -> Result<Schema, String> {
    let schemas = connections
        .iter()
        .map(|c| c.get_schema())
        .filter(|schema| schema.is_some())
        .map(|schema| schema.as_ref().unwrap().clone())
        .collect::<Vec<_>>()
        .join("\r\n");

    let ns = match schema::parse(&schemas) {
        Ok(ns) => ns,
        Err(e) => return Err(e.to_string()),
    };

    Ok(Schema::new(ns))
}

pub fn connection_error(ctx: EventContext, (id, e): ConnectionErrorEvent) {
    println!("Connection-error {:?}", e);

    match e.kind() {
        ErrorKind::ConnectionReset => {
            if !ctx.remove_connection(&id) {
                println!("Couldn't remove connection {}", id);
            }
        }
        _ => {}
    }
}

pub fn server_error(_: EventContext, event: ServerErrorEvent) {
    println!("Server-error {:?}", event);
}
