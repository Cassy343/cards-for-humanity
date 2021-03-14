mod packs;
mod server;

use warp::{Filter, ws::{Ws, WebSocket, Message}};
use futures::{SinkExt, StreamExt, channel::{oneshot::{self, Sender}, mpsc}};
use zip::ZipArchive;
use std::{io::{Cursor, copy, Error as IoError}, sync::atomic::AtomicUsize};
use std::fs::{File, create_dir_all};
use std::path::Path;
use std::net::SocketAddr;
use std::sync::{Arc, atomic::Ordering};
use server::{WsServerHandler, WsServerHandlerInner};
use tokio::sync::Mutex;

const CLIENT_FILES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/client.zip"));

#[tokio::main]
async fn main() {
    match unpack_client_files() {
        Err(e) => {
            println!("Failed to unpack client files: {}", e);
            return;
        }

        _ => {}
    }

    let ws_server_handler = Arc::new(Mutex::new(WsServerHandlerInner::new()));
    let shutdown_hook = start_server(ws_server_handler.clone()).await;
    
    loop {
        ws_server_handler.lock().await.broadcast(Message::text("Hello, world!")).await;
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}

// TODO: Implement checksum system
fn unpack_client_files() -> Result<(), IoError> {
    let mut archive = ZipArchive::new(Cursor::new(CLIENT_FILES)).expect("Client files corrupted.");

    for i in 0..archive.len() {
        let mut source_file = archive.by_index(i)?;
        let name = Path::new(source_file.name()).to_owned();

        if source_file.is_dir() {
            create_dir_all(&name)?;
        } else {
            copy(&mut source_file, &mut File::create(&name)?)?;
        }
    }

    Ok(())
}

async fn start_server(ws_server_handler: WsServerHandler) -> Sender<()> {
    let ws_server = warp::path("ws")
        .and(warp::ws())
        .and(warp::addr::remote())
        .and(warp::any().map(move || ws_server_handler.clone()))
        .map(|ws: Ws, address: Option<SocketAddr>, ws_server_handler: WsServerHandler| {
            ws.on_upgrade(move |socket| handle_socket(socket, address, ws_server_handler))
        });
    let index = warp::path::end().and(warp::fs::file("www/index.html"));
    let www = warp::fs::dir("www/").or(index);

    let (shutdown_hook, rx) = oneshot::channel::<()>();

    let (_addr, server) = warp::serve(www.or(ws_server)).bind_with_graceful_shutdown(
        ([127, 0, 0, 1], 8080),
        async {
            rx.await.ok();
        }
    );

    tokio::task::spawn(server);
    shutdown_hook
}

async fn handle_socket(socket: WebSocket, address: Option<SocketAddr>, ws_server_handler: WsServerHandler) {
    let (ws_tx, mut ws_rx) = socket.split();
    let (tx, rx) = mpsc::unbounded::<Message>();

    tokio::task::spawn(rx.map(|message| Ok(message)).forward(ws_tx));

    let mut handler_guard = ws_server_handler.lock().await;
    let id = handler_guard.add_client(tx, address);
    let mut pipe = handler_guard.message_pipe();
    drop(handler_guard);

    println!("New client connected (ID {})", id);

    while let Some(result) = ws_rx.next().await {
        let message = match result {
            Ok(message) => message,
            Err(e) => {
                eprintln!("WS error {}", e);
                break;
            }
        };

        if let Err(e) = pipe.send((id, message)).await {
            eprintln!("Failed to pipe WS message to handler, {}", e);
        }
    }

    ws_server_handler.lock().await.remove_client(id);
    println!("Client disconnected (ID {})", id);
}
