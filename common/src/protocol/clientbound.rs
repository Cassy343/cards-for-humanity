use std::collections::HashMap;

use super::{serverbound::ServerBoundPacket, GameSetting};
use crate::data::cards::{CardID, Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientBoundPacket {
    StartGame,
    SettingUpdate(GameSetting),
    AddPlayer {
        id: usize,
        name: String,
        is_host: bool,
        points: u32,
    },
    UpdatePlayerName {
        id: usize,
        name: String,
    },
    RemovePlayer {
        id: usize,
        new_host: Option<usize>,
    },
    DisplayResponses(HashMap<usize, Vec<Response>>),
    NextRound {
        is_czar: bool,
        new_responses: Vec<ResponseData>,
    },
    EndGame {
        winner: usize,
    },
}

impl ClientBoundPacket {
    pub fn echo_setting_update(setting: &GameSetting) -> Self {
        ClientBoundPacket::SettingUpdate(setting.clone())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseData {
    pub id: CardID,
    pub text: Response,
}

impl ResponseData {
    pub fn new(id: CardID, text: Response) -> Self {
        ResponseData { id, text }
    }
}
