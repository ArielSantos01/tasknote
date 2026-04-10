use crate::item::{Item, ItemType, Status};
use std::collections::BTreeMap;

pub struct BoardStats {
    pub name: String,
    pub done: usize,
    pub total: usize,
    pub items: Vec<Item>,
}

pub fn group_tasks_by_board(items: &[&Item]) -> Vec<BoardStats> {
    let mut map: BTreeMap<String, Vec<Item>> = BTreeMap::new();

    for item in items {
        if item.item_type == ItemType::Task {
            map.entry(item.board.clone())
                .or_default()
                .push((*item).clone());
        }
    }

    map.into_iter()
        .map(|(name, items)| {
            let done = items.iter().filter(|i| i.status == Status::Done).count();
            let total = items.len();
            BoardStats {
                name,
                done,
                total,
                items,
            }
        })
        .collect()
}

pub fn group_notes_by_board(items: &[&Item]) -> BTreeMap<String, Vec<Item>> {
    let mut map: BTreeMap<String, Vec<Item>> = BTreeMap::new();

    for item in items {
        if item.item_type == ItemType::Note {
            map.entry(item.board.clone())
                .or_default()
                .push((*item).clone());
        }
    }

    map
}

pub fn group_all_by_board(items: &[&Item]) -> Vec<BoardStats> {
    let mut map: BTreeMap<String, Vec<Item>> = BTreeMap::new();

    for item in items {
        map.entry(item.board.clone())
            .or_default()
            .push((*item).clone());
    }

    map.into_iter()
        .map(|(name, items)| {
            let done = items
                .iter()
                .filter(|i| i.item_type == ItemType::Task && i.status == Status::Done)
                .count();
            let total = items
                .iter()
                .filter(|i| i.item_type == ItemType::Task)
                .count();
            BoardStats {
                name,
                done,
                total,
                items,
            }
        })
        .collect()
}
