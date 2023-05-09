use futures::stream::StreamExt;
use serde::Deserialize;
use ts_rs::TS;
use warp::ws::{Message, WebSocket};

pub async fn handle_socket(mut socket: WebSocket) {
    let mut client = Client::new();

    loop {
        while let Some(result) = socket.next().await {
            match result {
                Ok(message) => {
                    let text = match message.to_str() {
                        Ok(text) => text,
                        Err(_) => {
                            todo!("We didn't receive text");
                        }
                    };

                    let message: ExternalServerbound = match serde_json::from_str(text) {
                        Ok(msg) => msg,
                        Err(error) => todo!("{error}"),
                    };

                    client.handle_message(message);
                }
                Err(error) => {
                    todo!("{error}")
                }
            }
        }
    }
}

struct Client {
    username: Option<String>,
}

impl Client {
    fn new() -> Self {
        Self { username: None }
    }

    fn handle_message(&mut self, message: ExternalServerbound) {
        match message {
            ExternalServerbound::SetUsername { username } => {
                println!("Set username to {username}");
                self.username = Some(username);
            }
        }
    }
}

#[derive(Deserialize, TS)]
#[ts(export)]
enum ExternalServerbound {
    SetUsername { username: String },
}
