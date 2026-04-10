mod board;
mod cli;
mod display;
mod item;
mod storage;
mod tui;

use clap::Parser;
use colored::*;

use cli::{parse_ids, Cli, Command, ConfigSetting};
use item::{Item, ItemType, Priority};
use storage::{find_item_mut, load, save};

const DEFAULT_BOARD: &str = "My Board";

fn main() {
    // If no args → launch TUI
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        let mut store = load();
        if let Err(e) = tui::run_tui(&mut store) {
            eprintln!("{}", format!("TUI error: {}", e).red());
            std::process::exit(1);
        }
        return;
    }

    let cli = Cli::parse();
    let mut store = load();

    let Some(cmd) = cli.command else {
        // No subcommand recognized; print help
        Cli::parse_from(["tasknote", "--help"]);
        return;
    };

    match cmd {
        // ── Config ────────────────────────────────────────────
        Command::Config { setting } => {
            match setting {
                None => {
                    let board = store.default_board.as_deref().unwrap_or(DEFAULT_BOARD);
                    println!("default board: {}", board.bold().white());
                }
                Some(ConfigSetting::Board { name }) => {
                    store.default_board = Some(name.clone());
                    save(&store);
                    println!("{}", format!("Default board set to '{}'.", name).green());
                }
            }
            return;
        }

        // ── Add task ───────────────────────────────────────────
        Command::Task(add) => {
            let board = add.board.unwrap_or_else(|| {
                store.default_board.clone().unwrap_or_else(|| DEFAULT_BOARD.to_string())
            });
            let id = store.next_id();
            store.items.push(Item::new_task(id, add.description, board.clone()));
            save(&store);
            println!("{}", format!("Task #{} added to '{}'.", id, board).green());
        }

        // ── Add note ───────────────────────────────────────────
        Command::Note(add) => {
            let board = add.board.unwrap_or_else(|| {
                store.default_board.clone().unwrap_or_else(|| DEFAULT_BOARD.to_string())
            });
            let id = store.next_id();
            store.items.push(Item::new_note(id, add.description, board.clone()));
            save(&store);
            println!("{}", format!("Note #{} added to '{}'.", id, board).green());
        }

        // ── View: tasks ────────────────────────────────────────
        Command::Tasks => {
            display::print_tasks(&store);
        }

        // ── View: notes ────────────────────────────────────────
        Command::Notes => {
            display::print_notes(&store);
        }

        // ── View: all ──────────────────────────────────────────
        Command::All => {
            display::print_all(&store);
        }

        // ── View: timeline ─────────────────────────────────────
        Command::Timeline => {
            display::print_timeline(&store);
        }

        // ── View: board filter ─────────────────────────────────
        Command::Board { name } => {
            display::print_board(&store, &name);
        }

        // ── View: archive ──────────────────────────────────────
        Command::Archive => {
            if let Err(e) = tui::run_archive_tui(&mut store) {
                eprintln!("{}", format!("TUI error: {}", e).red());
                std::process::exit(1);
            }
        }

        // ── Edit item ──────────────────────────────────────────
        Command::Edit { id, description } => {
            let new_desc = match description {
                Some(d) => Some(d),
                None => {
                    // Find current description to pre-fill
                    let current = store
                        .items
                        .iter()
                        .find(|i| i.id == id)
                        .map(|i| i.description.clone());

                    match current {
                        None => {
                            eprintln!("{}", format!("Item #{} not found.", id).red());
                            return;
                        }
                        Some(current_desc) => {
                            let mut rl = rustyline::DefaultEditor::new()
                                .expect("Failed to init editor");
                            match rl.readline_with_initial("  ", (&current_desc, "")) {
                                Ok(line) => {
                                    let trimmed = line.trim().to_string();
                                    if trimmed.is_empty() {
                                        println!("{}", "Edit cancelled.".dimmed());
                                        return;
                                    }
                                    Some(trimmed)
                                }
                                Err(_) => {
                                    println!("{}", "Edit cancelled.".dimmed());
                                    return;
                                }
                            }
                        }
                    }
                }
            };

            if let Some(desc) = new_desc {
                match find_item_mut(&mut store, id) {
                    None => eprintln!("{}", format!("Item #{} not found.", id).red()),
                    Some(item) => {
                        item.description = desc;
                        save(&store);
                        println!("{}", format!("Item #{} updated.", id).green());
                    }
                }
            }
        }

        // ── Set priority ───────────────────────────────────────
        Command::Priority { id, level } => {
            match Priority::from_str(&level) {
                None => eprintln!(
                    "{}",
                    "Invalid priority. Use: low | medium | high | none".red()
                ),
                Some(priority) => match find_item_mut(&mut store, id) {
                    None => eprintln!("{}", format!("Item #{} not found.", id).red()),
                    Some(item) => {
                        item.priority = priority;
                        save(&store);
                        println!("{}", format!("Priority set for item #{}.", id).green());
                    }
                },
            }
        }

        // ── Move items ─────────────────────────────────────────
        Command::Move { ids, board } => {
            match parse_ids(&ids) {
                Err(e) => eprintln!("{}", e.red()),
                Ok(ids) => {
                    let mut count = 0;
                    for id in &ids {
                        match find_item_mut(&mut store, *id) {
                            None => eprintln!("{}", format!("Item #{} not found.", id).red()),
                            Some(item) => {
                                item.board = board.clone();
                                count += 1;
                            }
                        }
                    }
                    save(&store);
                    println!(
                        "{}",
                        format!("Moved {} item(s) to '{}'.", count, board).green()
                    );
                }
            }
        }

        // ── Delete (archive) ───────────────────────────────────
        Command::Delete { ids } => {
            match parse_ids(&ids) {
                Err(e) => eprintln!("{}", e.red()),
                Ok(ids) => {
                    let mut count = 0;
                    for id in &ids {
                        match find_item_mut(&mut store, *id) {
                            None => eprintln!("{}", format!("Item #{} not found.", id).red()),
                            Some(item) => {
                                item.archived = true;
                                count += 1;
                            }
                        }
                    }
                    save(&store);
                    println!("{}", format!("Archived {} item(s).", count).green());
                }
            }
        }

        // ── Done ───────────────────────────────────────────────
        Command::Done { ids } => {
            match parse_ids(&ids) {
                Err(e) => eprintln!("{}", e.red()),
                Ok(ids) => {
                    let mut count = 0;
                    for id in &ids {
                        match find_item_mut(&mut store, *id) {
                            None => eprintln!("{}", format!("Item #{} not found.", id).red()),
                            Some(item) => {
                                if item.item_type != ItemType::Task {
                                    eprintln!(
                                        "{}",
                                        format!("Item #{} is not a task.", id).yellow()
                                    );
                                } else {
                                    item.mark_done();
                                    count += 1;
                                }
                            }
                        }
                    }
                    save(&store);
                    println!("{}", format!("Marked {} task(s) as done.", count).green());
                }
            }
        }

        // ── Undone ─────────────────────────────────────────────
        Command::Undone { ids } => {
            match parse_ids(&ids) {
                Err(e) => eprintln!("{}", e.red()),
                Ok(ids) => {
                    let mut count = 0;
                    for id in &ids {
                        match find_item_mut(&mut store, *id) {
                            None => eprintln!("{}", format!("Item #{} not found.", id).red()),
                            Some(item) => {
                                item.mark_undone();
                                count += 1;
                            }
                        }
                    }
                    save(&store);
                    println!("{}", format!("Marked {} item(s) as undone.", count).green());
                }
            }
        }

        // ── Restore ────────────────────────────────────────────
        Command::Restore { ids } => {
            match parse_ids(&ids) {
                Err(e) => eprintln!("{}", e.red()),
                Ok(ids) => {
                    let mut count = 0;
                    for id in &ids {
                        match find_item_mut(&mut store, *id) {
                            None => eprintln!("{}", format!("Item #{} not found.", id).red()),
                            Some(item) => {
                                item.archived = false;
                                count += 1;
                            }
                        }
                    }
                    save(&store);
                    println!("{}", format!("Restored {} item(s).", count).green());
                }
            }
        }

        // ── Clear archive ──────────────────────────────────────
        Command::Clear => {
            let before = store.items.len();
            store.items.retain(|i| !i.archived);
            let removed = before - store.items.len();
            save(&store);
            println!(
                "{}",
                format!("Permanently deleted {} archived item(s).", removed).green()
            );
        }
    }
}
