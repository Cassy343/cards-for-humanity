use crate::network::client::{ClientEvent, ClientEventData, ClientHandler};
use async_trait::async_trait;
use common::protocol::{
    clientbound::{ClientBoundPacket, PacketResponse},
    decode,
    serverbound::{ServerBoundPacket, WrappedServerBoundPacket},
};
use futures::channel::{mpsc::UnboundedReceiver, oneshot::Sender};
use log::{debug, error, warn};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};
use tokio::sync::{Mutex, RwLock};
use warp::ws::Message;

pub struct NetworkHandler {
    pub client_handler: Arc<Mutex<ClientHandler>>,
    incoming_messages: UnboundedReceiver<ClientEvent>,
    listeners: HashMap<usize, Rc<RefCell<Box<dyn Listener>>>>,
    server_shutdown_hook: Option<Sender<()>>,
    listener_id: usize,
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
            listeners: HashMap::new(),
            server_shutdown_hook: Some(server_shutdown_hook),
            listener_id: 0,
        }
    }

    pub async fn handle_messages(&mut self) {
        let mut acknowledgements = Vec::new();

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

                    let packets: Vec<WrappedServerBoundPacket> = match decode(text) {
                        Ok(packets) => packets,
                        Err(_) => {
                            warn!(
                                "Received invalid packet from client {}: {:?}",
                                client_id, message
                            );
                            continue;
                        }
                    };

                    let mut client_handler = self.client_handler.lock().await;

                    let client = match client_handler.get_client_mut(client_id) {
                        Some(client) => client,
                        None => {
                            warn!(
                                "Received message from unknown client {}: {:?}",
                                client_id, message
                            );
                            continue;
                        }
                    };

                    let listener_id = client.listener;

                    drop(client);
                    drop(client_handler);

                    let listener = match self.listeners.get(&listener_id) {
                        Some(l) => l.clone(),
                        None => {
                            warn!(
                                "Client ({}) has unregistered listener id {}",
                                client_id, listener_id
                            );
                            continue;
                        }
                    };

                    for packet in packets.iter() {
                        let response = listener
                            .borrow_mut()
                            .handle_packet(self, packet.packet(), client_id)
                            .await;
                        debug!("response {:?} to {:?}", response, packet);
                        if let Some(id) = packet.packet_id() {
                            acknowledgements.push((id, response));
                        }
                    }

                    self.client_handler
                        .lock()
                        .await
                        .send_packets(
                            client_id,
                            &acknowledgements
                                .drain(..)
                                .map(|(id, response)| ClientBoundPacket::Ack {
                                    packet_id: id,
                                    response,
                                })
                                .collect::<Vec<_>>(),
                        )
                        .await;
                }
                Some(ClientEvent { data, client_id }) => match data {
                    ClientEventData::Connect => {
                        let client_handler = self.client_handler.lock().await;

                        let client = match client_handler.get_client(client_id) {
                            Some(client) => client,
                            None => {
                                warn!("Received connection from unknown client {}", client_id);
                                continue;
                            }
                        };

                        let listener_id = client.listener;

                        let listener = match self.listeners.get(&listener_id) {
                            Some(l) => l,
                            None => {
                                warn!(
                                    "Client ({}) has unregistered listener id {}",
                                    client_id, listener_id
                                );
                                continue;
                            }
                        };

                        drop(client);
                        drop(client_handler);

                        listener
                            .clone()
                            .borrow_mut()
                            .client_connected(self, client_id)
                            .await;
                    }

                    ClientEventData::Disconnect => {
                        let client_handler = self.client_handler.lock().await;

                        let client = match client_handler.get_client(client_id) {
                            Some(client) => client,
                            None => {
                                warn!("Received connection from unknown client {}", client_id);
                                continue;
                            }
                        };

                        let listener_id = client.listener;

                        let listener = match self.listeners.get(&listener_id) {
                            Some(l) => l,
                            None => {
                                warn!(
                                    "Client ({}) has unregistered listener id {}",
                                    client_id, listener_id
                                );
                                continue;
                            }
                        };

                        drop(client);
                        drop(client_handler);

                        listener
                            .clone()
                            .borrow_mut()
                            .client_disconnected(self, client_id)
                            .await;
                    }

                    _ => unreachable!(),
                },
                None => {
                    error!("Incoming message channel unexpectedly closed.");
                    return;
                }
            }
        }

        self.listeners
            .retain(|_, listener| !listener.borrow().is_terminated());
    }

    pub fn add_listener<L: Listener + 'static>(&mut self, listener: L) -> usize {
        let new_id = self.listener_id;
        debug!("new_id {}", new_id);
        self.listeners
            .insert(new_id, Rc::new(RefCell::new(Box::new(listener))));
        self.listener_id += 1;
        debug!("new_id {}", new_id);
        new_id
    }

    pub fn next_id(&self) -> usize {
        self.listener_id
    }

    pub fn valid_listener(&self, id: usize) -> bool {
        self.listeners.contains_key(&id)
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

    pub async fn forward_client(&mut self, client_id: usize, listener_id: usize) -> Option<()> {
        self.client_handler
            .lock()
            .await
            .get_client_mut(client_id)?
            .listener = listener_id;
        self.listeners
            .get(&listener_id)?
            .clone()
            .borrow_mut()
            .client_connected(self, client_id)
            .await;
        Some(())
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
    ) -> PacketResponse;

    fn is_terminated(&self) -> bool {
        false
    }
}

#[async_trait(?Send)]
impl<T: Listener> Listener for Rc<RwLock<T>> {
    async fn client_connected(&mut self, network_handler: &mut NetworkHandler, client_id: usize) {
        self.write()
            .await
            .client_connected(network_handler, client_id)
            .await
    }

    async fn client_disconnected(
        &mut self,
        network_handler: &mut NetworkHandler,
        client_id: usize,
    ) {
        self.write()
            .await
            .client_disconnected(network_handler, client_id)
            .await
    }

    async fn handle_packet(
        &mut self,
        network_handler: &mut NetworkHandler,
        packet: &ServerBoundPacket,
        sender_id: usize,
    ) -> PacketResponse {
        self.write()
            .await
            .handle_packet(network_handler, packet, sender_id)
            .await
    }
}
