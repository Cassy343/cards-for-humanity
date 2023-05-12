use serde::Serialize;

use crate::{game::PlayerGameId, proto};

use super::{ExternalClientMessage, ExternalServerMessage};

proto!(
    ClientMessage,
    with_response: {},
    without_response: [External, WsDisconnected, PlayerListUpdate]
);

pub struct External(pub ExternalServerMessage);

pub struct WsDisconnected;

#[derive(Serialize, Clone)]
pub struct PlayerListUpdate {
    pub host: PlayerGameId,
    pub players: Vec<PlayerInfo>,
}

#[derive(Serialize, Clone)]
pub struct PlayerInfo {
    pub id: PlayerGameId,
    pub username: String,
    pub points: u32,
}
