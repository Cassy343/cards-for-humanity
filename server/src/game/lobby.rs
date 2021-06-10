use std::{cell::RefCell, rc::Rc};

use crate::network::{Listener, NetworkHandler};

use async_trait::async_trait;
use common::protocol::{
    clientbound::{ClientBoundPacket, PacketResponse},
    serverbound::ServerBoundPacket,
};
use futures::future::join_all;
use log::warn;
use tokio::sync::RwLock;

use super::{packs::PackStore, Game};

pub struct Lobby {
    pack_store: Rc<RefCell<PackStore>>,
    games: Vec<Rc<RwLock<Game>>>,
}

impl Lobby {
    pub fn new(pack_store: Rc<RefCell<PackStore>>) -> Self {
        Lobby {
            pack_store,
            games: Vec::new(),
        }
    }
}

#[async_trait(?Send)]
impl Listener for Lobby {
    async fn client_connected(&mut self, network_handler: &mut NetworkHandler, client_id: usize) {
        let games_future = self.games.iter().map(|g| g.read()).collect::<Vec<_>>();
        let games = join_all(games_future).await;

        let mut client_handler = network_handler.client_handler.lock().await;

        match client_handler.send_packet(client_id, &ClientBoundPacket::ServerList {
                servers: games
                    .iter()
                    .map(|g| (g.id, g.num_players(), g.max_players))
                    .collect(),
            })
            .await
        {
            Some(Err(e)) => warn!("Error sending server list to client: {}", e),
            _ => {}
        };

        match client_handler.send_packet(client_id, &ClientBoundPacket::CardPacks(self.pack_store.borrow().get_pack_names())).await {
            Some(Err(e)) => warn!("Error sending card packs to client: {}", e),
            _ => {}
        }
    }

    async fn client_disconnected(
        &mut self,
        _network_handler: &mut NetworkHandler,
        _client_id: usize,
    ) {
    }

    async fn handle_packet(
        &mut self,
        network_handler: &mut NetworkHandler,
        packet: &ServerBoundPacket,
        sender_id: usize,
    ) -> PacketResponse {
        match packet {
            ServerBoundPacket::CreateServer(settings) => {
                if settings.packs.len() == 0 {
                    return PacketResponse::RejectedWithReason("Packs cannot be emppty".to_owned());
                }

                if settings.points_to_win == 0 {
                    return PacketResponse::RejectedWithReason(
                        "Point to win has to be at least 1".to_owned(),
                    );
                }

                match settings.max_players {
                    Some(p) =>
                        if p < 2 {
                            return PacketResponse::RejectedWithReason(
                                "Max players needs to be at least 2".to_owned(),
                            );
                        },
                    None => {}
                }

                let new_game = match Game::new(
                    network_handler.next_id(),
                    self.pack_store.clone(),
                    settings.clone(),
                ) {
                    Ok(g) => {
                        Rc::new(RwLock::new(g))
                    },
                    Err(e) => {
                        warn!("Error making new game {}", e);
                        return PacketResponse::RejectedWithReason("Error creating new game".to_owned())
                    }
                };

                let listener_id = network_handler.add_listener(new_game.clone());
                self.games.push(new_game);

                match network_handler.forward_client(sender_id, listener_id).await {
                    Some(_) => PacketResponse::Accepted,
                    None => PacketResponse::Rejected
                }
            }

            ServerBoundPacket::JoinGame(server_id) => {
                if !network_handler.valid_listener(*server_id) {
                    return PacketResponse::RejectedWithReason("Invalid server id".to_owned());
                };

                match network_handler.forward_client(sender_id, *server_id).await {
                    Some(_) => PacketResponse::Accepted,
                    None => PacketResponse::Rejected
                }
            }

            ServerBoundPacket::RequestCardPacks => {
                let mut client_handler = network_handler.client_handler.lock().await;
                client_handler
                    .send_packet(
                        sender_id,
                        &ClientBoundPacket::CardPacks(self.pack_store.borrow().get_pack_names()),
                    )
                    .await;
                PacketResponse::Accepted
            }

            _ => PacketResponse::Rejected,
        }
    }

    fn is_terminated(&self) -> bool {
        false
    }
}
