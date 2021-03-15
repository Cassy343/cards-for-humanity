use crate::network::{Listener, NetworkHandler};
use common::protocol::serverbound::ServerBoundPacket;

pub struct Game;

impl Listener for Game {
    fn handle_packet(&mut self, network_handler: &mut NetworkHandler, packet: &ServerBoundPacket, sender_id: usize) {
        log::debug!("Received message from client {}: {:?}", sender_id, packet);
    }
}