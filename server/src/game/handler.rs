use crate::{
    chan::{channel, OneshotTx, Rx, Tx},
    client::ClientMessage,
};
use tokio::task;

use super::{GameMessage, JoinGame, JoinResponse, LeaveGame, PlayerGameId};

pub fn create_game(
    game_id: u16,
    creator_client: Tx<ClientMessage>,
    creator_username: String,
) -> GameHandle {
    let (tx, rx) = channel();

    let creator = PlayerHandle {
        client: creator_client,
        username: creator_username.clone(),
    };

    task::spawn(handle_game(rx, creator));

    GameHandle {
        game: tx,
        creator_username,
        creator_id: 0,
    }
}

async fn handle_game(mut rx: Rx<GameMessage>, creator: PlayerHandle) {
    let mut game = Game::new(creator);

    while let Some(message) = rx.recv().await {
        game.handle_message(message);
    }
}

#[derive(Clone)]
pub struct GameHandle {
    pub game: Tx<GameMessage>,
    pub creator_username: String,
    pub creator_id: PlayerGameId,
}

struct Game {
    players: Vec<Option<PlayerHandle>>,
}

impl Game {
    fn new(creator: PlayerHandle) -> Self {
        Self {
            players: vec![Some(creator)],
        }
    }

    fn handle_message(&mut self, message: GameMessage) {
        match message {
            GameMessage::JoinGame(JoinGame { client, username }, response) =>
                response.send(self.handle_join_game(client, username)),
            GameMessage::LeaveGame(LeaveGame { id }) => self.handle_leave_game(id),
        }
    }

    fn handle_join_game(&mut self, client: Tx<ClientMessage>, username: String) -> JoinResponse {
        // TODO: check against max player/spectator amount
        self.players.push(Some(PlayerHandle { client, username }));

        JoinResponse::JoinAsPlayer {
            id: self.players.len(),
        }
    }

    fn handle_leave_game(&mut self, id: PlayerGameId) {
        todo!()
    }
}

struct PlayerHandle {
    client: Tx<ClientMessage>,
    username: String,
}
