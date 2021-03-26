use crate::network::client::{ClientEvent, ClientEventData, ClientHandler};
use async_trait::async_trait;
use common::protocol::{decode, serverbound::ServerBoundPacket};
use futures::channel::{mpsc::UnboundedReceiver, oneshot::Sender};
use log::{error, warn};
use std::{cell::RefCell, rc::Rc, sync::Arc};
use tokio::sync::Mutex;
use warp::ws::Message;

pub struct NetworkHandler {
    pub client_handler: Arc<Mutex<ClientHandler>>,
    incoming_messages: UnboundedReceiver<ClientEvent>,
    listeners: Vec<Rc<RefCell<Box<dyn Listener>>>>,
    server_shutdown_hook: Option<Sender<()>>,
}

impl NetworkHandler {
    pub fn new(
        client_handler: Arc<Mutex<ClientHandler>>,
        incoming_messages: UnboundedReceiver<ClientEvent>,
        server_shutdown_hook: Sender<()>,
    ) -> Self {
        NetworkHandler {
            client_handler,
            incoming_messages,
            listeners: Vec::new(),
            server_shutdown_hook: Some(server_shutdown_hook),
        }
    }

    pub async fn handle_messages(&mut self) {
        while let Ok(message) = self.incoming_messages.try_next() {
            match message {
                Some(ClientEvent {
                    data: ClientEventData::Message(message),
                    client_id,
                }) => {
                    if !message.is_text() {
                        return;
                    }

                    let text = match message.to_str() {
                        Ok(text) => text,
                        Err(_) => {
                            warn!(
                                "Received invalid packet from client {}: {:?}",
                                client_id, message
                            );
                            continue;
                        }
                    };

                    let packets: Vec<ServerBoundPacket> = match decode(text) {
                        Ok(packets) => packets,
                        Err(_) => {
                            warn!(
                                "Received invalid packet from client {}: {:?}",
                                client_id, message
                            );
                            continue;
                        }
                    };

                    for i in 0 .. self.listeners.len() {
                        for packet in packets.iter() {
                            self.listeners[i]
                                .clone()
                                .borrow_mut()
                                .handle_packet(self, packet, client_id)
                                .await;
                        }
                    }
                }
                Some(ClientEvent { data, client_id }) => match data {
                    ClientEventData::Connect =>
                        for i in 0 .. self.listeners.len() {
                            self.listeners[i]
                                .clone()
                                .borrow_mut()
                                .client_connected(self, client_id)
                                .await;
                        },

                    ClientEventData::Disconnect =>
                        for i in 0 .. self.listeners.len() {
                            self.listeners[i]
                                .clone()
                                .borrow_mut()
                                .client_disconnected(self, client_id)
                                .await;
                        },

                    _ => unreachable!(),
                },
                None => {
                    error!("Incoming message channel unexpectedly closed.");
                    return;
                }
            }
        }

        self.listeners
            .retain(|listener| !listener.borrow().is_terminated());
    }

    pub fn add_listener<L: Listener + 'static>(&mut self, listener: L) {
        self.listeners
            .push(Rc::new(RefCell::new(Box::new(listener))));
    }

    pub async fn shutdown(&mut self) {
        match self.server_shutdown_hook.take() {
            Some(hook) => {
                // If it fails it doesn't matter since we're shutting down anyway
                let _ = hook.send(());
            }

            // We've already shutdown
            None => return,
        }

        self.client_handler
            .lock()
            .await
            .broadcast_all(Message::close())
            .await;
        self.listeners.clear();
    }
}

#[async_trait(?Send)]
pub trait Listener {
    async fn client_connected(&mut self, network_handler: &mut NetworkHandler, client_id: usize);

    async fn client_disconnected(&mut self, network_handler: &mut NetworkHandler, client_id: usize);

    async fn handle_packet(
        &mut self,
        network_handler: &mut NetworkHandler,
        packet: &ServerBoundPacket,
        sender_id: usize,
    );

    fn is_terminated(&self) -> bool {
        false
    }
}
