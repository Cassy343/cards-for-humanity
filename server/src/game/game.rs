use crate::network::{client::ClientHandler, Listener, NetworkHandler};
use async_trait::async_trait;
use common::{
    data::cards::{CardID, Pack, Prompt, Response},
    protocol::{clientbound::ClientBoundPacket, serverbound::ServerBoundPacket, GameSetting},
};
use rand::{thread_rng, Rng};
use std::{collections::HashMap, rc::Rc};
use tokio::sync::MutexGuard;

pub struct Game {
    players: HashMap<usize, Player>,
    host_id: usize,
    packs: Vec<Rc<Pack>>,
    available_prompts: Vec<CardID>,
    available_responses: Vec<CardID>,
    state: GameState,
    max_players: Option<usize>,
    max_selection_time: Option<u32>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            players: HashMap::new(),
            host_id: 0,
            packs: Vec::new(),
            available_prompts: Vec::new(),
            available_responses: Vec::new(),
            state: GameState::WaitingToStart,
            max_players: None,
            max_selection_time: None,
        }
    }

    fn initialize_prompts(&mut self) {
        for (index, pack) in self.packs.iter().enumerate() {
            self.available_prompts
                .extend((0 .. pack.prompts.len()).map(|j| CardID::new(index, j)));
        }
    }

    fn initialize_responses(&mut self) {
        for (index, pack) in self.packs.iter().enumerate() {
            self.available_responses
                .extend((0 .. pack.responses.len()).map(|j| CardID::new(index, j)));
        }
    }

    fn select_prompt(&mut self) -> Prompt {
        if self.available_prompts.is_empty() {
            self.initialize_prompts();
        }

        let card =
            self.available_prompts[thread_rng().gen_range(0 .. self.available_prompts.len())];
        self.packs[card.pack_number].prompts[card.card_number].clone()
    }

    fn add_responses(&mut self, dest: &mut Vec<Response>, count: u32) {
        for _ in 0 .. count {
            if self.available_responses.is_empty() {
                self.initialize_responses();
            }

            let card = self.available_responses
                [thread_rng().gen_range(0 .. self.available_responses.len())];
            dest.push(self.packs[card.pack_number].responses[card.card_number].clone());
        }
    }

    async fn broadcast_to_players<'a>(
        &mut self,
        client_handler: &mut MutexGuard<'a, ClientHandler>,
        packet: &ClientBoundPacket,
    ) {
        client_handler
            .broadcast(
                &packet,
                |client| self.players.contains_key(&client.id),
                |_| {},
            )
            .await;
    }

    fn settings_as_packets(&self) -> Vec<ClientBoundPacket> {
        let mut packets = Vec::with_capacity(2 + self.packs.len());
        packets.push(ClientBoundPacket::SettingUpdate(GameSetting::MaxPlayers(
            self.max_players,
        )));
        packets.push(ClientBoundPacket::SettingUpdate(
            GameSetting::MaxSelectionTime(self.max_selection_time),
        ));
        for pack in self.packs.iter() {
            packets.push(ClientBoundPacket::SettingUpdate(GameSetting::AddPack(
                pack.name.clone(),
            )));
        }
        packets
    }
}

#[async_trait(?Send)]
impl Listener for Game {
    async fn client_connected(&mut self, network_handler: &mut NetworkHandler, client_id: usize) {
        let set_host = self.players.is_empty();
        if set_host {
            self.host_id = client_id;
        }
        let player = Player::new(client_id, set_host);
        let packet = player.as_packet();

        let mut client_handler = network_handler.client_handler.lock().await;
        self.broadcast_to_players(&mut client_handler, &packet)
            .await;

        self.players.insert(client_id, player);

        let mut packets = self
            .players
            .values()
            .map(Player::as_packet)
            .collect::<Vec<_>>();
        packets.extend(self.settings_as_packets());
        client_handler.send_packets(client_id, &packets).await;
    }

    async fn client_disconnected(
        &mut self,
        network_handler: &mut NetworkHandler,
        client_id: usize,
    ) {
        let player = match self.players.remove(&client_id) {
            Some(player) => player,
            None => return,
        };

        let new_host = if player.is_host {
            self.players.keys().copied().next()
        } else {
            None
        };

        if let Some(host_id) = new_host {
            self.host_id = host_id;
        }

        let packet = ClientBoundPacket::RemovePlayer {
            id: player.client_id,
            new_host,
        };

        self.broadcast_to_players(&mut network_handler.client_handler.lock().await, &packet)
            .await;
    }

    async fn handle_packet(
        &mut self,
        network_handler: &mut NetworkHandler,
        packet: &ServerBoundPacket,
        sender_id: usize,
    ) {
        match self.state {
            GameState::WaitingToStart => {
                match packet {
                    ServerBoundPacket::StartGame => {
                        if self.host_id != sender_id {
                            return;
                        }

                        // TODO: Initialize round

                        self.state = GameState::Playing(PlayingState::PlayerSelection);
                    }

                    ServerBoundPacket::UpdateSetting(setting) => {
                        if self.host_id != sender_id {
                            return;
                        }

                        match setting {
                            GameSetting::MaxPlayers(limit) => self.max_players = limit.clone(),
                            GameSetting::MaxSelectionTime(limit) =>
                                self.max_selection_time = limit.clone(),
                            GameSetting::AddPack(pack) => {}
                            GameSetting::RemovePack(pack) => {}
                        }

                        self.broadcast_to_players(
                            &mut network_handler.client_handler.lock().await,
                            &ClientBoundPacket::echo_setting_update(setting),
                        )
                        .await;
                    }

                    ServerBoundPacket::SetPlayerName(name) => {
                        if let Some(player) = self.players.get_mut(&sender_id) {
                            player.name = name.clone();
                            self.broadcast_to_players(
                                &mut network_handler.client_handler.lock().await,
                                &ClientBoundPacket::UpdatePlayerName {
                                    id: sender_id,
                                    name: name.clone(),
                                },
                            )
                            .await;
                        }
                    }

                    _ => {}
                }
            }
            GameState::Playing(PlayingState::PlayerSelection) => {}
            GameState::Playing(PlayingState::CzarSelection) => {}
            GameState::End => {}
        }
    }
}

struct Player {
    client_id: usize,
    name: String,
    is_host: bool,
    points: u32,
}

impl Player {
    pub fn new(client_id: usize, is_host: bool) -> Self {
        Player {
            client_id,
            name: format!("Player #{}", client_id),
            is_host,
            points: 0,
        }
    }

    pub fn as_packet(&self) -> ClientBoundPacket {
        ClientBoundPacket::AddPlayer {
            id: self.client_id,
            name: self.name.clone(),
            is_host: self.is_host,
            points: self.points,
        }
    }
}

#[derive(Clone, Copy)]
enum GameState {
    WaitingToStart,
    Playing(PlayingState),
    End,
}

#[derive(Clone, Copy)]
enum PlayingState {
    PlayerSelection,
    CzarSelection,
}
