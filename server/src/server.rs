use std::{sync::Arc, usize};
use futures::{SinkExt, channel::mpsc::{self, SendError, UnboundedReceiver, UnboundedSender}};
use warp::ws::Message;
use std::net::SocketAddr;
use tokio::sync::{Mutex};
use std::collections::HashMap;

pub type WsServerHandler = Arc<Mutex<WsServerHandlerInner>>;

pub struct WsServerHandlerInner {
    client_list: HashMap<usize, Client>,
    client_id: usize,
    message_pipe: UnboundedSender<(usize, Message)>,
    message_aggregator: UnboundedReceiver<(usize, Message)>
}

impl WsServerHandlerInner {
    pub fn new() -> Self {
        let (message_pipe, message_aggregator) = mpsc::unbounded();
        WsServerHandlerInner {
            client_list: HashMap::new(),
            client_id: 0,
            message_pipe,
            message_aggregator
        }
    }

    pub fn message_pipe(&self) -> UnboundedSender<(usize, Message)> {
        self.message_pipe.clone()
    }

    pub fn add_client(&mut self, conn: UnboundedSender<Message>, address: Option<SocketAddr>) -> usize {
        let id = self.client_id;
        self.client_list.insert(id, Client::new(conn, address));
        self.client_id += 1;
        id
    }

    pub fn remove_client(&mut self, id: usize) -> Option<Client> {
        self.client_list.remove(&id)
    }

    // TODO: remove this function
    pub async fn broadcast(&mut self, message: Message) {
        for client in self.client_list.values_mut() {
            let _ = client.send(message.clone()).await;
        }
    }
}

pub struct Client {
    connection: UnboundedSender<Message>,
    address: Option<SocketAddr>
}

impl Client {
    pub fn new(connection: UnboundedSender<Message>, address: Option<SocketAddr>) -> Self {
        Client {
            connection,
            address
        }
    }

    pub async fn send(&mut self, message: Message) -> Result<(), SendError> {
        self.connection.send(message).await
    }
}