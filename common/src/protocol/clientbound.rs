use std::collections::HashMap;

use super::GameSetting;
use crate::data::cards::{CardID, Prompt, Response};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientBoundPacket {
    SetId(usize),
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
    PlayerFinishedPicking(usize),
    DisplayResponses(HashMap<usize, Vec<ResponseData>>),
    NextRound {
        czar: usize,
        prompt: Prompt,
        new_responses: Vec<ResponseData>,
    },
    CancelRound,
    DisplayWinner {
        winner: usize,
        end_game: bool,
    },
    Ack {
        packet_id: Uuid,
        response: PacketResponse,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PacketResponse {
    Accepted,
    Rejected,
    RejectedWithReason(String),
}
