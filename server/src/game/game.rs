use super::packs::PackStore;
use crate::network::{client::ClientHandler, Listener, NetworkHandler};
use async_trait::async_trait;
use common::{
    data::{
        cards::{CardID, Pack, Prompt, Response},
        VecMap,
    },
    protocol::{
        clientbound::{ClientBoundPacket, PacketResponse, ResponseData},
        serverbound::ServerBoundPacket,
        GameSetting,
        GameSettings,
    },
};
use log::error;
use rand::{thread_rng, Rng};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use tokio::sync::MutexGuard;

pub struct Game {
    pub id: usize,
    pack_store: Rc<RefCell<PackStore>>,
    players: VecMap<usize, Player>,
    host_id: usize,
    packs: Vec<Rc<Pack>>,
    available_prompts: Vec<CardID>,
    available_responses: Vec<CardID>,
    state: GameState,
    pub max_players: Option<usize>,
    max_selection_time: Option<u32>,
    points_to_win: u32,
    czar_index: usize,
    current_prompt: Option<Prompt>,
}

impl Game {
    pub fn new(
        id: usize,
        pack_store: Rc<RefCell<PackStore>>,
        settings: GameSettings,
    ) -> Result<Self, String> {
        let mut loaded_packs = Vec::new();

        for pack_name in settings.packs {
            loaded_packs.push(pack_store.borrow_mut().load_pack(&pack_name)?)
        }

        Ok(Game {
            id,
            pack_store,
            players: VecMap::new(),
            host_id: 0,
            packs: loaded_packs,
            available_prompts: Vec::new(),
            available_responses: Vec::new(),
            state: GameState::WaitingToStart,
            max_players: settings.max_players,
            max_selection_time: settings.max_selection_time,
            points_to_win: settings.points_to_win,
            czar_index: 0,
            current_prompt: None,
        })
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

    fn add_responses(&mut self, dest: &mut Vec<ResponseData>, count: usize) {
        for _ in 0 .. count {
            if self.available_responses.is_empty() {
                self.initialize_responses();
            }

            let card = self.available_responses
                [thread_rng().gen_range(0 .. self.available_responses.len())];
            dest.push(ResponseData::new(
                card,
                self.packs[card.pack_number].responses[card.card_number].clone(),
            ));
        }
    }

    fn response_text(&self, card: CardID) -> Response {
        self.packs[card.pack_number].responses[card.card_number].clone()
    }

    fn display_responses(&self) -> ClientBoundPacket {
        let pick_num = self
            .current_prompt
            .as_ref()
            .map(|prompt| prompt.pick)
            .unwrap_or(1) as usize;
        ClientBoundPacket::DisplayResponses(
            self.players
                .iter()
                .filter(|(_, player)| player.selections.len() == pick_num)
                .map(|(id, player)| {
                    (
                        *id,
                        player
                            .selections
                            .iter()
                            .map(|&card| ResponseData {
                                text: self.response_text(card),
                                id: card,
                            })
                            .collect(),
                    )
                })
                .collect(),
        )
    }

    async fn next_round(&mut self, network_handler: &mut NetworkHandler) {
        self.state = GameState::Playing(PlayingState::PlayerSelection);

        for player in self.players.values_mut() {
            player.selections.clear();
        }

        let last_czar = self.czar_index;
        self.czar_index = (self.czar_index + 1) % self.players.len();

        let add_cards = self
            .current_prompt
            .as_ref()
            .map(|prompt| prompt.pick as usize)
            .unwrap_or(0);
        let prompt = self.select_prompt();

        let mut client_handler = network_handler.client_handler.lock().await;
        for index in 0 .. self.players.len() {
            let mut responses = Vec::with_capacity(add_cards);
            if index != last_czar {
                self.add_responses(&mut responses, add_cards);
            }

            let round_data = ClientBoundPacket::NextRound {
                czar: self.czar_index,
                prompt: prompt.clone(),
                new_responses: responses,
            };

            client_handler
                .send_packet(self.players[index].0, &round_data)
                .await;
        }

        self.current_prompt = Some(prompt);
    }

    async fn broadcast_to_players<'a>(
        &self,
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
        packets.push(ClientBoundPacket::SettingUpdate(GameSetting::PointsToWin(
            self.points_to_win,
        )));
        for pack in self.packs.iter() {
            packets.push(ClientBoundPacket::SettingUpdate(GameSetting::AddPack(
                pack.name.clone(),
            )));
        }
        packets
    }

    pub fn num_players(&self) -> usize {
        self.players.len()
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

        if let GameState::Playing(playing_state) = self.state {
            let mut responses = Vec::with_capacity(10);
            self.add_responses(&mut responses, 10);
            if let Some(prompt) = self.current_prompt.as_ref() {
                packets.push(ClientBoundPacket::NextRound {
                    czar: self.czar_index,
                    prompt: prompt.clone(),
                    new_responses: responses,
                });
            }

            if playing_state == PlayingState::CzarSelection {
                let responses = self
                    .players
                    .iter()
                    .map(|(id, player)| {
                        (
                            *id,
                            player
                                .selections
                                .iter()
                                .copied()
                                .map(|card| ResponseData {
                                    text: self.response_text(card),
                                    id: card,
                                })
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect::<HashMap<_, _>>();
                packets.push(ClientBoundPacket::DisplayResponses(responses));
            }
        }

        client_handler.send_packets(client_id, &packets).await;
    }

    async fn client_disconnected(
        &mut self,
        network_handler: &mut NetworkHandler,
        client_id: usize,
    ) {
        // Cancel the round if the czar left
        let skip_round = client_id == self.players[self.czar_index].0;

        let player = match self.players.remove(&client_id) {
            Some(player) => player,
            None => return,
        };

        let new_host = if player.is_host {
            self.players
                .first_mut()
                .map(|(id, player)| {
                    player.is_host = true;
                    id
                })
                .copied()
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

        let mut client_handler = network_handler.client_handler.lock().await;
        self.broadcast_to_players(&mut client_handler, &packet)
            .await;

        if skip_round {
            self.czar_index %= self.players.len();
            self.broadcast_to_players(&mut client_handler, &ClientBoundPacket::CancelRound)
                .await;
            drop(client_handler);
            self.current_prompt = None;
            self.next_round(network_handler).await;
        }
    }

    async fn handle_packet(
        &mut self,
        network_handler: &mut NetworkHandler,
        packet: &ServerBoundPacket,
        sender_id: usize,
    ) -> PacketResponse {
        match self.state {
            GameState::WaitingToStart => {
                match packet {
                    ServerBoundPacket::StartGame => {
                        if self.host_id != sender_id {
                            return PacketResponse::Rejected;
                        }

                        // TODO: Initialize round
                        self.initialize_prompts();
                        self.initialize_responses();

                        // This branch should never be taken
                        if self.available_responses.is_empty() || self.available_prompts.is_empty()
                        {
                            return PacketResponse::RejectedWithReason(
                                "No packs selected".to_owned(),
                            );
                        }

                        // Select the first czar
                        self.czar_index = thread_rng().gen_range(0 .. self.players.len());

                        // Select the prompt
                        let prompt = self.select_prompt();

                        let mut client_handler = network_handler.client_handler.lock().await;
                        for index in 0 .. self.players.len() {
                            let mut responses = Vec::with_capacity(10);
                            self.add_responses(&mut responses, 10);

                            let round_data = ClientBoundPacket::NextRound {
                                czar: self.czar_index,
                                prompt: prompt.clone(),
                                new_responses: responses,
                            };

                            client_handler
                                .send_packets(self.players[index].0, &[
                                    ClientBoundPacket::StartGame,
                                    round_data,
                                ])
                                .await;
                        }

                        self.current_prompt = Some(prompt);
                        self.state = GameState::Playing(PlayingState::PlayerSelection);
                    }

                    ServerBoundPacket::UpdateSetting(setting) => {
                        if self.host_id != sender_id {
                            return PacketResponse::Rejected;
                        }

                        match setting {
                            &GameSetting::MaxPlayers(limit) => self.max_players = limit,
                            &GameSetting::MaxSelectionTime(limit) =>
                                self.max_selection_time = limit,
                            &GameSetting::PointsToWin(points) => self.points_to_win = points,
                            GameSetting::AddPack(pack_name) => {
                                if self.packs.iter().any(|pack| &pack.name == pack_name) {
                                    return PacketResponse::Rejected;
                                }

                                let pack = match self.pack_store.borrow_mut().load_pack(pack_name) {
                                    Ok(pack) => pack,
                                    Err(e) => {
                                        let error =
                                            format!("Failed to load pack {}: {}", pack_name, e);
                                        error!("{}", error);
                                        return PacketResponse::RejectedWithReason(error);
                                    }
                                };
                                self.packs.push(pack);
                            }
                            GameSetting::RemovePack(pack_name) => {
                                self.packs.retain(|pack| &pack.name != pack_name);
                            }
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

                    _ => return PacketResponse::Rejected,
                }
            }
            GameState::Playing(PlayingState::PlayerSelection) => {
                let card = match packet {
                    &ServerBoundPacket::SelectResponse(card) => card,
                    _ => return PacketResponse::Rejected,
                };

                let czar_id = self.players[self.czar_index].0;
                if sender_id == czar_id {
                    return PacketResponse::Rejected;
                }

                let pick_num = match self.current_prompt.as_ref().map(|prompt| prompt.pick) {
                    Some(pick_num) => pick_num as usize,
                    None => return PacketResponse::Rejected,
                };

                let selections = match self
                    .players
                    .get_mut(&sender_id)
                    .map(|player| &mut player.selections)
                {
                    Some(selections) => selections,
                    None => return PacketResponse::Rejected,
                };


                if selections.len() >= pick_num {
                    return PacketResponse::Rejected;
                }

                selections.push(card);

                if selections.len() == pick_num {
                    self.broadcast_to_players(
                        &mut network_handler.client_handler.lock().await,
                        &ClientBoundPacket::PlayerFinishedPicking(sender_id),
                    )
                    .await;
                }

                if self
                    .players
                    .iter()
                    .all(|(id, player)| *id == czar_id || player.selections.len() == pick_num)
                {
                    let display_responses = self.display_responses();

                    self.broadcast_to_players(
                        &mut network_handler.client_handler.lock().await,
                        &display_responses,
                    )
                    .await;

                    self.state = GameState::Playing(PlayingState::CzarSelection);
                }
            }
            GameState::Playing(PlayingState::CzarSelection) => {
                let winner_id = match packet {
                    &ServerBoundPacket::SelectRoundWinner(winner_id) => winner_id,
                    _ => return PacketResponse::Rejected,
                };

                let czar_id = self.players[self.czar_index].0;
                if sender_id != czar_id {
                    return PacketResponse::Rejected;
                }

                let winner = match self.players.get_mut(&(winner_id as usize)) {
                    Some(winner) => winner,
                    None =>
                        return PacketResponse::RejectedWithReason(format!(
                            "Invalid player ID: {}",
                            winner_id
                        )),
                };

                winner.points += 1;
                let end_game = winner.points >= self.points_to_win;
                self.broadcast_to_players(
                    &mut network_handler.client_handler.lock().await,
                    &ClientBoundPacket::DisplayWinner {
                        winner: winner_id as usize,
                        end_game,
                    },
                )
                .await;

                if end_game {
                    self.state = GameState::End;
                } else {
                    self.next_round(network_handler).await;
                }
            }
            GameState::End => {}
        }

        PacketResponse::Accepted
    }
}

struct Player {
    client_id: usize,
    name: String,
    is_host: bool,
    points: u32,
    selections: Vec<CardID>,
}

impl Player {
    pub fn new(client_id: usize, is_host: bool) -> Self {
        Player {
            client_id,
            name: format!("Player #{}", client_id),
            is_host,
            points: 0,
            selections: Vec::new(),
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

#[derive(Clone, Copy, Debug)]
enum GameState {
    WaitingToStart,
    Playing(PlayingState),
    End,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum PlayingState {
    PlayerSelection,
    CzarSelection,
}
