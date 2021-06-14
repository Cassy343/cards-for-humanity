use crate::data::cards::CardID;

use super::{GameSetting, GameSettings};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerBoundPacket {
    // Game packets
    SetPlayerName(String),
    StartGame,
    UpdateSetting(GameSetting),
    SelectResponse(CardID),
    SelectRoundWinner(Uuid),

    // Lobby packets
    CreateServer(GameSettings),
    JoinGame(Uuid),
    RefreshServerList,
    RequestCardPacks,
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
