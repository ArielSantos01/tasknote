use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "tasknote",
    about = "CLI + TUI task and note manager",
    long_about = None,
    disable_help_subcommand = true,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Add a task: tn -t "description" [-b "Board"]
    #[command(name = "task", short_flag = 't')]
    Task(AddItem),

    /// Add a note: tn -n "content" [-b "Board"]
    #[command(name = "note", short_flag = 'n')]
    Note(AddItem),

    /// Show all task boards
    #[command(name = "tasks", long_flag = "tasks")]
    Tasks,

    /// Show all note boards
    #[command(name = "notes", long_flag = "notes")]
    Notes,

    /// Show tasks and notes together
    #[command(name = "all", long_flag = "all")]
    All,

    /// Show tasks sorted by creation date
    #[command(name = "timeline", long_flag = "timeline")]
    Timeline,

    /// Filter view by board name: tn -b "Board"
    #[command(name = "board", short_flag = 'b')]
    Board { name: String },

    /// Show archived items
    #[command(name = "archive", long_flag = "archive")]
    Archive,

    /// Edit item description: tn -e <ID> ["new description"]
    #[command(name = "edit", short_flag = 'e')]
    Edit { id: u32, description: Option<String> },

    /// Set priority: tn -p <ID> <low|medium|high|none>
    #[command(name = "priority", short_flag = 'p')]
    Priority { id: u32, level: String },

    /// Move items to another board: tn -m <IDs> "Board"
    #[command(name = "move", short_flag = 'm')]
    Move { ids: String, board: String },

    /// Delete items (archive): tn -d <IDs>
    #[command(name = "delete", short_flag = 'd')]
    Delete { ids: String },

    /// Mark tasks as done: tn done 1,2,3
    #[command(name = "done")]
    Done { ids: String },

    /// Unmark tasks: tn undone 1,2,3
    #[command(name = "undone")]
    Undone { ids: String },

    /// Restore items from archive: tn --restore <IDs>
    #[command(name = "restore", long_flag = "restore")]
    Restore { ids: String },

    /// Permanently delete all archived items
    #[command(name = "clear", long_flag = "clear")]
    Clear,

    /// Get or set configuration: tn config board "Name"
    #[command(name = "config")]
    Config {
        #[command(subcommand)]
        setting: Option<ConfigSetting>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigSetting {
    /// Set the default board name
    Board { name: String },
}

#[derive(Args, Debug)]
pub struct AddItem {
    pub description: String,

    /// Board name (default: "My Board")
    #[arg(short = 'b', long = "board")]
    pub board: Option<String>,
}

pub fn parse_ids(ids_str: &str) -> Result<Vec<u32>, String> {
    ids_str
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<u32>()
                .map_err(|_| format!("Invalid ID: '{}'", s.trim()))
        })
        .collect()
}
