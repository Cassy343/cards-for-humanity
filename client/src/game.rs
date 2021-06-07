use std::{
    collections::HashMap,
    sync::{mpsc::Receiver, Arc, Mutex},
    u128,
};

use common::{
    data::cards::CardID,
    protocol::{
        clientbound::{ClientBoundPacket, PacketResponse},
        serverbound::ServerBoundPacket,
        GameSetting,
    },
};
use uuid::Uuid;

use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::HtmlElement;

use crate::{
    console_log,
    html::{
        add_card_to_hand,
        add_player,
        clear_player_marks,
        clear_response_cards,
        get_hand_element,
        init_game,
        mark_player_czar,
        mark_player_played,
        place_blank_response,
        remove_card_from_hand,
        remove_player,
        set_player_responses,
        set_prompt_card,
        set_user_name,
        set_user_points,
        update_player_name,
        update_player_points,
    },
    ws::WebSocket,
};


#[derive(Clone)]
pub struct Player {
    pub name: String,
    pub points: u32,
}

#[derive(Debug)]
enum GameState {
    Lobby,
    MakeResponse(u8),
    PickResponse(HashMap<Uuid, Vec<ResponseCard>>),
    Waiting,
}

#[derive(Debug)]
enum CachedPacket {
    SelectResponse(ResponseCard, usize),
    StartGame,
    UpdateSetting(GameSetting),
    SelectRoundWinner,
    SetPlayerName(String),
}

pub struct GameManager {
    player: Player,
    id: Uuid,
    packet_cache: HashMap<Uuid, CachedPacket>,
    others: HashMap<Uuid, Player>,
    state: GameState,
    hand: Hand,
    is_czar: bool,
    host: Uuid,
    socket: Arc<Mutex<WebSocket>>,
    /// We store the card closures to appropriately drop them when we need to
    /// normally we would .forget() the Closures but since we register a lot of Closures
    /// we want to responsibly drop them to avoid leaking memory
    hand_closures: HashMap<CardID, Closure<dyn FnMut()>>,
    response_closures: Vec<Closure<dyn FnMut()>>,
}


pub fn game_init(socket: WebSocket, packet_receiver: Receiver<ClientBoundPacket>) {
    let manager = Arc::new(Mutex::new(GameManager {
        player: Player {
            name: "Test".to_owned(),
            points: 0,
        },
        id: Uuid::from_u128(0),
        packet_cache: HashMap::new(),
        others: HashMap::new(),
        state: GameState::Lobby,
        hand: Vec::new(),
        is_czar: false,
        host: Uuid::from_u128(0),
        socket: Arc::new(Mutex::new(socket)),
        hand_closures: HashMap::new(),
        response_closures: Vec::new(),
    }));

    init_game();

    let game_manager = manager.clone();

    let game_loop = Closure::<dyn FnMut()>::new(move || {
        while let Ok(packet) = packet_receiver.try_recv() {
            console_log!("Packet received: {:?}", packet);
            game_loop(game_manager.clone(), packet)
        }
    });
    web_sys::window()
        .unwrap()
        .set_interval_with_callback_and_timeout_and_arguments_0(
            game_loop.as_ref().unchecked_ref(),
            50,
        )
        .unwrap();
    game_loop.forget();
}

fn game_loop(manager_arc: Arc<Mutex<GameManager>>, packet: ClientBoundPacket) {
    let mut manager = manager_arc
        .lock()
        .expect("Error getting mut for GameManager");
    match packet {
        ClientBoundPacket::SetId(id) => manager.id = Uuid::from_u128(id as u128),
        ClientBoundPacket::AddPlayer {
            id,
            name,
            is_host,
            points,
        } => {
            if id as u128 != manager.id.as_u128() {
                let player = Player { name, points };

                let id = Uuid::from_u128(id as u128);

                add_player(&player, &id);

                manager.others.insert(id, player);
            }


            if is_host {
                manager.host = Uuid::from_u128(id as u128);
            }
        }

        ClientBoundPacket::NextRound {
            czar,
            prompt,
            new_responses,
        } => {
            manager.is_czar = manager.id.as_u128() == czar as u128;

            if !manager.is_czar {
                mark_player_czar(&Uuid::from_u128(czar as u128))
            }

            manager
                .hand
                .extend(new_responses.iter().map(|card| ResponseCard {
                    text: card.text.clone(),
                    id: card.id,
                }));

            set_prompt_card(&PromptCard {
                text: prompt.text.clone(),
            });

            manager.state = GameState::MakeResponse(prompt.pick);

            drop(manager);

            for (_index, card) in new_responses.iter().enumerate() {
                let element = add_card_to_hand(&ResponseCard {
                    text: card.text.clone(),
                    id: card.id,
                });

                set_hand_onclick(element, card.id, manager_arc.clone())
            }
        }

        ClientBoundPacket::DisplayResponses(responses) => {
            clear_response_cards();

            for (id, _) in &manager.others {
                clear_player_marks(id)
            }

            let res: HashMap<Uuid, Vec<ResponseCard>> = responses
                .into_iter()
                .map(|(id, responses)| {
                    (
                        Uuid::from_u128(id as u128),
                        responses
                            .into_iter()
                            .map(|r| ResponseCard {
                                id: r.id,
                                text: r.text,
                            })
                            .collect(),
                    )
                })
                .collect();

            manager.state = GameState::PickResponse(res.clone());
            drop(manager);

            res.iter().for_each(|(id, responses)| {
                let element = set_player_responses(id, responses);
                set_response_onclick(element, *id, manager_arc.clone())
            });
        }

        ClientBoundPacket::StartGame => {
            set_user_name(&manager.player.name);
            set_user_points(manager.player.points);
        }

        ClientBoundPacket::UpdatePlayerName { id, name } => {
            update_player_name(Uuid::from_u128(id as u128), &name);
            manager
                .others
                .get_mut(&Uuid::from_u128(id as u128))
                .unwrap()
                .name = name;
        }

        ClientBoundPacket::RemovePlayer { id, new_host } => {
            remove_player(Uuid::from_u128(id as u128));
            manager.others.remove(&Uuid::from_u128(id as u128));

            if let Some(host_id) = new_host {
                manager.host = Uuid::from_u128(host_id as u128)
            }

            remove_player(Uuid::from_u128(id as u128));
        }

        ClientBoundPacket::PlayerFinishedPicking(id) =>
            if id as u128 != manager.id.as_u128() {
                mark_player_played(&Uuid::from_u128(id as u128));
                place_blank_response();
            },

        ClientBoundPacket::DisplayWinner { winner, end_game } => {
            clear_response_cards();

            if end_game {
                web_sys::window()
                    .unwrap()
                    .alert_with_message("THE GAME IS OVER")
                    .unwrap();
                panic!("PANIC")
            } else {
                if winner as u128 == manager.id.as_u128() {
                    manager.player.points += 1;
                    set_user_points(manager.player.points);
                } else {
                    let id = Uuid::from_u128(winner as u128);
                    let player = manager.others.get_mut(&id).unwrap();
                    player.points += 1;
                    update_player_points(id, player.points);
                }
            }
        }

        ClientBoundPacket::SettingUpdate(_settings) => {
            // unimplemented!()
        }

        ClientBoundPacket::Ack {
            packet_id,
            response,
        } => {
            console_log!("{:?}\n{}\n{:?}", response, packet_id, manager.packet_cache);
            match response {
                PacketResponse::Accepted => {
                    // If packets are accepted any extra data we might have cached should be dropped
                    let cached = match manager.packet_cache.remove(&packet_id) {
                        Some(p) => p,
                        // This should only be taken with the StartGame packet sent on connect
                        None => return,
                    };
                    match cached {
                        CachedPacket::SelectResponse(card, card_index) => {
                            remove_card_from_hand(card_index as u8);
                            place_blank_response();
                            manager.hand.remove(card_index);
                            manager.hand_closures.remove(&card.id);
                        }
                        CachedPacket::SelectRoundWinner => {
                            manager.response_closures = Vec::new();
                        }
                        _ => todo!(),
                    }
                }
                // TODO: handle RejectedWithReason by showing the reason to the user
                _ => {
                    let packet = match manager.packet_cache.remove(&packet_id) {
                        Some(p) => p,
                        // This should only be taken with the StartGame packet sent on connect
                        None => return,
                    };

                    revert_packet(&mut manager, packet);
                }
            }
        }

        ClientBoundPacket::CancelRound => {
            clear_response_cards();
        }
    }
}

fn revert_packet(manager: &mut GameManager, packet: CachedPacket) {
    match packet {
        CachedPacket::SelectResponse(card, card_index) => {
            get_hand_element(card_index).set_hidden(false);
            manager.hand.push(card);
            manager.state = match manager.state {
                GameState::MakeResponse(picks_left) => GameState::MakeResponse(picks_left + 1),
                _ => GameState::MakeResponse(1),
            }
        }
        _ => {}
    }
}


type Hand = Vec<ResponseCard>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResponseCard {
    pub text: String,
    pub id: CardID,
}

#[derive(Serialize, Deserialize)]
pub struct PromptCard {
    pub text: String,
}


fn set_hand_onclick(element: HtmlElement, card_id: CardID, manager: Arc<Mutex<GameManager>>) {
    let card_manager = manager.clone();
    let card_element = element.clone();
    let card_closure = Closure::<dyn FnMut()>::new(move || {
        hand_click(card_element.clone(), card_id, card_manager.clone())
    });

    element.set_onclick(Some(card_closure.as_ref().unchecked_ref()));

    // Store the closure to drop it later
    let mut manager_mutex = manager.lock().unwrap();
    manager_mutex.hand_closures.insert(card_id, card_closure);
}

fn hand_click(element: HtmlElement, card_id: CardID, manager: Arc<Mutex<GameManager>>) {
    let mut manager = manager.lock().unwrap();
    if !manager.is_czar {
        match manager.state {
            GameState::MakeResponse(picks_left) => {
                // Send card to server
                let socket = manager.socket.lock().unwrap();
                let id = socket
                    .send_packet_with_id(ServerBoundPacket::SelectResponse(card_id))
                    .unwrap();
                drop(socket);

                // Cache the card in case of revert
                let (card_index, card) = manager
                    .hand
                    .iter()
                    .enumerate()
                    .filter(|(_i, f)| f.id == card_id)
                    .next()
                    .unwrap();
                let card = card.clone();
                manager
                    .packet_cache
                    .insert(id, CachedPacket::SelectResponse(card, card_index));

                // Remove the card from the hand
                element.set_hidden(true);

                // Change our GameState
                manager.state = if picks_left > 1 {
                    GameState::MakeResponse(picks_left - 1)
                } else {
                    GameState::Waiting
                };
            }
            // Anything else we don't do anything with the hand
            _ => {}
        }
    }
}

fn set_response_onclick(element: HtmlElement, user_id: Uuid, manager: Arc<Mutex<GameManager>>) {
    let card_manager = manager.clone();
    let card_closure =
        Closure::<dyn FnMut()>::new(move || response_click(user_id, card_manager.clone()));

    element.set_onclick(Some(card_closure.as_ref().unchecked_ref()));

    // Store the closure to drop it later
    let mut manager_mutex = manager.lock().unwrap();
    manager_mutex.response_closures.push(card_closure);
}

fn response_click(user_id: Uuid, manager: Arc<Mutex<GameManager>>) {
    let mut manager = manager.lock().unwrap();
    if manager.is_czar {
        match &manager.state {
            GameState::PickResponse(_) => {
                let socket = manager.socket.lock().unwrap();
                let packet_id = socket
                    .send_packet_with_id(ServerBoundPacket::SelectRoundWinner(
                        user_id.as_u128() as usize
                    ))
                    .unwrap();
                drop(socket);

                manager
                    .packet_cache
                    .insert(packet_id, CachedPacket::SelectRoundWinner);
                manager.state = GameState::Waiting;
            }
            _ => {}
        }
    }
}
