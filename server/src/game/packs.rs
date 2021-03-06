use common::data::cards::{Pack, Prompt, Response};
use rand::Rng;
use std::{
    collections::HashMap,
    convert::AsRef,
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
    rc::{Rc, Weak},
    sync::Arc,
};

const DEFAULT_PACK: &str = "CAH Base Set";
const DEFAULT_PACK_JSON: &str = "CAH Base Set.json";

/// A store to manage loading and unloading [Packs](Pack)
pub struct PackStore {
    pack_dir: PathBuf,
    loaded_packs: HashMap<String, Arc<Pack>>,
    // bool is if the pack is official
    // usizes are prompts and responses respectively
    possible_packs: HashMap<String, (bool, usize, usize)>,
}

impl PackStore {
    /// Creates a new PackStore
    pub fn new<P: AsRef<Path>>(pack_dir: P) -> std::io::Result<Self> {
        let pack_dir = pack_dir.as_ref();

        let official_packs = fs::read_dir(&pack_dir.join("official"))?
            .filter_map(|e| {
                if e.is_ok() {
                    if let Ok(name) = e {
                        let pack = Self::read_pack(&name.path()).unwrap();
                        Some((
                            name.file_name().to_str().unwrap().to_owned(),
                            (pack.official, pack.prompts.len(), pack.responses.len()),
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>();

        let custom_packs = fs::read_dir(&pack_dir.join("custom"))?
            .filter_map(|e| {
                if e.is_ok() {
                    if let Ok(name) = e {
                        // This runs at startup I'm not handeling the unwrap
                        let pack = Self::read_pack(&name.path()).unwrap();
                        Some((
                            // This unwrap should not fail
                            name.file_name().to_str().unwrap().to_owned(),
                            (pack.official, pack.prompts.len(), pack.responses.len()),
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>();

        let mut possible_packs = HashMap::new();

        possible_packs.extend(official_packs.clone());
        possible_packs.extend(custom_packs);

        let mut pack_store = PackStore {
            possible_packs,
            pack_dir: pack_dir.to_owned(),
            loaded_packs: HashMap::new(),
        };

        pack_store
            .load_pack(DEFAULT_PACK)
            .map_err(|_| io::Error::new(ErrorKind::NotFound, "Default pack file not found."))?;
        Ok(pack_store)
    }

    pub fn default_pack(&self) -> Arc<Pack> {
        self.loaded_packs.get(DEFAULT_PACK_JSON).unwrap().clone()
    }

    /// Loads in a pack from json
    pub fn load_pack(&mut self, pack_name: &str) -> Result<Arc<Pack>, String> {
        let pack_name = &format!("{}.json", pack_name);
        if self.loaded_packs.contains_key(pack_name) {
            Ok(self.loaded_packs.get(pack_name).unwrap().clone())
        } else if let Some((official, ..)) = self.possible_packs.get(pack_name) {
            let pack = if *official {
                Self::read_pack(&self.official_dir().join(pack_name))?
            } else {
                Self::read_pack(&self.custom_dir().join(pack_name))?
            };

            self.loaded_packs
                .insert(pack_name.to_owned(), Arc::new(pack));

            Ok(self.loaded_packs.get(pack_name).unwrap().clone())
        } else {
            Err(format!("Pack {} is not found", pack_name))
        }
    }

    /// Unloads a pack if no games are using it
    pub fn unload_pack(&mut self, pack_name: &str) {
        if pack_name == DEFAULT_PACK {
            return;
        }

        let pack_name = &format!("{}.json", pack_name);
        if self.loaded_packs.contains_key(pack_name) {
            let pack = self.loaded_packs.get(pack_name).unwrap();

            // If the PackStore is the only owned Rc left, unload the pack
            if Arc::strong_count(pack) == 1 {
                self.loaded_packs.remove(pack_name);
            }
        }
    }

    pub fn create_pack(&mut self, pack: Pack) -> Result<(), String> {
        let pack_name = format!("{}.json", pack.name.clone());

        let json = match serde_json::to_string(&pack) {
            Ok(j) => j,
            Err(e) => return Err(format!("Error serializing pack: {}", e)),
        };

        match fs::write(self.custom_dir().join(&pack_name), json) {
            Ok(_) => {
                self.possible_packs
                    .insert(pack_name, (false, pack.prompts.len(), pack.responses.len()));
                Ok(())
            }
            Err(e) => Err(format!("Error writing to file: {}", e)),
        }
    }

    pub fn get_packs_meta(&self) -> Vec<(String, usize, usize)> {
        self.possible_packs
            .iter()
            .map(|(name, (_, prompts, responses))| {
                (name.replace(".json", ""), *prompts, *responses)
            })
            .collect()
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
