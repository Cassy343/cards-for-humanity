use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLock as StdRwLock},
};

use crate::network::{Listener, NetworkHandler};

use async_trait::async_trait;
use common::protocol::{
    clientbound::{ClientBoundPacket, PacketResponse},
    serverbound::ServerBoundPacket,
};
use futures::future::join_all;
use log::warn;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{packs::PackStore, Game};

pub struct Lobby {
    pack_store: Arc<StdRwLock<PackStore>>,
    games: Vec<Rc<RwLock<Game>>>,
}

impl Lobby {
    pub fn new(pack_store: Arc<StdRwLock<PackStore>>) -> Self {
        Lobby {
            pack_store,
            games: Vec::new(),
        }
    }
}

#[async_trait(?Send)]
impl Listener for Lobby {
    async fn client_connected(&mut self, network_handler: &mut NetworkHandler, client_id: Uuid) {
        // Drop any Games that have terminated
        self.games.retain(|g| {
            // Same note as in handler.rs
            // now_or_never could return None if the RwLock is being used but it doesn't matter
            !futures::FutureExt::now_or_never(async { g.read().await.is_terminated() })
                .unwrap_or(false)
        });

        let games_future = self.games.iter().map(|g| g.read()).collect::<Vec<_>>();
        let games = join_all(games_future).await;

        let mut client_handler = network_handler.client_handler.lock().await;

        match client_handler
            .send_packet(client_id, &ClientBoundPacket::ServerList {
                servers: games
                    .iter()
                    .map(|g| (g.id, g.host_name(), g.num_players(), g.max_players))
                    .collect(),
            })
            .await
        {
            Some(Err(e)) => warn!("Error sending server list to client: {}", e),
            _ => {}
        };

        match client_handler
            .send_packet(
                client_id,
                &ClientBoundPacket::CardPacks(self.pack_store.read().unwrap().get_packs_meta()),
            )
            .await
        {
            Some(Err(e)) => warn!("Error sending card packs to client: {}", e),
            _ => {}
        }
    }

    async fn client_disconnected(
        &mut self,
        _network_handler: &mut NetworkHandler,
        _client_id: Uuid,
    ) {
    }

    async fn handle_packet(
        &mut self,
        network_handler: &mut NetworkHandler,
        packet: &ServerBoundPacket,
        sender_id: Uuid,
    ) -> PacketResponse {
        match packet {
            ServerBoundPacket::CreateServer(settings) => {
                if settings.packs.len() == 0 {
                    return PacketResponse::RejectedWithReason("Packs cannot be empty".to_owned());
                }

                if settings.points_to_win == 0 {
                    return PacketResponse::RejectedWithReason(
                        "Points to win has to be at least 1".to_owned(),
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
                    // Use a fake Uuid to create the game because we can't know what uuid is assigned to it
                    Uuid::from_u128(0),
                    sender_id.clone(),
                    self.pack_store.clone(),
                    settings.clone(),
                ) {
                    Ok(g) => Rc::new(RwLock::new(g)),
                    Err(e) => {
                        warn!("Error making new game {}", e);
                        return PacketResponse::RejectedWithReason(
                            "Error creating new game".to_owned(),
                        );
                    }
                };

                let listener_id = network_handler.add_listener(new_game.clone());
                new_game.write().await.id = listener_id;
                self.games.push(new_game);

                match network_handler.forward_client(sender_id, listener_id).await {
                    Some(_) => PacketResponse::Accepted,
                    None => PacketResponse::Rejected,
                }
            }

            ServerBoundPacket::JoinGame(server_id) => {
                if !network_handler.valid_listener(*server_id) {
                    return PacketResponse::RejectedWithReason("Invalid server id".to_owned());
                };

                match network_handler.forward_client(sender_id, *server_id).await {
                    Some(_) => PacketResponse::Accepted,
                    None => PacketResponse::Rejected,
                }
            }

            ServerBoundPacket::RequestCardPacks => {
                let mut client_handler = network_handler.client_handler.lock().await;
                client_handler
                    .send_packet(
                        sender_id,
                        &ClientBoundPacket::CardPacks(
                            self.pack_store.read().unwrap().get_packs_meta(),
                        ),
                    )
                    .await;
                PacketResponse::Accepted
            }

            _ => PacketResponse::RejectedWithReason("Unexpected packet type in lobby".to_owned()),
        }
    }

    fn is_terminated(&self) -> bool {
        false
    }
}
