use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ItemType {
    Task,
    Note,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl Priority {
    pub fn from_str(s: &str) -> Option<Option<Priority>> {
        match s.to_lowercase().as_str() {
            "low" => Some(Some(Priority::Low)),
            "medium" => Some(Some(Priority::Medium)),
            "high" => Some(Some(Priority::High)),
            "none" => Some(None),
            _ => None,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Priority::Low => "LOW",
            Priority::Medium => "MEDIUM",
            Priority::High => "HIGH",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Status {
    Pending,
    InProgress,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: u32,
    pub item_type: ItemType,
    pub description: String,
    pub board: String,
    pub priority: Option<Priority>,
    pub status: Status,
    pub created_at: DateTime<Local>,
    pub completed_at: Option<DateTime<Local>>,
    pub archived: bool,
}

impl Item {
    pub fn new_task(id: u32, description: String, board: String) -> Self {
        Item {
            id,
            item_type: ItemType::Task,
            description,
            board,
            priority: None,
            status: Status::Pending,
            created_at: Local::now(),
            completed_at: None,
            archived: false,
        }
    }

    pub fn new_note(id: u32, description: String, board: String) -> Self {
        Item {
            id,
            item_type: ItemType::Note,
            description,
            board,
            priority: None,
            status: Status::Pending,
            created_at: Local::now(),
            completed_at: None,
            archived: false,
        }
    }

    pub fn is_done(&self) -> bool {
        self.status == Status::Done
    }

    pub fn mark_done(&mut self) {
        self.status = Status::Done;
        self.completed_at = Some(Local::now());
    }

    pub fn mark_undone(&mut self) {
        self.status = Status::Pending;
        self.completed_at = None;
    }
}
