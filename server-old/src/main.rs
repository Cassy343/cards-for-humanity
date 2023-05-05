mod game;
mod logging;
mod network;

use common::data::cards::Pack;
use futures::{
    channel::oneshot::{self, Sender},
    TryFutureExt,
};
use game::packs::PackStore;
use linefeed::{Interface, ReadResult};
use log::error;
use network::{client::ClientHandler, NetworkHandler};
use once_cell::sync::OnceCell;
use std::{
    cell::RefCell,
    error::Error,
    fs::{create_dir_all, File},
    io::{copy, Cursor, Error as IoError},
    net::SocketAddr,
    path::Path,
    rc::Rc,
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::sync::{
    mpsc::{self, channel},
    Mutex,
};
use uuid::Uuid;
use warp::{ws::Ws, Filter};
use zip::ZipArchive;

const CLIENT_FILES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/client.zip"));
static LOBBY_ID: OnceCell<Uuid> = OnceCell::new();

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

    // Create a channel to upload packs through
    // Since actually uploading a pack is rare a buffer of 10 is generous
    let (pack_sender, pack_reciever) = channel::<Pack>(10);

    let pack_store = match PackStore::new("./packs") {
        Ok(pack_store) => Arc::new(std::sync::RwLock::new(pack_store)),
        Err(e) => {
            error!("Failed to create card pack manager: {}", e);
            return Ok(());
        }
    };

    let (raw_ch, incoming_messages) = ClientHandler::new();
    let client_handler = Arc::new(Mutex::new(raw_ch));
    let server_shutdown_hook = start_server(client_handler.clone(), pack_store.clone()).await;
    let mut network_handler =
        NetworkHandler::new(client_handler, incoming_messages, server_shutdown_hook);
    let lobby_id = network_handler.add_listener(game::Lobby::new(pack_store.clone()));
    LOBBY_ID.set(lobby_id).expect("Error setting LOBBY_ID");

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

        network_handler.handle_messages().await;
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

async fn start_server(
    client_handler: Arc<Mutex<ClientHandler>>,
    pack_store: Arc<std::sync::RwLock<PackStore>>,
) -> Sender<()> {
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
    let upload_store = pack_store.clone();
    let upload = warp::path!("upload")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || upload_store.clone()))
        .map(|pack: Pack, store: Arc<RwLock<PackStore>>| {
            match store.clone().write().unwrap().create_pack(pack) {
                Ok(_) => "".to_owned(),
                Err(e) => format!("Error uploading pack: {}", e),
            }
        });
    let search_store = pack_store.clone();
    let search_pack = warp::path!("packs" / String)
        .and(warp::any().map(move || search_store.clone()))
        .map(|pack_name: String, store: Arc<RwLock<PackStore>>| {
            let pack = match store.write().unwrap().load_pack(&decode_uri(&pack_name)) {
                Ok(p) => p,
                Err(_) => return format!("Pack {} does not exist", pack_name),
            };
            let string = serde_json::to_string::<Pack>(&pack).unwrap();
            // Make sure we don't keep the pack loaded
            drop(pack);
            store.write().unwrap().unload_pack(&pack_name);
            string
        });
    let list_store = pack_store;
    let list = warp::path!("packs").map(move || {
        serde_json::to_string(
            &list_store
                .read()
                .unwrap()
                .get_packs_meta()
                .iter()
                .map(|i| i.0.clone())
                .collect::<Vec<_>>(),
        )
        .unwrap()
    });

    let (shutdown_hook, rx) = oneshot::channel::<()>();

    let (_addr, server) = warp::serve(www.or(upload).or(ws_server).or(search_pack).or(list))
        .bind_with_graceful_shutdown(([0, 0, 0, 0], 25565), async {
            rx.await.ok();
        });

    tokio::task::spawn(server);
    shutdown_hook
}


fn decode_uri(str: &str) -> String {
    str.replace("%20", " ")
        .replace("%22", "\"")
        .replace("%3C", "<")
        .replace("%3E", ">")
        .replace("%23", "#")
        .replace("%25", "%")
        .replace("%7B", "{")
        .replace("%7D", "}")
        .replace("%7C", "|")
        .replace("%5C", "\\")
        .replace("%5E", "^")
        .replace("%7E", "~")
        .replace("%5B", "[")
        .replace("%5D", "]")
        .replace("%60", "`")
}
