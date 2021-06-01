use std::{borrow::{Borrow, BorrowMut}, cell::RefCell, collections::HashMap, rc::Rc, sync::{mpsc::Receiver, Arc, Mutex}};

use common::{
    data::cards::{CardID, Prompt, Response},
    protocol::{clientbound::ClientBoundPacket, serverbound::ServerBoundPacket},
};
use uuid::Uuid;
use wasm_bindgen::{JsCast, JsValue, closure::WasmClosure, prelude::*};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlCanvasElement, MouseEvent, WebGlRenderingContext};

use serde::{Serialize, Deserialize};

use crate::{console_log, js_imports::update_black_card, ws::WebSocket};

#[derive(Clone)]
pub struct Player {
    pub name: String,
    pub points: u32,
}

enum GameState {
    Lobby,
    MakeResponse(Prompt),
    PickResponse(Vec<Response>),
    Waiting,
}

pub struct GameManager {
    player: Player,
    id: Uuid,
    packet_cache: HashMap<Uuid, ServerBoundPacket>,
    others: HashMap<Uuid, Player>,
    state: GameState,
    hand: Hand,
    is_czar: bool,
    host: Uuid,
    socket: Arc<Mutex<WebSocket>>,
}


pub async fn game_init(socket: WebSocket, packet_receiver: Arc<Receiver<ClientBoundPacket>>) {
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
    }));

    let game_manager = manager.clone();

    while let Ok(packet) = packet_receiver.clone().try_recv() {
        console_log!("Packet received: {:?}", packet);
        game_loop(game_manager.clone(), packet)
    }
}

fn game_loop(manager: Arc<Mutex<GameManager>>, packet: ClientBoundPacket) {
    let mut manager = manager.lock().expect("Error getting mut for GameManager");
    match packet {
        ClientBoundPacket::SetId(id) => manager.id = Uuid::from_u128(id as u128),
        ClientBoundPacket::AddPlayer {
            id,
            name,
            is_host,
            points,
        } => {
            manager
                .others
                .insert(Uuid::from_u128(id as u128), Player { name, points });

            if is_host {
                manager.host = Uuid::from_u128(id as u128)
            }
        }
        ClientBoundPacket::NextRound {
            is_czar,
            prompt,
            new_responses,
        } => {
            manager.is_czar = is_czar;

            let hand_len = manager.hand.len();
            manager
                .hand
                .extend(new_responses.iter().enumerate().map(|(i, card)| {
                    ResponseCard {
                        text: card.text.clone(),
                        id: card.id,
                    }
                }));

            update_black_card(PromptCard {
                text: prompt.text.clone()
            });
            manager.state = GameState::MakeResponse(prompt);
        }
        ClientBoundPacket::DisplayResponses(responses) => {
            manager.state = GameState::PickResponse(
                responses.iter().map(|e| e.1.to_owned()).flatten().collect(),
            );
        }
        ClientBoundPacket::StartGame => {}
        _ => {}
    }
}


type Hand = Vec<ResponseCard>;

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct ResponseCard {
    text: String,
    id: CardID
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct PromptCard {
    text: String
}