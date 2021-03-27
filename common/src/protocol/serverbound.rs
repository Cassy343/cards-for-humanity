use crate::data::cards::CardID;

use super::GameSetting;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum WrappedServerBoundPacket {
    Raw(ServerBoundPacket),
    WithId {
        packet: ServerBoundPacket,
        packet_id: Uuid,
    },
}

impl WrappedServerBoundPacket {
    pub fn packet(&self) -> &ServerBoundPacket {
        match &self {
            WrappedServerBoundPacket::Raw(packet) => packet,
            WrappedServerBoundPacket::WithId { packet, .. } => packet,
        }
    }

    pub fn packet_id(&self) -> Option<Uuid> {
        match self {
            WrappedServerBoundPacket::Raw(_) => None,
            &WrappedServerBoundPacket::WithId { packet_id, .. } => Some(packet_id),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerBoundPacket {
    SetPlayerName(String),
    StartGame,
    UpdateSetting(GameSetting),
    SelectResponse(CardID),
    SelectRoundWinner(usize),
}

impl ServerBoundPacket {
    pub fn with_id(self) -> (WrappedServerBoundPacket, Uuid) {
        let packet_id = Uuid::new_v4();
        (
            WrappedServerBoundPacket::WithId {
                packet: self,
                packet_id,
            },
            packet_id,
        )
    }
}
