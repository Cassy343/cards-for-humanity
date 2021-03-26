use super::GameSetting;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerBoundPacket {
    SetPlayerName(String),
    StartGame,
    UpdateSetting(GameSetting),
    SelectResponse(usize),
}
