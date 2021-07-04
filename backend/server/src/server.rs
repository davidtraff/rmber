use protocol::{Packet, StringKey};
use store::ValueStore;
use store::rocksdb::DB;
use store::rocksdb::create_rocksdb;
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
type RocksDBStore = ValueStore<DB>;

pub type ConnectionEvent = TcpStream;
pub type PacketEvent = (ConnectionId, Packet<StringKey>);
pub type ConnectionErrorEvent = (ConnectionId, Error);
pub type ServerErrorEvent = Error;

pub type EventContext<'a> = (&'a mut RocksDBStore, &'a mut Vec<Connection>, &'a PacketTx);

// pub struct EventContext<'a> {
//     pub store: &'a mut RocksDBStore,
//     connections: &'a mut Vec<Connection>,
//     packet_tx: &'a PacketTx,
// }

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
    store: RocksDBStore,
}

impl Server {
    pub fn new(listener: TcpListener) -> Self {
        Server {
            listener: TcpListenerStream::new(listener),
            connections: vec![],
            store: create_rocksdb("./db"),
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

            let ctx: EventContext = (
                &mut self.store,
                &mut self.connections,
                &tx
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
