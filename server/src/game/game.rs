use std::collections::HashMap;
use crate::network::{Listener, NetworkHandler};
use common::data::cards::{Pack, Prompt, Response};
use rand::{Rng, thread_rng};
use std::rc::Rc;
use common::protocol::serverbound::ServerBoundPacket;

pub struct Game {
    players: HashMap<usize, Player>,
    packs: Vec<Rc<Pack>>,
    available_prompts: Vec<(usize, usize)>,
    available_responses: Vec<(usize, usize)>
}

impl Game {
    pub fn new() -> Self {
        Game {
            players: HashMap::new(),
            packs: Vec::new(),
            available_prompts: Vec::new(),
            available_responses: Vec::new()
        }
    }

    fn initialize_prompts(&mut self) {
        for (index, pack) in self.packs.iter().enumerate() {
            self.available_prompts.extend((0..pack.prompts.len()).map(|j| (index, j)));
        }
    }

    fn initialize_responses(&mut self) {
        for (index, pack) in self.packs.iter().enumerate() {
            self.available_responses.extend((0..pack.responses.len()).map(|j| (index, j)));
        }
    }

    fn select_prompt(&mut self) -> Prompt {
        if self.available_prompts.is_empty() {
            self.initialize_prompts();
        }

        let (pack, choice) = self.available_prompts[thread_rng().gen_range(0..self.available_prompts.len())];
        self.packs[pack].prompts[choice].clone()
    }

    fn select_response(&mut self) -> Response {
        if self.available_responses.is_empty() {
            self.initialize_responses();
        }

        let (pack, choice) = self.available_responses[thread_rng().gen_range(0..self.available_responses.len())];
        self.packs[pack].responses[choice].clone()
    }
}

impl Listener for Game {
    fn handle_packet(&mut self, network_handler: &mut NetworkHandler, packet: &ServerBoundPacket, sender_id: usize) {
        log::debug!("Received message from client {}: {:?}", sender_id, packet);
    }
}

struct Player {
    client_id: usize,
    points: u32
}

enum GameState {
    WaitingToStart,
    Playing(PlayingState),
    End
}

enum PlayingState {
    PlayerSelection,
    CzarSelection
}