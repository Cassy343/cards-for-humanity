use std::{borrow::{Borrow, BorrowMut}, cell::RefCell, collections::HashMap, rc::Rc, sync::{mpsc::Receiver, Arc, Mutex}};

use common::{
    data::cards::{CardID, Prompt, Response},
    protocol::{clientbound::ClientBoundPacket, serverbound::ServerBoundPacket},
};
use nalgebra::Vector2;
use uuid::Uuid;
use wasm_bindgen::{JsCast, JsValue, closure::WasmClosure, prelude::Closure};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlCanvasElement, MouseEvent, WebGlRenderingContext};

use crate::{console_log, rendering::{BASE_RESOLUTION, Color, RenderManager, Renderable, shapes::*}, ws::WebSocket};

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
    objects: HashMap<String, Box<dyn Renderable>>,
    clickables: Vec<Box<dyn Clickable>>,
    render_manager: RenderManager,
    socket: Arc<Mutex<WebSocket>>,
}


pub async fn game_init(socket: WebSocket, packet_receiver: Arc<Receiver<ClientBoundPacket>>) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let webgl_canvas = document.get_element_by_id("webgl").unwrap();
    let webgl_canvas: web_sys::HtmlCanvasElement = webgl_canvas.dyn_into().unwrap();
    let text_canvas = document.get_element_by_id("text").unwrap();
    let text_canvas: web_sys::HtmlCanvasElement = text_canvas.dyn_into().unwrap();

    let render_manager = render_init(&text_canvas, &webgl_canvas).await.unwrap();

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
        objects: HashMap::new(),
        clickables: Vec::new(),
        render_manager,
        socket: Arc::new(Mutex::new(socket)),
    }));

    let game_manager = manager.clone();

    let click_manager = game_manager.clone();
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
    spawn_local(async move {
        draw_loop(draw_manager.clone());
    });
}

async fn render_init(
    text_canvas: &HtmlCanvasElement,
    webgl_canvas: &HtmlCanvasElement,
) -> Result<RenderManager, JsValue> {
    let mut manager = RenderManager::new(webgl_canvas, text_canvas)?;

    manager.register_shader(
        "card",
        &super::fetch("./shaders/card.vert")
            .await
            .expect("error getting shader"),
        &super::fetch("./shaders/card.frag")
            .await
            .expect("error getting shader"),
        WebGlRenderingContext::TRIANGLE_STRIP,
    )?;

    manager.set_background_color(Color::from_rgb(0x1e, 0x34, 0x54));
    manager.clear();

    Ok(manager)
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
                        Vector2::new(50.0, 70.0),
                    )
                }));

            if manager.objects.contains_key("prompt_card") {
                manager.objects.remove("prompt_card").unwrap();
            }

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

fn draw_loop(manager: Arc<Mutex<GameManager>>) {
    console_log!("getting manager");
    let mut manager = match manager.lock() {
        Ok(m) => m,
        Err(e) => {
            console_log!("Error getting manager: {}", e);
            return;
        }
    };

    manager.render_manager.clear();

    console_log!("drawing objects");
    manager
        .render_manager
        .draw_object(&manager.hand as &dyn Renderable)
        .unwrap();

    console_log!("{}", manager.player.name);

    let mut players: Vec<_> = manager.others.values().map(|c| c.clone()).collect();
    players.push(manager.player.clone());

    manager
        .render_manager
        .draw_object(&players as &dyn Renderable)
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
                            CardID {
                                card_number: 0,
                                pack_number: 0,
                            },
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
    console_log!("running click checks");

    let pos = Vector2::new(event.client_x() as f32, event.client_y()as f32).component_div(&Vector2::from(BASE_RESOLUTION));

    let manager = manager.lock().unwrap();

    for clickable in &manager.clickables {
        if clickable.click_check(pos) {
            clickable.click(&manager, pos).unwrap();
        };
    }

    for card in &manager.hand {
        if card.click_check(pos) {
            card.click(&manager, pos).unwrap();
        }
    }
}


type Hand = Vec<ResponseCard>;

pub trait Clickable {
    fn click_check(&self, pos: Vector2<f32>) -> bool;
    fn click(&self, manager: &GameManager, pos: Vector2<f32>) -> Result<(), String>;
}

impl Clickable for RoundedRect {
    fn click_check(&self, pos: Vector2<f32>) -> bool {
        console_log!("click pos: {}\nour pos: {}", pos, self.position);
        pos.x >= self.position.x
            && pos.x <= (self.position.x + self.dimensions.x)
            && pos.y >= self.position.y
            && pos.y <= (self.position.y + self.dimensions.y)
    }

    fn click(&self, _: &GameManager, _: Vector2<f32>) -> Result<(), String> {
        console_log!("I've been clicked :O");
        Ok(())
    }
}

impl Clickable for TextBubble {
    fn click_check(&self, pos: Vector2<f32>) -> bool {
        self.rect.click_check(pos)
    }

    fn click(&self, _: &GameManager, _: Vector2<f32>) -> Result<(), String> {
        console_log!("{} was clicked", self.text.text);
        Ok(())
    }
}

impl Clickable for ResponseCard {
    fn click_check(&self, pos: Vector2<f32>) -> bool {
        self.inner.click_check(pos)
    }

    fn click(&self, manager: &GameManager, _: Vector2<f32>) -> Result<(), String> {
        match manager.state {
            // There should be no cards on the lobby and if there are they shouldn't do anything
            GameState::Lobby => {}
            GameState::MakeResponse(_) =>
                if !manager.is_czar {
                    let socket = manager.socket.lock().unwrap();
                    socket
                        .send_packet(&ServerBoundPacket::SelectResponse(self.id))
                        .unwrap();
                },
            GameState::PickResponse(_) =>
                if manager.is_czar {
                    let socket = manager.socket.lock().unwrap();
                    socket
                        .send_packet(&ServerBoundPacket::SelectRoundWinner(
                            ((self.inner.rect.position.x - 20.0) / 50.0) as usize,
                        ))
                        .unwrap();
                },
            // Don't do anything if we're just waiting
            // In future could allow you to highlight cards or smth?
            GameState::Waiting => {}
        }

        Ok(())
    }
}
