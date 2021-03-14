use std::{collections::HashMap, fs, path::Path, rc::{Rc, Weak}};

use common::{Pack, Prompt, Response};
use rand::Rng;

/// A store to manage loading and unloading [Packs](Pack)
pub struct PackStore {
    pack_dir: String,
    loaded_packs: HashMap<String, Rc<Pack>>,
    official_packs: Vec<String>,
    possible_packs: Vec<String>,
}

impl PackStore {
    /// Creates a new PackStore
    pub fn new(pack_dir: &str) -> std::io::Result<Self> {
        let official_packs = fs::read_dir(&format!("{}/official/", pack_dir))?
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
        let custom_packs = fs::read_dir(&format!("{}/custom/", pack_dir))?
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
            official_packs
        })
    }

    /// Loads in a pack from json
    pub fn load_pack(&mut self, pack_name: &str) -> Result<PackHandle, String> {
        let pack_name = &format!("{}.json", pack_name);
        if self.loaded_packs.contains_key(pack_name) {
            Ok(PackHandle {
                pack: self.loaded_packs.get(pack_name).unwrap().clone(),
                used_prompts: Vec::new(),
                used_responses: Vec::new(),
            })
        } else if self.possible_packs.contains(&pack_name.to_owned()) {
            let pack = if self.official_packs.contains(&pack_name.to_owned()) {
                Self::read_pack(Path::new(&format!("{}/official/{}", self.pack_dir, pack_name)))?
            } else {
                Self::read_pack(Path::new(&format!("{}/custom/{}", self.pack_dir, pack_name)))?
            };

            self.loaded_packs
                .insert(pack_name.to_owned(), Rc::new(pack));

            Ok(PackHandle {
                pack: self.loaded_packs.get(pack_name).unwrap().clone(),
                used_prompts: Vec::new(),
                used_responses: Vec::new(),
            })
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

    fn read_pack(path: &Path) -> Result<Pack, String> {
        let json = match fs::read_to_string(path) {
            Ok(f) => f,
            Err(e) => return Err(format!("Error reading pack file: {}", e)),
        };

        serde_json::from_str::<Pack>(&json).map_err(|e| format!("Error deserializing pack: {}", e))
    }
}

pub struct PackHandle {
    pack: Rc<Pack>,
    used_prompts: Vec<usize>,
    used_responses: Vec<usize>,
}

impl PackHandle {
    pub fn pack(&self) -> Weak<Pack> {
        Rc::downgrade(&self.pack)
    }

    /// Get a random prompt from the pack
    pub fn get_prompt(&mut self) -> &Prompt {
        let mut rng = rand::thread_rng();

        let mut index: usize;

        loop {
            index = rng.gen_range(0 .. self.pack.prompts.len());

            if !self.used_prompts.contains(&index) {
                break
            }
        }

        // Unwrap is safe cause we only have indexes in bounds
        self.used_prompts.push(index);
        self.pack.prompts.get(index).unwrap()
    }

    /// Get a random response from the pack
    pub fn get_response(&mut self) -> &Response {
        let mut rng = rand::thread_rng();

        let mut index: usize;

        loop {
            index = rng.gen_range(0 .. self.pack.responses.len());

            if !self.used_responses.contains(&index) {
                break
            }
        }

        // Unwrap is safe cause we only have indexes in bounds
        self.used_responses.push(index);
        self.pack.responses.get(index).unwrap()
    }
}
