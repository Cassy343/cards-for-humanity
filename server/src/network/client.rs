use futures::{
    channel::mpsc::{self, SendError, UnboundedReceiver, UnboundedSender},
    SinkExt,
    StreamExt,
};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use warp::ws::{Message, WebSocket};
use log::{debug, error};

pub struct ClientHandler {
    client_list: HashMap<usize, Client>,
    client_id: usize,
    message_pipe: UnboundedSender<ClientMessage>,
}

impl ClientHandler {
    pub fn new() -> (Self, UnboundedReceiver<ClientMessage>) {
        let (message_pipe, message_aggregator) = mpsc::unbounded();
        (
            ClientHandler {
                client_list: HashMap::new(),
                client_id: 0,
                message_pipe,
            },
            message_aggregator,
        )
    }

    pub fn message_pipe(&self) -> UnboundedSender<ClientMessage> {
        self.message_pipe.clone()
    }

    pub fn add_client(
        &mut self,
        conn: UnboundedSender<Message>,
        address: Option<SocketAddr>,
    ) -> usize {
        let id = self.client_id;
        self.client_list.insert(id, Client::new(conn, address));
        self.client_id += 1;
        id
    }

    pub fn remove_client(&mut self, id: usize) -> Option<Client> {
        self.client_list.remove(&id)
    }

    pub async fn broadcast(&mut self, message: Message) {
        for client in self.client_list.values_mut() {
            // TODO: better error handling?
            let _ = client.send(message.clone()).await;
        }
    }

    pub async fn handle_socket(
        socket: WebSocket,
        address: Option<SocketAddr>,
        client_handler: Arc<Mutex<ClientHandler>>,
    ) {
        let (ws_tx, mut ws_rx) = socket.split();
        let (tx, rx) = mpsc::unbounded::<Message>();

        tokio::task::spawn(rx.map(|message| Ok(message)).forward(ws_tx));

        let mut handler_guard = client_handler.lock().await;
        let id = handler_guard.add_client(tx.clone(), address);
        let mut pipe = handler_guard.message_pipe();
        drop(handler_guard);

        debug!("New client connected (ID {})", id);

        while let Some(result) = ws_rx.next().await {
            let message = match result {
                Ok(message) => message,
                Err(e) => {
                    error!("WS error {}", e);
                    break;
                }
            };

            if let Err(e) = pipe.send(ClientMessage::new(message, id)).await {
                error!("Failed to pipe WS message to handler, {}", e);
            }
        }

        client_handler.lock().await.remove_client(id);
        debug!("Client disconnected (ID {})", id);
    }
}

pub struct Client {
    connection: UnboundedSender<Message>,
    address: Option<SocketAddr>,
}

impl Client {
    pub fn new(connection: UnboundedSender<Message>, address: Option<SocketAddr>) -> Self {
        Client {
            connection,
            address,
        }
    }

    pub async fn send(&mut self, message: Message) -> Result<(), SendError> {
        self.connection.send(message).await
    }
}

pub struct ClientMessage {
    pub message: Message,
    pub client_id: usize,
}

impl ClientMessage {
    pub fn new(message: Message, client_id: usize) -> Self {
        ClientMessage { message, client_id }
    }
}
