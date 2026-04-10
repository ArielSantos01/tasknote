use std::fs;
use std::path::PathBuf;

use crate::item::Item;

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct Store {
    pub items: Vec<Item>,
    pub next_id: u32,
    #[serde(default)]
    pub default_board: Option<String>,
}

impl Store {
    pub fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

fn storage_path() -> PathBuf {
    let base = dirs::home_dir().expect("Cannot find home directory");
    base.join(".tasknote").join("storage.json")
}

pub fn load() -> Store {
    let path = storage_path();
    if !path.exists() {
        return Store {
            items: vec![],
            next_id: 1,
            default_board: None,
        };
    }
    let data = fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&data).unwrap_or(Store {
        items: vec![],
        next_id: 1,
        default_board: None,
    })
}

pub fn save(store: &Store) {
    let path = storage_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Cannot create storage directory");
    }
    let data = serde_json::to_string_pretty(store).expect("Serialization failed");
    fs::write(&path, data).expect("Cannot write storage file");
}

pub fn active_items(store: &Store) -> Vec<&Item> {
    store.items.iter().filter(|i| !i.archived).collect()
}

pub fn archived_items(store: &Store) -> Vec<&Item> {
    store.items.iter().filter(|i| i.archived).collect()
}

pub fn find_item_mut(store: &mut Store, id: u32) -> Option<&mut Item> {
    store.items.iter_mut().find(|i| i.id == id)
}
