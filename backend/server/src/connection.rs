use protocol::{Packet, StringKey};
use schema::QuerySet;
use std::cell::RefCell;
use std::io::{Error, ErrorKind};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::{
    net::TcpStream,
    sync::{mpsc::UnboundedSender, Mutex},
};

pub type ConnectionId = protocol::RawKey<8>;

#[derive(Debug)]
pub struct Connection {
    pub id: ConnectionId,
    pub address: SocketAddr,

    read: Arc<Mutex<OwnedReadHalf>>,
    write: RefCell<OwnedWriteHalf>,
    subscriptions: RefCell<QuerySet>,
    raw_schema: RefCell<Option<String>>,
}

impl Connection {
    pub fn new(stream: TcpStream, address: SocketAddr) -> Result<Self, Error> {
        let (read, write) = stream.into_split();

        match ConnectionId::new_random() {
            Ok(id) => Ok(Connection {
                id,
                read: Arc::new(Mutex::new(read)),
                // write: Arc::new(Mutex::new(write)),
                write: RefCell::new(write),
                address,
                subscriptions: RefCell::new(QuerySet::empty()),
                raw_schema: RefCell::new(None),
            }),
            Err(e) => Err(e),
        }
    }

    pub fn listen(&self, tx: UnboundedSender<(ConnectionId, Result<Packet<StringKey>, Error>)>) {
        let stream = self.read.clone();
        let id = self.id.clone();

        tokio::spawn(async move {
            let mut stream = stream.lock().await;

            loop {
                let packet = Packet::<StringKey>::read_from(&mut *stream).await;
                let mut error_kind = None;

                if let Err(e) = &packet {
                    error_kind = Some(e.kind());
                }

                match tx.send((id, packet)) {
                    Ok(_) => {}
                    Err(_) => break,
                }

                match error_kind {
                    Some(
                        ErrorKind::ConnectionReset
                        | ErrorKind::ConnectionAborted
                        | ErrorKind::ConnectionRefused
                        | ErrorKind::NotConnected,
                    ) => {
                        break;
                    }
                    _ => {}
                }
            }
        });
    }

    pub async fn write_packet(&self, packet: Packet<StringKey>) -> Result<(), Error> {
        let mut stream = self.write.borrow_mut();

        packet.write_to(&mut *stream).await?;

        Ok(())
    }

    pub async fn send_ok(&self) {
        let packet = Packet::Ok {};

        match self.write_packet(packet).await {
            Ok(_) => {}
            Err(e) => {
                println!(
                    "Could not send OK-packet to connection {}. Reason: {:?}",
                    self.id, e
                );
            }
        };
    }

    pub async fn send_err(&self, code: u32, error: &str) {
        let packet = Packet::Error {
            code,
            message: String::from(error),
        };

        match self.write_packet(packet).await {
            Ok(_) => {}
            Err(e) => {
                println!(
                    "Could not send ERR-packet to connection {}. Reason: {:?}",
                    self.id, e
                );
            }
        };
    }

    pub fn subscription_set(&self) -> std::cell::RefMut<QuerySet> {
        self.subscriptions.borrow_mut()
    }

    pub fn set_schema(&self, new_schema: String) {
        self.raw_schema.replace(Some(new_schema));
    }

    pub fn get_schema(&self) -> std::cell::Ref<Option<String>> {
        self.raw_schema.borrow()
    }
}
