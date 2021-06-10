pub mod clientbound;
pub mod serverbound;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameSetting {
    MaxPlayers(Option<usize>),
    MaxSelectionTime(Option<u32>),
    PointsToWin(u32),
    AddPack(String),
    RemovePack(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameSettings {
    pub max_players: Option<usize>,
    pub max_selection_time: Option<u32>,
    pub points_to_win: u32,
    pub packs: Vec<String>,
}

pub fn encode<P: Serialize>(packet: &P) -> String {
    serde_json::to_string(packet).unwrap()
}

pub fn decode<'de, P: Deserialize<'de>>(data: &'de str) -> Result<P, serde_json::Error> {
    serde_json::from_str(data)
}
