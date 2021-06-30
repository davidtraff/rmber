use protocol::{Packet, StringKey};
use schema::Schema;
use std::cell::RefCell;
use std::io::Error;
use tokio::net::TcpListener;
use tokio::sync::mpsc::UnboundedSender;
use tokio::{net::TcpStream, sync::mpsc::unbounded_channel};
use tokio_stream::{
    wrappers::{TcpListenerStream, UnboundedReceiverStream},
    StreamExt,
};

use crate::connection::{Connection, ConnectionId};
use crate::event_handlers::{connection_error, handle_new_connection, handle_packet, server_error};

type PacketTx = UnboundedSender<(ConnectionId, Result<Packet<StringKey>, Error>)>;

pub type ConnectionEvent = TcpStream;
pub type PacketEvent = (ConnectionId, Packet<StringKey>);
pub type ConnectionErrorEvent = (ConnectionId, Error);
pub type ServerErrorEvent = Error;

pub struct EventContext<'a> {
    connections: RefCell<&'a mut Vec<Connection>>,
    schema: RefCell<&'a mut Schema>,
    packet_tx: &'a PacketTx,
}

impl<'a> EventContext<'a> {
    pub fn new(
        connections: &'a mut Vec<Connection>,
        packet_tx: &'a PacketTx,
        schema: &'a mut Schema,
    ) -> EventContext<'a> {
        EventContext {
            connections: RefCell::new(connections),
            schema: RefCell::new(schema),
            packet_tx,
        }
    }

    pub fn add_connection(&self, connection: Connection) {
        let mut connections = self.connections.borrow_mut();

        connections.push(connection);
    }

    pub fn remove_connection(&self, connection_id: &ConnectionId) -> bool {
        let mut connections = self.connections.borrow_mut();

        let idx = connections
            .iter()
            .position(|c| c.id.eq(&connection_id));

        match idx {
            Some(idx) => {
                connections.remove(idx);

                true
            }
            None => false,
        }
    }

    pub fn emit_packet(&self, connection_id: &ConnectionId, packet: Packet<StringKey>) {
        match self.packet_tx.send((connection_id.clone(), Ok(packet))) {
            Ok(_) => {},
            Err(e) => {
                println!("Couldn't emit a packet on the bus {}", e);
            }
        }
    }

    pub fn emit_packet_error(&self, connection_id: &ConnectionId, error: std::io::Error) {
        match self.packet_tx.send((connection_id.clone(), Err(error))) {
            Ok(_) => {},
            Err(e) => {
                println!("Couldn't emit a packet-error on the bus {}", e);
            }
        }
    }

    pub fn get_packet_tx(&self) -> PacketTx {
        self.packet_tx.clone()
    }

    pub fn connections(&self) -> std::cell::Ref<&mut Vec<Connection>> {
        self.connections.borrow()
    }

    pub fn schema(&self) -> std::cell::Ref<&mut Schema> {
        self.schema.borrow()
    }

    pub fn replace_schema(&self, new_schema: Schema) -> Schema {
        let mut schema = self.schema.borrow_mut();

        std::mem::replace(*schema, new_schema)
    }
}

#[derive(Debug)]
enum Event {
    Connection(TcpStream),
    Packet((ConnectionId, Packet<StringKey>)),
    ConnectionError((ConnectionId, Error)),
    ServerError(Error),
}

pub struct Server {
    listener: TcpListenerStream,
    connections: Vec<Connection>,
    current_schema: Schema,
}

impl Server {
    pub fn new(listener: TcpListener) -> Self {
        Server {
            listener: TcpListenerStream::new(listener),
            connections: vec![],
            current_schema: Schema::empty(),
        }
    }

    pub async fn run(&mut self) {
        let (tx, rx) = unbounded_channel();
        let rx = UnboundedReceiverStream::new(rx);

        let listener = &mut self.listener;
        let new_connections = listener.map(|x| transform_connection(x));
        let packets = rx.map(|(id, packet)| transform_packet(id, packet));

        let mut events = new_connections.merge(packets);

        loop {
            let event = events.next().await;

            let ctx = EventContext::new(
                &mut self.connections,
                &tx,
                &mut self.current_schema,
            );

            match event {
                Some(Event::Connection(stream)) => {
                    handle_new_connection(ctx, stream);
                }
                Some(Event::Packet(e)) => {
                    handle_packet(ctx, e).await;
                }
                Some(Event::ConnectionError(e)) => {
                    connection_error(ctx, e);
                }
                Some(Event::ServerError(e)) => {
                    server_error(ctx, e);
                }
                None => break,
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
