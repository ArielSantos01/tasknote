# tasknote

A fast CLI + TUI task and note manager built in Rust.

## Install

```bash
cargo build --release
cp target/release/tasknote ~/.local/bin/tn
```

## Usage

```
tn                          # open TUI (interactive mode)
tn --archive                # open archive TUI

# Add items
tn -t "Buy coffee"          # add task to default board
tn -t "Review PR" -b "Work" # add task to specific board
tn -n "Useful link: ..."    # add note

# View
tn --tasks                  # show all task boards
tn --notes                  # show all note boards
tn --all                    # show everything
tn --timeline               # tasks sorted by creation date
tn -b "Work"                # filter by board

# Complete tasks
tn done 1                   # mark task #1 done
tn done 1,2,3               # mark multiple done
tn undone 1                 # mark undone

# Edit
tn -e 1                     # edit description interactively (pre-filled)
tn -e 1 "new description"   # edit description directly
tn -p 1 high                # set priority (low|medium|high|none)
tn -m 1,2 "Work"            # move items to another board

# Config
tn config                   # show current default board
tn config board "Personal"  # set default board

# Archive
tn -d 1,2                   # archive items
tn --archive                # open archive TUI (restore or permanently delete)
tn --restore 1              # restore from archive (CLI)
tn --clear                  # permanently delete all archived items
```

## TUI — Active tasks (`tn`)

| Key         | Action                        |
|-------------|-------------------------------|
| ↑ / k       | Move cursor up                |
| ↓ / j       | Move cursor down              |
| Enter/Space | Mark done / undone            |
| e           | Edit description (pre-filled) |
| p           | Set priority (interactive)    |
| d           | Archive selected task         |
| q / Esc     | Quit                          |

## TUI — Archive (`tn --archive`)

| Key | Action                    |
|-----|---------------------------|
| ↑ / k | Move cursor up        |
| ↓ / j | Move cursor down      |
| r   | Restore selected item     |
| x   | Permanently delete item   |
| q / Esc | Quit               |

## Priority levels

| Level  | Color  |
|--------|--------|
| HIGH   | Red    |
| MEDIUM | Yellow |
| LOW    | Cyan   |

## Data

Stored in `~/.tasknote/storage.json`.
