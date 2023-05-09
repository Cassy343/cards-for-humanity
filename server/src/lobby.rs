use tokio::{
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    task,
};

pub fn open_lobby() -> UnboundedSender<LobbyRequest> {
    let (tx, rx) = mpsc::unbounded_channel();
    task::spawn(handle_lobby(rx));
    tx
}

async fn handle_lobby(mut rx: UnboundedReceiver<LobbyRequest>) {
    while let Some(request) = rx.recv().await {
        match request {}
    }
}

pub enum LobbyRequest {}
