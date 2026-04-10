use colored::*;

use crate::board::{group_all_by_board, group_notes_by_board, group_tasks_by_board};
use crate::item::{Item, ItemType, Priority, Status};
use crate::storage::{active_items, Store};

const SEPARATOR: &str = "─────────────────────────────────────────────────────────";
const COL_WIDTH: usize = 60;

fn priority_colored(p: &Priority) -> ColoredString {
    match p {
        Priority::Low => p.label().cyan(),
        Priority::Medium => p.label().bold().yellow(),
        Priority::High => p.label().bold().red(),
    }
}

fn format_item_line(item: &Item, selected: bool) -> String {
    let icon = match item.item_type {
        ItemType::Task => match item.status {
            Status::Done => "✔".green().to_string(),
            Status::InProgress => "●".cyan().to_string(),
            Status::Pending => "●".white().to_string(),
        },
        ItemType::Note => "◆".blue().to_string(),
    };

    let id_str = format!("{:>3}", item.id).dimmed().to_string();

    let desc = if item.status == Status::Done {
        item.description.strikethrough().green().to_string()
    } else {
        item.description.normal().to_string()
    };

    let date_str = if let Some(dt) = &item.completed_at {
        dt.format("%Y-%m-%d").to_string().dimmed().to_string()
    } else {
        String::new()
    };

    let priority_str = if let Some(p) = &item.priority {
        priority_colored(p).to_string()
    } else {
        String::new()
    };

    // Build right side: date or priority (priority wins if task not done)
    let right = if !date_str.is_empty() {
        date_str
    } else {
        priority_str
    };

    // Compute padding
    let desc_display_len = item.description.len();
    let prefix_len = 2 + 1 + 3 + 3 + 2; // "  ● " + id + " · "
    let available = if COL_WIDTH > prefix_len + desc_display_len {
        COL_WIDTH - prefix_len - desc_display_len
    } else {
        2
    };

    let padding = " ".repeat(available.max(2));

    let prefix = if selected {
        format!("▶ {} {} · {}", icon, id_str, desc)
    } else {
        format!("  {} {} · {}", icon, id_str, desc)
    };

    if right.is_empty() {
        prefix
    } else {
        format!("{}{}{}", prefix, padding, right)
    }
}

pub fn print_tasks(store: &Store) {
    let active = active_items(store);
    let boards = group_tasks_by_board(&active);

    if boards.is_empty() {
        println!("{}", "No tasks found.".dimmed());
        return;
    }

    let mut total_tasks = 0usize;
    let mut total_done = 0usize;
    let board_count = boards.len();

    for board in &boards {
        println!(
            "\n  {}",
            format!("{} [{}/{}]", board.name, board.done, board.total)
                .bold()
                .white()
        );
        for item in &board.items {
            println!("{}", format_item_line(item, false));
        }
        total_tasks += board.total;
        total_done += board.done;
    }

    println!("\n  {}", SEPARATOR.dimmed());

    let pct = if total_tasks > 0 {
        (total_done * 100) / total_tasks
    } else {
        0
    };
    println!(
        "  {}",
        format!(
            "{}% of {} tasks done · across {} board{}",
            pct,
            total_tasks,
            board_count,
            if board_count == 1 { "" } else { "s" }
        )
        .dimmed()
    );
}

pub fn print_notes(store: &Store) {
    let active = active_items(store);
    let boards = group_notes_by_board(&active);

    if boards.is_empty() {
        println!("{}", "No notes found.".dimmed());
        return;
    }

    let mut total_notes = 0usize;
    let board_count = boards.len();

    for (name, items) in &boards {
        println!("\n  {}", name.bold().white());
        for item in items {
            println!("{}", format_item_line(item, false));
        }
        total_notes += items.len();
    }

    println!("\n  {}", SEPARATOR.dimmed());
    println!(
        "  {}",
        format!(
            "{} note{} · across {} board{}",
            total_notes,
            if total_notes == 1 { "" } else { "s" },
            board_count,
            if board_count == 1 { "" } else { "s" }
        )
        .dimmed()
    );
}

pub fn print_all(store: &Store) {
    let active = active_items(store);
    let boards = group_all_by_board(&active);

    if boards.is_empty() {
        println!("{}", "No items found.".dimmed());
        return;
    }

    for board in &boards {
        let has_tasks = board.total > 0;

        let header = if has_tasks {
            format!("{} [{}/{}]", board.name, board.done, board.total)
        } else {
            board.name.clone()
        };

        println!("\n  {}", header.bold().white());

        for item in &board.items {
            println!("{}", format_item_line(item, false));
        }
    }

    println!("\n  {}", SEPARATOR.dimmed());
}

pub fn print_timeline(store: &Store) {
    let active = active_items(store);
    let mut tasks: Vec<&Item> = active
        .iter()
        .filter(|i| i.item_type == ItemType::Task)
        .copied()
        .collect();

    tasks.sort_by_key(|i| i.created_at);

    if tasks.is_empty() {
        println!("{}", "No tasks found.".dimmed());
        return;
    }

    println!("\n  {}", "Timeline".bold().white());
    for item in &tasks {
        println!("{}", format_item_line(item, false));
    }
    println!("\n  {}", SEPARATOR.dimmed());
    println!("  {}", format!("{} tasks", tasks.len()).dimmed());
}

pub fn print_board(store: &Store, board_name: &str) {
    let active = active_items(store);
    let boards = group_tasks_by_board(&active);

    let board = boards.iter().find(|b| {
        b.name.to_lowercase() == board_name.to_lowercase()
    });

    match board {
        None => eprintln!(
            "{}",
            format!("Board '{}' not found.", board_name).red()
        ),
        Some(board) => {
            println!(
                "\n  {}",
                format!("{} [{}/{}]", board.name, board.done, board.total)
                    .bold()
                    .white()
            );
            for item in &board.items {
                println!("{}", format_item_line(item, false));
            }
            println!("\n  {}", SEPARATOR.dimmed());
        }
    }
}


