use std::collections::HashMap;

use super::GameSetting;
use crate::data::cards::{CardID, Pack, Prompt, Response};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientBoundPacket {
    SetId(Uuid),
    StartGame,
    SettingUpdate(GameSetting),
    AddPlayer {
        id: Uuid,
        name: String,
        is_host: bool,
        points: u32,
    },
    UpdatePlayerName {
        id: Uuid,
        name: String,
    },
    RemovePlayer {
        id: Uuid,
        new_host: Option<Uuid>,
    },
    PlayerFinishedPicking(Uuid),
    DisplayResponses(HashMap<Uuid, Vec<ResponseData>>),
    NextRound {
        czar: Uuid,
        prompt: Prompt,
        new_responses: Vec<ResponseData>,
    },
    CancelRound,
    DisplayWinner {
        winner: Uuid,
        end_game: bool,
    },
    Ack {
        packet_id: Uuid,
        response: PacketResponse,
    },
    ServerList {
        servers: Vec<(Uuid, usize, Option<usize>)>,
    },
    CardPacks(Vec<(String, usize, usize)>),
}

impl ClientBoundPacket {
    pub fn echo_setting_update(setting: &GameSetting) -> Self {
        ClientBoundPacket::SettingUpdate(setting.clone())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
