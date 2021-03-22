use std::{collections::HashMap, convert::AsRef, fs, path::{Path, PathBuf}, rc::{Rc, Weak}};
use common::data::cards::{Pack, Prompt, Response};
use rand::Rng;

/// A store to manage loading and unloading [Packs](Pack)
pub struct PackStore {
    pack_dir: PathBuf,
    loaded_packs: HashMap<String, Rc<Pack>>,
    official_packs: Vec<String>,
    possible_packs: Vec<String>,
}

impl PackStore {
    /// Creates a new PackStore
    pub fn new<P: AsRef<Path>>(pack_dir: P) -> std::io::Result<Self> {
        let pack_dir = pack_dir.as_ref();

        let official_packs = fs::read_dir(&pack_dir.join("official"))?
            .filter_map(|e| {
                if e.is_ok() {
                    if let Some(name) = e.unwrap().file_name().to_str() {
                        Some(name.to_owned())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let custom_packs = fs::read_dir(&pack_dir.join("custom"))?
            .filter_map(|e| {
                if e.is_ok() {
                    if let Some(name) = e.unwrap().file_name().to_str() {
                        Some(name.to_owned())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let mut possible_packs = Vec::new();

        possible_packs.extend(official_packs.clone());
        possible_packs.extend(custom_packs);

        Ok(PackStore {
            possible_packs,
            pack_dir: pack_dir.to_owned(),
            loaded_packs: HashMap::new(),
            official_packs,
        })
    }

    /// Loads in a pack from json
    pub fn load_pack(&mut self, pack_name: &str) -> Result<Rc<Pack>, String> {
        let pack_name = &format!("{}.json", pack_name);
        if self.loaded_packs.contains_key(pack_name) {
            Ok(self.loaded_packs.get(pack_name).unwrap().clone())
        } else if self.possible_packs.contains(pack_name) {
            let pack = if self.official_packs.contains(&pack_name.to_owned()) {
                Self::read_pack(&self.official_dir().join(pack_name))?
            } else {
                Self::read_pack(&self.custom_dir().join(pack_name))?
            };

            self.loaded_packs
                .insert(pack_name.to_owned(), Rc::new(pack));

            Ok(self.loaded_packs.get(pack_name).unwrap().clone())
        } else {
            Err(format!("Pack {} is not found", pack_name))
        }
    }

    /// Unloads a pack if no games are using it
    pub fn unload_pack(&mut self, pack_name: &str) {
        let pack_name = &format!("{}.json", pack_name);
        if self.loaded_packs.contains_key(pack_name) {
            let pack = self.loaded_packs.get(pack_name).unwrap();

            // If the PackStore is the only owned Rc left, unload the pack
            if Rc::strong_count(pack) == 1 {
                self.loaded_packs.remove(pack_name);
            }
        }
    }

    pub fn create_pack(&mut self, pack: Pack) -> Result<(), String> {
        let pack_name = pack.name.clone();

        let json = match serde_json::to_string(&pack) {
            Ok(j) => j,
            Err(e) => return Err(format!("Error serializing pack: {}", e)),
        };

        match fs::write(
            self.custom_dir().join(&pack_name),
            json,
        ) {
            Ok(_) => {
                self.possible_packs.push(format!("{}.json", pack_name));
                Ok(())
            }
            Err(e) => Err(format!("Error writing to file: {}", e)),
        }
    }

    fn read_pack(path: &Path) -> Result<Pack, String> {
        let json = match fs::read_to_string(path) {
            Ok(f) => f,
            Err(e) => return Err(format!("Error reading pack file: {}", e)),
        };

        serde_json::from_str::<Pack>(&json).map_err(|e| format!("Error deserializing pack: {}", e))
    }

    fn official_dir(&self) -> PathBuf {
        self.pack_dir.join("official")
    }

    fn custom_dir(&self) -> PathBuf {
        self.pack_dir.join("custom")
    }
}
