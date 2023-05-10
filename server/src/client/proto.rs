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

#[derive(Serialize)]
pub struct PlayerListUpdate {
    host: PlayerGameId,
    players: Vec<PlayerInfo>,
}

#[derive(Serialize)]
pub struct PlayerInfo {
    id: PlayerGameId,
    name: String,
    points: u32,
}
