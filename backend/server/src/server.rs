use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;

use super::connection::Connection; 
use super::event_loop::poll;

pub struct Server {
    listener: TcpListenerStream,
    connections: Vec<Connection>,
}

impl Server {
    pub fn new(listener: TcpListener) -> Self {
        Server {
            listener: TcpListenerStream::new(listener),
            connections: vec![],
        }
    }

    pub async fn run(&mut self) {
        loop {
            poll(&mut self.listener, &mut self.connections).await;
        }
    }
}
