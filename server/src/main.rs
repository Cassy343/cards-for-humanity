mod data;

use std::{
    fs::{create_dir_all, File},
    io::{copy, Cursor, Error as IoError},
    path::Path,
};

use anyhow;
use warp::{ws::Ws, Filter};
use zip::ZipArchive;

const CLIENT_FILES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/client.zip"));

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match unpack_client_files() {
        Err(e) => {
            println!("Failed to unpack client files: {}", e);
            return Ok(());
        }

        _ => {}
    }

    let index = warp::path::end().and(warp::fs::file("public/index.html"));
    let public = warp::fs::dir("public/").or(index);
    let ws = warp::path("ws").and(warp::ws()).map(|ws: Ws| {
        ws.on_upgrade(move |socket| async {
            println!("Got WS connection");
        })
    });

    warp::serve(public.or(ws)).bind(([0, 0, 0, 0], 25565)).await;

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
