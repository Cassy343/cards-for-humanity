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

pub fn encode<P: Serialize>(packet: &P) -> String {
    serde_json::to_string(packet).unwrap()
}

pub fn decode<'de, P: Deserialize<'de>>(data: &'de str) -> Result<P, serde_json::Error> {
    serde_json::from_str(data)
}
