use anyhow::{anyhow, Context};
use std::{
    collections::HashMap,
    convert::AsRef,
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
    sync::Arc,
};

use super::card::{Pack, PackMeta};

const DEFAULT_PACK: &str = "CAH Base Set";
const DEFAULT_PACK_JSON: &str = "CAH Base Set.json";

/// A store to manage loading and unloading [Packs](Pack)
pub struct PackStore {
    pack_dir: PathBuf,
    loaded_packs: HashMap<String, Arc<Pack>>,
    possible_packs: HashMap<String, PackMeta>,
}

impl PackStore {
    /// Creates a new PackStore
    pub fn new<P: AsRef<Path>>(pack_dir: P) -> std::io::Result<Self> {
        let mut pack_store = Self {
            pack_dir: pack_dir.as_ref().to_owned(),
            loaded_packs: HashMap::new(),
            possible_packs: HashMap::new(),
        };
        pack_store.init()?;
        Ok(pack_store)
    }

    fn init(&mut self) -> io::Result<()> {
        let official_packs = fs::read_dir(&self.official_dir())?
            .filter_map(|e| {
                e.ok().map(|name| {
                    let pack = Self::read_pack(&name.path()).unwrap();
                    (name.file_name().to_str().unwrap().to_owned(), pack.meta())
                })
            })
            .collect::<HashMap<_, _>>();

        let custom_packs = fs::read_dir(&self.custom_dir())?
            .filter_map(|e| {
                e.ok().map(|name| {
                    // This runs at startup I'm not handeling the unwrap
                    let pack = Self::read_pack(&name.path()).unwrap();
                    (
                        // This unwrap should not fail
                        name.file_name().to_str().unwrap().to_owned(),
                        pack.meta(),
                    )
                })
            })
            .collect::<HashMap<_, _>>();

        let mut possible_packs = HashMap::new();

        possible_packs.extend(official_packs);
        possible_packs.extend(custom_packs);

        self.possible_packs = possible_packs;

        self.load_pack(DEFAULT_PACK)
            .map_err(|_| io::Error::new(ErrorKind::NotFound, "Default pack file not found."))?;

        Ok(())
    }

    pub fn default_pack(&self) -> Arc<Pack> {
        self.loaded_packs.get(DEFAULT_PACK_JSON).unwrap().clone()
    }

    /// Loads in a pack from json
    pub fn load_pack(&mut self, pack_name: &str) -> anyhow::Result<Arc<Pack>> {
        let pack_name = &format!("{}.json", pack_name);

        if let Some(pack) = self.loaded_packs.get(pack_name) {
            return Ok(Arc::clone(pack));
        }

        if let Some(PackMeta { official, .. }) = self.possible_packs.get(pack_name) {
            let pack = if *official {
                Self::read_pack(&self.official_dir().join(pack_name))?
            } else {
                Self::read_pack(&self.custom_dir().join(pack_name))?
            };

            let pack = Arc::new(pack);
            self.loaded_packs
                .insert(pack_name.to_owned(), Arc::clone(&pack));

            Ok(pack)
        } else {
            Err(anyhow!("Pack {} is not found", pack_name))
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

    pub fn create_pack(&mut self, pack: Pack) -> anyhow::Result<()> {
        let pack_name = format!("{}.json", pack.name.clone());

        let json = serde_json::to_string(&pack).context("Error serializing pack")?;
        fs::write(self.custom_dir().join(&pack_name), json).context("Error saving pack")?;
        let mut meta = pack.meta();
        meta.official = false;
        self.possible_packs.insert(pack_name, meta);
        Ok(())
    }

    pub fn possible_packs(&self) -> &HashMap<String, PackMeta> {
        &self.possible_packs
    }

    fn read_pack(path: &Path) -> anyhow::Result<Pack> {
        let json = fs::read_to_string(path).context("Error reading pack file")?;
        serde_json::from_str::<Pack>(&json)
            .context("Error deserializing pack")
            .map_err(Into::into)
    }

    fn official_dir(&self) -> PathBuf {
        self.pack_dir.join("official")
    }

    fn custom_dir(&self) -> PathBuf {
        self.pack_dir.join("custom")
    }
}
