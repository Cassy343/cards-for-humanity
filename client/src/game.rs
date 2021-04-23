use std::{
    collections::HashMap,
    sync::{mpsc::Receiver, Arc, Mutex},
};

use common::{data::cards::{CardID, Prompt, Response}, protocol::{clientbound::ClientBoundPacket, serverbound::ServerBoundPacket}};
use nalgebra::Vector2;
use uuid::Uuid;
use wasm_bindgen::{prelude::Closure, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlElement, MouseEvent};

use crate::{
    console_log,
    rendering::{shapes::*, RenderManager, Renderable},
    ws::WebSocket,
};

struct Player {
    name: String,
    points: u32,
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
    objects: HashMap<String, Box<dyn Renderable>>,
    clickables: Vec<Box<dyn Clickable>>,
    render_manager: RenderManager,
    socket: Arc<Mutex<WebSocket>>  
}


pub fn game_init(socket: WebSocket, packet_receiver: Arc<Receiver<ClientBoundPacket>>) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let webgl_canvas = document.get_element_by_id("webgl").unwrap();
    let webgl_canvas: web_sys::HtmlCanvasElement = webgl_canvas.dyn_into().unwrap();
    let text_canvas = document.get_element_by_id("text").unwrap();
    let text_canvas: web_sys::HtmlCanvasElement = text_canvas.dyn_into().unwrap();

    let manager = Arc::new(Mutex::new(GameManager {
        player: Player {
            name: "".to_owned(),
            points: 0,
        },
        id: Uuid::from_u128(0),
        packet_cache: HashMap::new(),
        others: HashMap::new(),
        state: GameState::Lobby,
        hand: Vec::new(),
        is_czar: false,
        host: Uuid::from_u128(0),
        objects: HashMap::new(),
        clickables: Vec::new(),
        render_manager: RenderManager::new(&webgl_canvas, &text_canvas).unwrap(),
        socket: Arc::new(Mutex::new(socket))
    }));

    
    let game_manager = manager.clone();
    
    let click_manager = game_manager.clone();
    let text_canvas: HtmlElement = text_canvas.dyn_into().unwrap();
    let click_callback = Closure::<dyn FnMut(MouseEvent)>::new(move |event: MouseEvent| click_callback(event, click_manager.clone()));
    text_canvas.set_onclick(Some(click_callback.as_ref().unchecked_ref()));
    click_callback.forget();

    spawn_local(async move {
        while let Ok(packet) = packet_receiver.clone().try_recv() {
            console_log!("Packet received: {:?}", packet);
            game_loop(game_manager.clone(), packet)
        }
    });

    let draw_manager = manager.clone();

    let draw_loop = Closure::<dyn FnMut()>::new(move || draw_loop(draw_manager.clone()));

    web_sys::window()
        .unwrap()
        .set_interval_with_callback_and_timeout_and_arguments_0(
            draw_loop.as_ref().unchecked_ref(),
            50,
        )
        .unwrap();
    draw_loop.forget();
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
                    ResponseCard::new(
                        &card.text,
                        card.id,
                        Vector2::new((20 + (hand_len + i) * 30) as f32, 500.0),
                        Vector2::new(20.0, 30.0),
                    )
                }));

            manager.objects.remove("prompt_card").unwrap();

            manager.state = GameState::MakeResponse(prompt);
        }
        ClientBoundPacket::DisplayResponses(responses) => {
            manager.state = GameState::PickResponse(
                responses.iter().map(|e| e.1.to_owned()).flatten().collect(),
            );
        }
        ClientBoundPacket::StartGame => {}
        _ => unimplemented!(),
    }
}

fn draw_loop(manager: Arc<Mutex<GameManager>>) {
    let mut manager = manager.lock().unwrap();

    manager
        .render_manager
        .draw_object(&manager.hand as &dyn Renderable)
        .unwrap();

    match &manager.state {
        GameState::MakeResponse(prompt) => {
            if !manager.objects.contains_key("prompt_card") {
                let prompt = PromptCard::new(
                    &prompt.text,
                    Vector2::new(50.0, 50.0),
                    Vector2::new(20.0, 30.0),
                );

                manager
                    .objects
                    .insert("prompt_card".to_owned(), Box::new(prompt));

                // Switch to Waiting as the czar doesn't have anything to do rn
                if manager.is_czar {
                    manager.state = GameState::Waiting;
                }
            }
        }
        GameState::PickResponse(responses) => {
            if !manager.objects.contains_key("responses") {
                let response_hand: Hand = responses
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        ResponseCard::new(
                            v,
                            CardID {card_number: 0, pack_number: 0},
                            Vector2::new(20. + i as f32 * 50., 50.),
                            Vector2::new(20.0, 30.0),
                        )
                    })
                    .collect();

                manager
                    .objects
                    .insert("responses".to_owned(), Box::new(response_hand));

                if !manager.is_czar {
                    // Everyone else is waiting for the czar to choose the winner
                    manager.state = GameState::Waiting;
                }
            }
        }
        _ => {}
    }

    manager
        .render_manager
        .draw_objects(manager.objects.iter().map(|(_, v)| v.as_ref()).collect())
        .unwrap();
}

fn click_callback(event: MouseEvent, manager: Arc<Mutex<GameManager>>) {

}


type Hand = Vec<ResponseCard>;

pub trait Clickable {
    fn click_check(&self, pos: Vector2<i32>) -> bool;
    fn click(&self, manager: GameManager, pos: Vector2<i32>) -> Result<(), String>;
}

impl Clickable for RoundedRect {
    fn click_check(&self, pos: Vector2<i32>) -> bool {
        pos.x >= self.position.x as i32
            && pos.x <= (self.position.x + self.dimensions.x) as i32
            && pos.y >= self.position.y as i32
            && pos.y <= (self.position.y + self.dimensions.y) as i32
    }

    fn click(&self, _: GameManager, _: Vector2<i32>) -> Result<(), String> {
        Ok(())
    }
}

impl Clickable for TextBubble {
    fn click_check(&self, pos: Vector2<i32>) -> bool {
        self.rect.click_check(pos)
    }

    fn click(&self, _: GameManager, _: Vector2<i32>) -> Result<(), String> {
        Ok(())
    }
}

impl Clickable for ResponseCard {
    fn click_check(&self, pos: Vector2<i32>) -> bool {
        self.inner.click_check(pos)
    }

    fn click(&self, manager: GameManager, _: Vector2<i32>) -> Result<(), String> {
        
        match manager.state {
            // There should be no cards on the lobby and if there are they shouldn't do anything
            GameState::Lobby => {},
            GameState::MakeResponse(_) => {
                if !manager.is_czar {
                    let socket = manager.socket.lock().unwrap();
                    socket.send_packet(&ServerBoundPacket::SelectResponse(self.id)).unwrap();
                }
            },
            GameState::PickResponse(_) => {
                if manager.is_czar {
                    let socket = manager.socket.lock().unwrap();
                    socket.send_packet(&ServerBoundPacket::SelectRoundWinner(((self.inner.rect.position.x - 20.0) / 50.0) as usize)).unwrap();
                }
            },
            // Don't do anything if we're just waiting
            // In future could allow you to highlight cards or smth?
            GameState::Waiting => {}
        }

        Ok(())
    }
}