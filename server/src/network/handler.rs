use std::sync::Arc;
use tokio::sync::Mutex;
use warp::ws::Message;
use crate::network::client::{ClientHandler, ClientMessage};
use futures::channel::{mpsc::UnboundedReceiver, oneshot::Sender};
use std::cell::RefCell;
use std::rc::Rc;
use log::{error, warn};
use common::protocol::{decode, serverbound::ServerBoundPacket};

pub struct NetworkHandler {
    client_handler: Arc<Mutex<ClientHandler>>,
    incoming_messages: UnboundedReceiver<ClientMessage>,
    listeners: Vec<Rc<RefCell<Box<dyn Listener>>>>,
    server_shutdown_hook: Option<Sender<()>>
}

impl NetworkHandler {
    pub fn new(
        client_handler: Arc<Mutex<ClientHandler>>,
        incoming_messages: UnboundedReceiver<ClientMessage>,
        server_shutdown_hook: Sender<()>
    ) -> Self
    {
        NetworkHandler {
            client_handler,
            incoming_messages,
            listeners: Vec::new(),
            server_shutdown_hook: Some(server_shutdown_hook)
        }
    }

    pub fn handle_messages(&mut self) {
        while let Ok(message) = self.incoming_messages.try_next() {
            match message {
                Some(ClientMessage { message, client_id }) if message.is_text() => {
                    let text = match message.to_str() {
                        Ok(text) => text,
                        Err(_) => {
                            warn!("Received invalid packet from client {}: {:?}", client_id, message);
                            continue;
                        }
                    };

                    let packet: ServerBoundPacket = match decode(text) {
                        Ok(packet) => packet,
                        Err(_) => {
                            warn!("Received invalid packet from client {}: {:?}", client_id, message);
                            continue;
                        }
                    };

                    for i in 0..self.listeners.len() {
                        self.listeners[i].clone().borrow_mut().handle_packet(self, &packet, client_id);
                    }
                },
                None => {
                    error!("Incoming message channel unexpectedly closed.");
                    return;
                },
                _ => {}
            }
        }

        self.listeners.retain(|listener| !listener.borrow().is_terminated());
    }

    pub fn add_listener<L: Listener + 'static>(&mut self, listener: L) {
        self.listeners.push(Rc::new(RefCell::new(Box::new(listener))));
    }

    pub async fn shutdown(&mut self) {
        match self.server_shutdown_hook.take() {
            Some(hook) => {
                // If it fails it doesn't matter since we're shutting down anyway
                let _ = hook.send(());
            },

            // We've already shutdown
            None => return
        }

        self.client_handler.lock().await.broadcast_all(Message::close()).await;
        self.listeners.clear();
    }
}

pub trait Listener {
    fn handle_packet(&mut self, network_handler: &mut NetworkHandler, packet: &ServerBoundPacket, sender_id: usize);

    fn is_terminated(&self) -> bool {
        false
    }
}