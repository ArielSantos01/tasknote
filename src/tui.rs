use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Terminal,
};
use std::io;
use std::io::Write as _;

use crate::board::group_tasks_by_board;
use crate::item::{ItemType, Priority, Status};
use crate::storage::{active_items, archived_items, find_item_mut, save, Store};

enum InputMode {
    Normal,
    PrioritySelect,
}

struct TuiState {
    selectable: Vec<u32>,
    cursor: usize,
    mode: InputMode,
}

impl TuiState {
    fn new(selectable: Vec<u32>) -> Self {
        TuiState {
            selectable,
            cursor: 0,
            mode: InputMode::Normal,
        }
    }

    fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    fn move_down(&mut self) {
        if !self.selectable.is_empty() && self.cursor < self.selectable.len() - 1 {
            self.cursor += 1;
        }
    }

    fn selected_id(&self) -> Option<u32> {
        self.selectable.get(self.cursor).copied()
    }
}

pub fn run_tui(store: &mut Store) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, store);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn build_selectable(store: &Store) -> Vec<u32> {
    let active = active_items(store);
    let boards = group_tasks_by_board(&active);
    let mut ids = vec![];
    for board in &boards {
        for item in &board.items {
            ids.push(item.id);
        }
    }
    ids
}

fn priority_span(p: &Priority) -> Span<'static> {
    match p {
        Priority::Low => Span::styled(
            format!("{:<9}", p.label()),
            Style::default().fg(Color::Cyan),
        ),
        Priority::Medium => Span::styled(
            format!("{:<9}", p.label()),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
        Priority::High => Span::styled(
            format!("{:<9}", p.label()),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
    }
}

// prefix width: cursor(2) + id(4) + icon(2) + priority(9) + date(23) = 40
const PREFIX_LEN: usize = 40;

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 || text.chars().count() <= width {
        return vec![text.to_string()];
    }
    let mut result = vec![];
    let mut remaining = text;
    loop {
        let char_count = remaining.chars().count();
        if char_count <= width {
            if !remaining.is_empty() {
                result.push(remaining.to_string());
            }
            break;
        }
        // Find last space within width to break cleanly
        let chars: Vec<char> = remaining.chars().collect();
        let break_at = chars[..width]
            .iter()
            .rposition(|&c| c == ' ')
            .unwrap_or(width);
        let break_at = if break_at == 0 { width } else { break_at };
        let (head, tail) = remaining.split_at(
            remaining.char_indices().nth(break_at).map(|(i, _)| i).unwrap_or(remaining.len()),
        );
        result.push(head.trim_end().to_string());
        remaining = tail.trim_start();
    }
    result
}

fn col_header_line() -> Line<'static> {
    let text = format!(
        "{:<2}{:<4}{:<2}{:<9}{:<23}{}",
        "", "ID", "", "PRIORITY", "CREATED/DONE", "DESCRIPTION"
    );
    Line::from(Span::styled(
        text,
        Style::default().fg(Color::DarkGray),
    ))
}

fn separator_line() -> Line<'static> {
    Line::from(Span::styled(
        "  ─────────────────────────────────────────────────────",
        Style::default().fg(Color::DarkGray),
    ))
}

fn render_lines(store: &Store, state: &TuiState, width: u16) -> Vec<Line<'static>> {
    let active = active_items(store);
    let boards = group_tasks_by_board(&active);
    let mut lines: Vec<Line<'static>> = vec![];
    let desc_width = (width as usize).saturating_sub(PREFIX_LEN);

    for board in &boards {
        lines.push(Line::from(Span::styled(
            format!("  {} [{}/{}]", board.name, board.done, board.total),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )));
        lines.push(col_header_line());
        lines.push(separator_line());

        for item in &board.items {
            let selected = state.selected_id() == Some(item.id);
            let line_style = if selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let cursor = Span::raw(if selected { "▶ " } else { "  " });
            let id_span = Span::styled(
                format!("{:>3} ", item.id),
                Style::default().fg(Color::DarkGray),
            );
            let icon_span = match item.status {
                Status::Done => Span::styled("✔ ", Style::default().fg(Color::Green)),
                Status::InProgress => Span::styled("● ", Style::default().fg(Color::Cyan)),
                Status::Pending => Span::styled("● ", Style::default().fg(Color::White)),
            };
            let prio_span = match &item.priority {
                None => Span::raw("         "),
                Some(p) => priority_span(p),
            };
            let created = item.created_at.format("%Y-%m-%d").to_string();
            let done_date = item.completed_at
                .as_ref()
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_default();
            let date_span = Span::styled(
                format!("{:<23}", format!("{}/{}", created, done_date)),
                Style::default().fg(Color::DarkGray),
            );

            let desc_style = if item.status == Status::Done {
                Style::default().fg(Color::Green).add_modifier(Modifier::CROSSED_OUT)
            } else {
                Style::default().fg(Color::White)
            };

            let chunks = wrap_text(&item.description, desc_width.max(1));

            // First line: all columns + first chunk of description
            let first = Span::styled(chunks[0].clone(), desc_style);
            lines.push(
                Line::from(vec![cursor, id_span, icon_span, prio_span, date_span, first])
                    .style(line_style),
            );

            // Continuation lines (indented to description column)
            let indent = " ".repeat(PREFIX_LEN);
            for chunk in &chunks[1..] {
                lines.push(
                    Line::from(Span::styled(format!("{}{}", indent, chunk), desc_style))
                        .style(line_style),
                );
            }
        }

        lines.push(Line::from(""));
    }

    lines
}

fn hint_line<'a>(pairs: &[(&'a str, &'a str)]) -> Line<'a> {
    let style = Style::default().fg(Color::DarkGray);
    let mut spans = vec![Span::raw("  ")];
    for (key, label) in pairs {
        spans.push(Span::styled(*key, style));
        spans.push(Span::styled(*label, style));
    }
    Line::from(spans)
}

fn render_footer<'a>(state: &TuiState) -> (Line<'a>, Line<'a>) {
    let sep = Line::from(Span::styled(
        "  ─────────────────────────────────────────────────────────",
        Style::default().fg(Color::DarkGray),
    ));

    let hints = match state.mode {
        InputMode::Normal => hint_line(&[
            ("[↑↓]", " move  "),
            ("[Enter]", " done/undone  "),
            ("[e]", " edit  "),
            ("[p]", " priority  "),
            ("[d]", " delete  "),
            ("[q]", " quit"),
        ]),
        InputMode::PrioritySelect => Line::from(vec![
            Span::styled("  Priority: ", Style::default().fg(Color::DarkGray)),
            Span::styled("[n]", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" NONE  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[l]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" LOW  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[m]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" MEDIUM  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[h]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(" HIGH  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[Esc]", Style::default().fg(Color::DarkGray)),
            Span::styled(" cancel", Style::default().fg(Color::DarkGray)),
        ]),
    };

    (sep, hints)
}

fn suspend_tui(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

fn resume_tui(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
    terminal.clear()?;
    Ok(())
}

// ── Archive TUI ────────────────────────────────────────────────────────────

pub fn run_archive_tui(store: &mut Store) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_archive_loop(&mut terminal, store);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn build_archive_selectable(store: &Store) -> Vec<u32> {
    archived_items(store).iter().map(|i| i.id).collect()
}

fn render_archive_lines(store: &Store, cursor_id: Option<u32>, width: u16) -> Vec<Line<'static>> {
    let archived = archived_items(store);
    let mut lines: Vec<Line<'static>> = vec![];

    if archived.is_empty() {
        lines.push(Line::from(Span::styled(
            "  Archive is empty.",
            Style::default().fg(Color::DarkGray),
        )));
        return lines;
    }

    let mut boards: Vec<String> = vec![];
    for item in &archived {
        if !boards.contains(&item.board) {
            boards.push(item.board.clone());
        }
    }

    let desc_width = (width as usize).saturating_sub(PREFIX_LEN);

    for board_name in &boards {
        lines.push(Line::from(Span::styled(
            format!("  {}", board_name),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )));
        lines.push(col_header_line());
        lines.push(separator_line());

        for item in archived.iter().filter(|i| &i.board == board_name) {
            let selected = cursor_id == Some(item.id);
            let line_style = if selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let cursor = Span::raw(if selected { "▶ " } else { "  " });
            let id_span = Span::styled(
                format!("{:>3} ", item.id),
                Style::default().fg(Color::DarkGray),
            );
            let icon_span = match item.item_type {
                ItemType::Task => match item.status {
                    Status::Done => Span::styled("✔ ", Style::default().fg(Color::Green)),
                    _ => Span::styled("● ", Style::default().fg(Color::White)),
                },
                ItemType::Note => Span::styled("◆ ", Style::default().fg(Color::Blue)),
            };
            let prio_span = match &item.priority {
                None => Span::raw("         "),
                Some(p) => priority_span(p),
            };
            let created = item.created_at.format("%Y-%m-%d").to_string();
            let done_date = item.completed_at
                .as_ref()
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_default();
            let date_span = Span::styled(
                format!("{:<23}", format!("{}/{}", created, done_date)),
                Style::default().fg(Color::DarkGray),
            );
            let desc_style = Style::default().fg(Color::DarkGray);
            let chunks = wrap_text(&item.description, desc_width.max(1));

            lines.push(
                Line::from(vec![
                    cursor,
                    id_span,
                    icon_span,
                    prio_span,
                    date_span,
                    Span::styled(chunks[0].clone(), desc_style),
                ])
                .style(line_style),
            );
            let indent = " ".repeat(PREFIX_LEN);
            for chunk in &chunks[1..] {
                lines.push(
                    Line::from(Span::styled(format!("{}{}", indent, chunk), desc_style))
                        .style(line_style),
                );
            }
        }

        lines.push(Line::from(""));
    }

    lines
}

fn run_archive_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    store: &mut Store,
) -> io::Result<()> {
    let mut selectable = build_archive_selectable(store);
    let mut cursor: usize = 0;

    loop {
        let cursor_id = selectable.get(cursor).copied();

        terminal.draw(|frame| {
            let size = frame.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1), Constraint::Length(1)])
                .split(size);

            let lines = render_archive_lines(store, cursor_id, chunks[0].width);
            frame.render_widget(Paragraph::new(lines), chunks[0]);

            let sep = Paragraph::new(Line::from(Span::styled(
                "  ─────────────────────────────────────────────────────────",
                Style::default().fg(Color::DarkGray),
            )));
            frame.render_widget(sep, chunks[1]);

            let hints = Paragraph::new(hint_line(&[
                ("[↑↓]", " move  "),
                ("[r]", " restore  "),
                ("[x]", " delete permanently  "),
                ("[q]", " quit"),
            ]));
            frame.render_widget(hints, chunks[2]);
        })?;

        if !event::poll(std::time::Duration::from_millis(200))? {
            continue;
        }

        let Event::Key(key) = event::read()? else { continue };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => break,

            KeyCode::Up | KeyCode::Char('k') => {
                if cursor > 0 {
                    cursor -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !selectable.is_empty() && cursor < selectable.len() - 1 {
                    cursor += 1;
                }
            }

            KeyCode::Char('r') => {
                if let Some(id) = cursor_id {
                    if let Some(item) = find_item_mut(store, id) {
                        item.archived = false;
                    }
                    save(store);
                    selectable = build_archive_selectable(store);
                    if !selectable.is_empty() && cursor >= selectable.len() {
                        cursor = selectable.len() - 1;
                    }
                }
            }

            KeyCode::Char('x') => {
                if let Some(id) = cursor_id {
                    store.items.retain(|i| i.id != id);
                    save(store);
                    selectable = build_archive_selectable(store);
                    if !selectable.is_empty() && cursor >= selectable.len() {
                        cursor = selectable.len() - 1;
                    }
                }
            }

            _ => {}
        }
    }

    Ok(())
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    store: &mut Store,
) -> io::Result<()> {
    let mut state = TuiState::new(build_selectable(store));

    loop {
        terminal.draw(|frame| {
            let size = frame.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1), Constraint::Length(1)])
                .split(size);

            let lines = render_lines(store, &state, chunks[0].width);
            frame.render_widget(Paragraph::new(lines), chunks[0]);

            let (sep, hints) = render_footer(&state);
            frame.render_widget(Paragraph::new(sep), chunks[1]);
            frame.render_widget(Paragraph::new(hints), chunks[2]);
        })?;

        if !event::poll(std::time::Duration::from_millis(200))? {
            continue;
        }

        let Event::Key(key) = event::read()? else { continue };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match state.mode {
            // ── Priority selector ──────────────────────────────
            InputMode::PrioritySelect => {
                let new_priority = match key.code {
                    KeyCode::Char('n') => Some(None),
                    KeyCode::Char('l') => Some(Some(Priority::Low)),
                    KeyCode::Char('m') => Some(Some(Priority::Medium)),
                    KeyCode::Char('h') => Some(Some(Priority::High)),
                    KeyCode::Esc => None,
                    _ => continue,
                };
                state.mode = InputMode::Normal;
                if let Some(priority) = new_priority {
                    if let Some(id) = state.selected_id() {
                        if let Some(item) = find_item_mut(store, id) {
                            item.priority = priority;
                        }
                        save(store);
                    }
                }
            }

            // ── Normal mode ────────────────────────────────────
            InputMode::Normal => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,

                KeyCode::Up | KeyCode::Char('k') => state.move_up(),
                KeyCode::Down | KeyCode::Char('j') => state.move_down(),

                // Toggle done/undone
                KeyCode::Enter | KeyCode::Char(' ') => {
                    if let Some(id) = state.selected_id() {
                        if let Some(item) = find_item_mut(store, id) {
                            if item.is_done() {
                                item.mark_undone();
                            } else {
                                item.mark_done();
                            }
                        }
                        save(store);
                        state.selectable = build_selectable(store);
                        if state.cursor >= state.selectable.len() && !state.selectable.is_empty() {
                            state.cursor = state.selectable.len() - 1;
                        }
                    }
                }

                // Edit description
                KeyCode::Char('e') => {
                    if let Some(id) = state.selected_id() {
                        let current_desc = store
                            .items
                            .iter()
                            .find(|i| i.id == id)
                            .map(|i| i.description.clone())
                            .unwrap_or_default();

                        suspend_tui(terminal)?;

                        print!("  Edit: ");
                        io::stdout().flush()?;

                        let mut rl = rustyline::DefaultEditor::new()
                            .expect("Failed to init editor");
                        let result = rl.readline_with_initial("", (&current_desc, ""));

                        resume_tui(terminal)?;

                        if let Ok(new_desc) = result {
                            let trimmed = new_desc.trim().to_string();
                            if !trimmed.is_empty() && trimmed != current_desc {
                                if let Some(item) = find_item_mut(store, id) {
                                    item.description = trimmed;
                                }
                                save(store);
                            }
                        }
                    }
                }

                // Delete (archive)
                KeyCode::Char('d') => {
                    if let Some(id) = state.selected_id() {
                        if let Some(item) = find_item_mut(store, id) {
                            item.archived = true;
                        }
                        save(store);
                        state.selectable = build_selectable(store);
                        if !state.selectable.is_empty() && state.cursor >= state.selectable.len() {
                            state.cursor = state.selectable.len() - 1;
                        }
                    }
                }

                // Priority selector
                KeyCode::Char('p') => {
                    if state.selected_id().is_some() {
                        state.mode = InputMode::PrioritySelect;
                    }
                }

                _ => {}
            },
        }
    }

    Ok(())
}
