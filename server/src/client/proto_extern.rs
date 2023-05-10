use serde::{Deserialize, Serialize};

use crate::game::JoinResponse;

use super::proto::PlayerListUpdate;

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum ExternalServerMessage {
    SetUsername { username: String },
    CreateGame,
    JoinGame { id: u16 },
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum ExternalClientMessage {
    PlayerListUpdate {
        #[serde(flatten)]
        payload: PlayerListUpdate,
    },
    JoinResponse {
        response: Option<JoinResponse>,
    },
}
