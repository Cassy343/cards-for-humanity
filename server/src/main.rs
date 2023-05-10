mod chan;
mod client;
mod data;
mod game;
mod lobby;

use anyhow;
use client::handle_socket;
use lobby::open_lobby;
use warp::{ws::Ws, Filter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let lobby = open_lobby();

    let index = warp::any().and(warp::fs::file("public/index.html"));
    let public = warp::fs::dir("public/");
    let ws = warp::path("ws")
        .and(warp::ws())
        .and(warp::any().map(move || lobby.clone()))
        .map(|ws: Ws, lobby| ws.on_upgrade(move |socket| handle_socket(socket, lobby)));

    // TODO: configure port
    warp::serve(ws.or(public).or(index))
        .bind(([0, 0, 0, 0], 25565))
        .await;

    Ok(())
}
