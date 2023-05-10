use serde::Serialize;

use crate::{chan::Tx, client::ClientMessage, proto};

use super::PlayerGameId;

proto!(
    GameMessage,
    with_response: {
        JoinGame: JoinResponse
    },
    without_response: [LeaveGame]
);

pub struct JoinGame {
    pub client: Tx<ClientMessage>,
    pub username: String,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum JoinResponse {
    JoinAsPlayer { id: PlayerGameId },
    JoinAsSpectator,
    Rejected,
}

pub struct LeaveGame {
    pub id: PlayerGameId,
}
