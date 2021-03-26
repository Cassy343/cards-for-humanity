use common::protocol::encode;
use futures::{
    channel::mpsc::{self, SendError, UnboundedReceiver, UnboundedSender},
    SinkExt,
    StreamExt,
};
use log::{debug, error};
use serde::Serialize;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use warp::ws::{Message, WebSocket};

pub struct ClientHandler {
    client_list: HashMap<usize, Client>,
    client_id: usize,
    message_pipe: UnboundedSender<ClientEvent>,
}

impl ClientHandler {
    pub fn new() -> (Self, UnboundedReceiver<ClientEvent>) {
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

    pub fn message_pipe(&self) -> UnboundedSender<ClientEvent> {
        self.message_pipe.clone()
    }

    pub fn add_client(
        &mut self,
        conn: UnboundedSender<Message>,
        address: Option<SocketAddr>,
    ) -> usize {
        let id = self.client_id;
        self.client_list.insert(id, Client::new(id, conn, address));
        self.client_id += 1;
        id
    }

    pub fn get_client(&self, client_id: usize) -> Option<&Client> {
        self.client_list.get(&client_id)
    }

    pub fn get_client_mut(&mut self, client_id: usize) -> Option<&mut Client> {
        self.client_list.get_mut(&client_id)
    }

    pub fn remove_client(&mut self, id: usize) -> Option<Client> {
        self.client_list.remove(&id)
    }

    pub async fn send_packet<P: Serialize>(
        &mut self,
        client_id: usize,
        packet: &P,
    ) -> Option<Result<(), SendError>> {
        self.send_packets(client_id, &[packet]).await
    }

    pub async fn send_packets<'a, T, P>(
        &mut self,
        client_id: usize,
        packets: T,
    ) -> Option<Result<(), SendError>>
    where
        T: IntoIterator<Item = &'a P>,
        P: Serialize + 'a,
    {
        Some(
            self.get_client_mut(client_id)?
                .send(Message::text(encode(
                    &(packets.into_iter().collect::<Vec<_>>()),
                )))
                .await,
        )
    }

    pub async fn broadcast<P, F, E>(&mut self, packet: &P, mut filter: F, mut on_error: E)
    where
        P: Serialize,
        F: FnMut(&Client) -> bool,
        E: FnMut(&Client),
    {
        let encoded = encode(&[packet]);
        for client in self
            .client_list
            .values_mut()
            .filter(|client| filter(client))
        {
            match client.send(Message::text(encoded.clone())).await {
                Err(_) => on_error(client),
                _ => {}
            }
        }
    }

    pub async fn broadcast_all(&mut self, message: Message) {
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

        if let Err(e) = pipe.send(ClientEvent::connect(id)).await {
            error!("Failed to send client connect event, {}", e);
            handler_guard.remove_client(id);
        }

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

            if let Err(e) = pipe.send(ClientEvent::message(message, id)).await {
                error!("Failed to pipe WS message to handler, {}", e);
            }
        }

        let _ = pipe.send(ClientEvent::disconnect(id)).await;
        client_handler.lock().await.remove_client(id);
        debug!("Client disconnected (ID {})", id);
    }
}

pub struct Client {
    pub id: usize,
    connection: UnboundedSender<Message>,
    address: Option<SocketAddr>,
}

impl Client {
    pub fn new(
        id: usize,
        connection: UnboundedSender<Message>,
        address: Option<SocketAddr>,
    ) -> Self {
        Client {
            id,
            connection,
            address,
        }
    }

    pub async fn send(&mut self, message: Message) -> Result<(), SendError> {
        self.connection.send(message).await
    }
}

pub struct ClientEvent {
    pub data: ClientEventData,
    pub client_id: usize,
}

impl ClientEvent {
    pub fn connect(client_id: usize) -> Self {
        ClientEvent {
            data: ClientEventData::Connect,
            client_id,
        }
    }

    pub fn message(message: Message, client_id: usize) -> Self {
        ClientEvent {
            data: ClientEventData::Message(message),
            client_id,
        }
    }

    pub fn disconnect(client_id: usize) -> Self {
        ClientEvent {
            data: ClientEventData::Disconnect,
            client_id,
        }
    }
}

pub enum ClientEventData {
    Connect,
    Message(Message),
    Disconnect,
}
