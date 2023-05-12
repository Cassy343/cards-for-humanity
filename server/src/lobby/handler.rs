use std::collections::HashMap;

use rand::{thread_rng, Rng};
use tokio::task;

use crate::{
    chan::{channel, Rx, Tx},
    client::ClientMessage,
    game::{create_game, GameHandle},
};

use super::{CreateGame, GetGameHandle, LobbyMessage};

pub fn open_lobby() -> Tx<LobbyMessage> {
    let (tx, rx) = channel();
    task::spawn(handle_lobby(rx));
    tx
}

async fn handle_lobby(mut rx: Rx<LobbyMessage>) {
    let mut lobby = Lobby::new();

    while let Some(request) = rx.recv().await {
        lobby.handle_request(request);
    }
}

struct Lobby {
    games: HashMap<u16, GameHandle>,
}

impl Lobby {
    fn new() -> Self {
        Self {
            games: HashMap::new(),
        }
    }

    fn handle_request(&mut self, request: LobbyMessage) {
        match request {
            LobbyMessage::CreateGame(CreateGame { client, username }, response) =>
                response.send(self.handle_create_game(client, username)),
            LobbyMessage::GetGameHandle(GetGameHandle { id }, response) =>
                response.send(self.handle_get_game_handle(id)),
        }
    }

    fn handle_create_game(&mut self, client: Tx<ClientMessage>, username: String) -> GameHandle {
        let mut rng = thread_rng();
        let mut game_id: u16 = rng.gen();
        while self.games.contains_key(&game_id) {
            game_id = rng.gen();
        }

        let handle = create_game(client, username, game_id);
        self.games.insert(game_id, handle.clone());
        handle
    }

    fn handle_get_game_handle(&self, id: u16) -> Option<GameHandle> {
        self.games.get(&id).cloned()
    }
}
