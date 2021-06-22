use schema::Schema;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;

use super::connection::Connection; 
use super::event_loop::poll;

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
        loop {
            poll(&mut self.listener, &mut self.connections, &mut self.current_schema).await;
        }
    }
}
