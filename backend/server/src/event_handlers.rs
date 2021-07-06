use std::io::{Error, ErrorKind};

use protocol::{PACKET_SCHEMA_ERR, PACKET_SUBSCRIPTION_ERR, PACKET_UPDATE_ERR, Packet};

use crate::{
    connection::Connection,
    server::{ConnectionErrorEvent, ConnectionEvent, EventContext, PacketEvent, ServerErrorEvent},
};

pub fn handle_new_connection((_, connections, tx, _): EventContext, connection: ConnectionEvent) {
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

    connection.listen(tx.clone());
    println!("New connection {}", &connection.id);

    connections.push(connection);
}

pub async fn handle_packet((store, connections, packet_tx, point_tx): EventContext<'_>, (id, packet): PacketEvent) {
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
                Err(e) => connection.send_err(PACKET_SUBSCRIPTION_ERR, &e.to_string()).await,
            };
        }
        Packet::RegisterSchema { schema } => {
            connection.set_schema(schema);

            let schemas = connections.iter()
                .map(|c| c.get_schema())
                .filter(|c| c.is_some())
                .map(|c| c.as_ref().unwrap().clone());

            if let Err(e) = store.build_schema(schemas) {
                connection.send_err(PACKET_SCHEMA_ERR, &e.to_string()).await;
            } else {
                connection.send_ok().await;
            }
        }
        Packet::Update {
            id,
            new_value,
        } => {
            match store.update_point(&id, new_value).await {
                Ok(value) => {
                    point_tx.send((id, value)).unwrap();
                    connection.send_ok().await;
                },
                Err(e) => connection.send_err(PACKET_UPDATE_ERR, &e.to_string()).await,
            }
        },
        Packet::Error { code: _, message: _ } => {
            // In this case we emit a disconnect.
            let msg = (
                id,
                Err(Error::new(ErrorKind::ConnectionAborted, "Client error."))
            );

            packet_tx.send(msg).unwrap();
        },
        _ => {}
    }
}

pub fn connection_error((_, connections, _, _): EventContext, (id, e): ConnectionErrorEvent) {
    println!("Connection-error {:?}", e);

    match e.kind() {
        ErrorKind::ConnectionReset | ErrorKind::ConnectionAborted | ErrorKind::ConnectionRefused => {
            println!("Removing connection {}", &id);
            if let Some(idx) = connections.iter().position(|c| c.id.eq(&id)) {
                connections.remove(idx);
            }
        }
        _ => {}
    }
}

pub fn server_error(_: EventContext, event: ServerErrorEvent) {
    println!("Server-error {:?}", event);
}
