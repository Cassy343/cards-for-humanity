use warp::{Filter, ws::{Ws, WebSocket}};
use futures::{StreamExt, channel::oneshot::{self, Sender}};
use zip::ZipArchive;
use std::io::{Cursor, copy, Error as IoError};
use std::fs::{File, create_dir_all};
use std::path::Path;

mod packs;

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

    let shutdown_hook = start_server().await;
    loop {}
}

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

async fn start_server() -> Sender<()> {
    let ws_server = warp::path("ws")
        .and(warp::ws())
        .map(|ws: Ws| {
            ws.on_upgrade(handle_socket)
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

async fn handle_socket(socket: WebSocket) {
    let (tx, mut rx) = socket.split();

    while let Some(result) = rx.next().await {
        let message = match result {
            Ok(message) => message,
            Err(e) => {
                eprintln!("WS error {}", e);
                break;
            }
        };

        println!("{:?}", message);
    }
}
