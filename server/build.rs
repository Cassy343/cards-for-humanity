use std::{env, fs::File, io::copy, path::Path, process::Command};
use walkdir::WalkDir;
use zip::ZipWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("client.zip");
    let dest = File::create(dest_path)?;
    let mut zip_writer = ZipWriter::new(dest);

    add_directory_recursively(&mut zip_writer, "www")?;
    add_directory_recursively(&mut zip_writer, "packs")?;

    zip_writer.add_directory("www/client", Default::default())?;
    copy_client_file(&mut zip_writer, "client.js")?;
    copy_client_file(&mut zip_writer, "client_bg.wasm")?;

    zip_writer.finish()?;

    println!("cargo:rerun-if-changed=../target/client-out/");
    println!("cargo:rerun-if-changed=../client/");
    println!("cargo:rerun-if-changed=./www/");
    Ok(())
}

fn add_directory_recursively(
    zip_writer: &mut ZipWriter<File>,
    root: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in WalkDir::new(root)
        .into_iter()
        .flat_map(|entry| entry.ok())
        .filter(|entry| !entry.path_is_symlink())
    {
        let path = entry.path();
        let path_string = path.to_str().unwrap();
        if path.is_dir() {
            zip_writer.add_directory(path_string, Default::default())?;
        } else {
            zip_writer.start_file(path_string, Default::default())?;
            copy(&mut File::open(path)?, zip_writer)?;
        }
    }

    Ok(())
}

fn copy_client_file(
    zip_writer: &mut ZipWriter<File>,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    zip_writer.start_file(&format!("www/client/{}", name), Default::default())?;
    copy(
        &mut File::open(&format!("../target/client-out/{}", name))?,
        zip_writer,
    )?;
    Ok(())
}
