mod game;
mod logging;
mod network;

use futures::channel::oneshot::{self, Sender};
use linefeed::{Interface, ReadResult};
use log::error;
use network::{client::ClientHandler, NetworkHandler};
use std::{
    error::Error,
    fs::{create_dir_all, File},
    io::{copy, Cursor, Error as IoError},
    net::SocketAddr,
    path::Path,
    sync::Arc,
    time::Duration,
};
use tokio::sync::Mutex;
use warp::{ws::Ws, Filter};
use zip::ZipArchive;

const CLIENT_FILES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/client.zip"));

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let console_interface = Arc::new(Interface::new("cfh")?);
    console_interface.set_prompt("> ")?;
    logging::init_logger(console_interface.clone())?;

    match unpack_client_files() {
        Err(e) => {
            error!("Failed to unpack client files: {}", e);
            return Ok(());
        }

        _ => {}
    }

    let (raw_ch, incoming_messages) = ClientHandler::new();
    let client_handler = Arc::new(Mutex::new(raw_ch));
    let server_shutdown_hook = start_server(client_handler.clone()).await;
    let mut network_handler =
        NetworkHandler::new(client_handler, incoming_messages, server_shutdown_hook);
    network_handler.add_listener(game::Game::new());

    loop {
        // Check for a new command every 50ms
        match console_interface.read_line_step(Some(Duration::from_millis(50))) {
            Ok(result) => match result {
                Some(ReadResult::Input(command)) => {
                    console_interface.add_history_unique(command.clone());

                    if command.to_ascii_lowercase() == "stop" {
                        break;
                    }

                    // TODO: handle other commands
                }
                _ => {}
            },
            Err(e) => error!("Failed to read console input: {}", e),
        }

        network_handler.handle_messages();
    }

    network_handler.shutdown().await;

    // Move off of the command prompt
    logging::cleanup();
    println!();

    Ok(())
}

// TODO: Implement checksum system
fn unpack_client_files() -> Result<(), IoError> {
    let mut archive = ZipArchive::new(Cursor::new(CLIENT_FILES)).expect("Client files corrupted.");

    for i in 0 .. archive.len() {
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

async fn start_server(client_handler: Arc<Mutex<ClientHandler>>) -> Sender<()> {
    let ws_server = warp::path("ws")
        .and(warp::ws())
        .and(warp::addr::remote())
        .and(warp::any().map(move || client_handler.clone()))
        .map(
            |ws: Ws, address: Option<SocketAddr>, client_handler: Arc<Mutex<ClientHandler>>| {
                ws.on_upgrade(move |socket| {
                    ClientHandler::handle_socket(socket, address, client_handler)
                })
            },
        );
    let index = warp::path::end().and(warp::fs::file("www/index.html"));
    let www = warp::fs::dir("www/").or(index);

    let (shutdown_hook, rx) = oneshot::channel::<()>();

    let (_addr, server) =
        warp::serve(www.or(ws_server)).bind_with_graceful_shutdown(([127, 0, 0, 1], 8080), async {
            rx.await.ok();
        });

    tokio::task::spawn(server);
    shutdown_hook
}
