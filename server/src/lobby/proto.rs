use serde::Serialize;

use crate::{chan::Tx, client::ClientMessage, game::GameHandle, proto};

proto!(
    LobbyMessage,
    with_response: {
        CreateGame: GameHandle,
        GetGameHandle: Option<GameHandle>
    },
    without_response: []
);

pub struct CreateGame {
    pub client: Tx<ClientMessage>,
    pub username: String,
}

pub struct GetGameHandle {
    pub id: u16,
}
