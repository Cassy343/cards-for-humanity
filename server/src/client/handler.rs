use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use warp::ws::{Message, WebSocket};

use crate::{
    chan::{channel, Tx},
    game::{GameHandle, JoinGame, JoinResponse, PlayerGameId},
    lobby::{CreateGame, GetGameHandle, LobbyMessage},
};
use tokio::task;

use super::{
    ClientMessage,
    External,
    ExternalClientMessage,
    ExternalServerMessage,
    PlayerListUpdate,
    WsDisconnected,
};

type WsSink = SplitSink<WebSocket, Message>;

pub async fn handle_socket(socket: WebSocket, lobby: Tx<LobbyMessage>) {
    let (tx, mut rx) = channel::<ClientMessage>();
    let (ws_sink, ws_stream) = socket.split();
    task::spawn(forward(ws_stream, tx.clone()));

    let mut client = Client::new(tx, ws_sink, lobby);

    while let Some(message) = rx.recv().await {
        client.handle_message(message).await;
    }
}

async fn forward(mut stream: SplitStream<WebSocket>, sink: Tx<ClientMessage>) {
    while let Some(result) = stream.next().await {
        match result {
            Ok(message) => {
                let text = match message.to_str() {
                    Ok(text) => text,
                    Err(_) => {
                        todo!("We didn't receive text");
                    }
                };

                let message: ExternalServerMessage = match serde_json::from_str(text) {
                    Ok(msg) => msg,
                    Err(error) => todo!("{error}"),
                };

                sink.send(External(message)).await;
            }
            Err(error) => {
                todo!("{error}")
            }
        }
    }

    sink.send(WsDisconnected).await;
}

struct Client {
    tx: Tx<ClientMessage>,
    ws: WsSink,
    lobby: Tx<LobbyMessage>,
    username: Option<String>,
    current_game: Option<PlayerGameHandle>,
}

impl Client {
    fn new(tx: Tx<ClientMessage>, ws: WsSink, lobby: Tx<LobbyMessage>) -> Self {
        Self {
            tx,
            ws,
            lobby,
            username: None,
            current_game: None,
        }
    }

    async fn send_external(&mut self, message: ExternalClientMessage) {
        let text =
            serde_json::to_string(&message).expect("Failed to convert external message to json");
        if let Err(error) = self.ws.send(Message::text(text)).await {
            todo!("Failed to send WS message: {error}");
        }
    }

    async fn handle_message(&mut self, message: ClientMessage) {
        match message {
            ClientMessage::External(External(message)) => {
                if let Some(response) = self.handle_external_message(message).await {
                    self.send_external(response).await;
                }
            }
            ClientMessage::WsDisconnected(_) => self.handle_ws_disconnected(),
            ClientMessage::PlayerListUpdate(update) => self.handle_player_list_update(update).await,
        }
    }

    fn handle_ws_disconnected(&self) {
        todo!()
    }

    async fn handle_player_list_update(&mut self, update: PlayerListUpdate) {
        self.send_external(ExternalClientMessage::PlayerListUpdate { payload: update })
            .await;
    }

    async fn handle_external_message(
        &mut self,
        message: ExternalServerMessage,
    ) -> Option<ExternalClientMessage> {
        match message {
            ExternalServerMessage::SetUsername { username } => self.handle_set_username(username),
            ExternalServerMessage::CreateGame => self.handle_create_game().await,
            ExternalServerMessage::JoinGame { id } => self.handle_join_game(id).await,
        }
    }

    fn handle_set_username(&mut self, username: String) -> Option<ExternalClientMessage> {
        println!("Set username to {username}");
        self.username = Some(username);
        None
    }

    async fn handle_create_game(&mut self) -> Option<ExternalClientMessage> {
        let username = self.username.as_ref().cloned()?;

        let handle = self
            .lobby
            .send(CreateGame {
                client: self.tx.clone(),
                username,
            })
            .await;
        let id = handle.creator_id;
        self.current_game = Some(PlayerGameHandle { handle, id });

        Some(ExternalClientMessage::JoinResponse {
            response: Some(JoinResponse::JoinAsPlayer { id }),
        })
    }

    async fn handle_join_game(&mut self, id: u16) -> Option<ExternalClientMessage> {
        let username = self.username.as_ref().cloned()?;

        let handle = match self.lobby.send(GetGameHandle { id }).await {
            Some(handle) => handle,
            None => return Some(ExternalClientMessage::JoinResponse { response: None }),
        };

        let response = handle
            .game
            .send(JoinGame {
                client: self.tx.clone(),
                username,
            })
            .await;

        if let &JoinResponse::JoinAsPlayer { id } = &response {
            self.current_game = Some(PlayerGameHandle { handle, id });
        }

        Some(ExternalClientMessage::JoinResponse {
            response: Some(response),
        })
    }
}

struct PlayerGameHandle {
    handle: GameHandle,
    id: PlayerGameId,
}
