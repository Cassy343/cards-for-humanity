mod chan;
mod client;
mod data;
mod lobby;

use anyhow;
use client::handle_socket;
use lobby::open_lobby;
use warp::{ws::Ws, Filter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let lobby_sender = open_lobby();

    let index = warp::any().and(warp::fs::file("public/index.html"));
    let public = warp::fs::dir("public/");
    let ws = warp::path("ws")
        .and(warp::ws())
        .map(|ws: Ws| ws.on_upgrade(handle_socket));

    warp::serve(ws.or(public).or(index))
        .bind(([0, 0, 0, 0], 25565))
        .await;

    Ok(())
}
