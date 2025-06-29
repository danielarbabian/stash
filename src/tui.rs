use std::io;
use std::fs;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use tui_textarea::TextArea;
use uuid::Uuid;

use crate::models::Note;
use crate::store;

#[derive(Debug, Clone)]
pub enum AppMode {
    Home,
    AddNote,
    ViewNote(Uuid),
    Help,
}

#[derive(Debug, Clone)]
pub enum EditorMode {
    Command,
    Insert,
}

pub struct App {
    pub mode: AppMode,
    pub editor_mode: EditorMode,
    pub notes: Vec<Note>,
    pub selected_note: usize,
    pub content_editor: TextArea<'static>,
    pub title_input: String,
    pub active_field: ActiveField,
    pub should_quit: bool,
    pub status_message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ActiveField {
    Title,
    Content,
}

impl Default for App {
    fn default() -> App {
        let content_editor = TextArea::default();

        App {
            mode: AppMode::Home,
            editor_mode: EditorMode::Command,
            notes: Vec::new(),
            selected_note: 0,
            content_editor,
            title_input: String::new(),
            active_field: ActiveField::Content,
            should_quit: false,
            status_message: None,
        }
    }
}

impl App {
    pub fn new() -> App {
        let mut app = App::default();
        app.load_existing_notes();
        app
    }

    fn load_existing_notes(&mut self) {
        self.notes.clear();
        if let Some(home) = dirs::home_dir() {
            let notes_dir = home.join(".stash").join("notes");
            if let Ok(entries) = fs::read_dir(notes_dir) {
                for entry in entries.flatten() {
                    if let Some(extension) = entry.path().extension() {
                        if extension == "md" {
                            match Note::load_from_file(entry.path()) {
                                Ok(note) => {
                                    self.notes.push(note);
                                }
                                Err(e) => {
                                    eprintln!("Failed to load note {:?}: {}", entry.path(), e);
                                }
                            }
                        }
                    }
                }
                self.notes.sort_by(|a, b| b.created.cmp(&a.created));
            }
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal);

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if let Err(err) = result {
            println!("{err:?}");
        }

        Ok(())
    }

    fn run_app(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match self.mode {
                        AppMode::Home => self.handle_home_input(key.code),
                        AppMode::AddNote => self.handle_add_note_input(key.code, key.modifiers),
                        AppMode::ViewNote(_) => self.handle_view_note_input(key.code),
                        AppMode::Help => self.handle_help_input(key.code),
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    fn handle_home_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('a') => {
                self.mode = AppMode::AddNote;
                self.editor_mode = EditorMode::Insert;
                self.active_field = ActiveField::Content;
                self.content_editor = TextArea::default();
                self.title_input.clear();
            }
            KeyCode::Char('h') => self.mode = AppMode::Help,
            KeyCode::Char('r') => {
                self.load_existing_notes();
                self.status_message = Some("Notes refreshed".to_string());
            }
            KeyCode::Up => {
                if self.selected_note > 0 {
                    self.selected_note -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected_note < self.notes.len().saturating_sub(1) {
                    self.selected_note += 1;
                }
            }
            KeyCode::Enter => {
                if !self.notes.is_empty() && self.selected_note < self.notes.len() {
                    let note_id = self.notes[self.selected_note].id;
                    self.mode = AppMode::ViewNote(note_id);
                }
            }
            _ => {}
        }
    }

    fn handle_add_note_input(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match self.editor_mode {
            EditorMode::Insert => {
                match key {
                    KeyCode::Esc => {
                        self.editor_mode = EditorMode::Command;
                    }
                    _ => {
                        match self.active_field {
                            ActiveField::Content => {
                                self.content_editor.input(crossterm::event::KeyEvent::new(key, modifiers));
                            }
                            ActiveField::Title => {
                                match key {
                                    KeyCode::Char(c) => {
                                        self.title_input.push(c);
                                    }
                                    KeyCode::Backspace => {
                                        self.title_input.pop();
                                    }
                                    KeyCode::Enter => {
                                        self.active_field = ActiveField::Content;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
            EditorMode::Command => {
                match key {
                    KeyCode::Char('i') => {
                        self.editor_mode = EditorMode::Insert;
                    }
                    KeyCode::Char('s') if modifiers.contains(KeyModifiers::SUPER) => {
                        self.save_note();
                    }
                    KeyCode::Char('s') => {
                        self.save_note();
                    }
                    KeyCode::Char('q') => {
                        self.mode = AppMode::Home;
                        self.editor_mode = EditorMode::Command;
                    }
                    KeyCode::Char('t') => {
                        self.active_field = ActiveField::Title;
                        self.editor_mode = EditorMode::Insert;
                    }
                    KeyCode::Char('c') => {
                        self.active_field = ActiveField::Content;
                        self.editor_mode = EditorMode::Insert;
                    }
                    KeyCode::Tab => {
                        match self.active_field {
                            ActiveField::Title => self.active_field = ActiveField::Content,
                            ActiveField::Content => self.active_field = ActiveField::Title,
                        }
                    }
                    KeyCode::Esc => {
                        self.mode = AppMode::Home;
                        self.editor_mode = EditorMode::Command;
                    }
                    _ => {}
                }
            }
        }
    }

    fn save_note(&mut self) {
        let content = self.content_editor.lines().join("\n");
        if !content.trim().is_empty() {
            let title = if self.title_input.is_empty() {
                None
            } else {
                Some(self.title_input.clone())
            };

            match store::save_quick_note(content, title) {
                Ok(()) => {
                    self.status_message = Some("Note saved successfully".to_string());
                    self.load_existing_notes();
                    self.mode = AppMode::Home;
                    self.editor_mode = EditorMode::Command;
                    self.content_editor = TextArea::default();
                    self.title_input.clear();
                }
                Err(e) => {
                    self.status_message = Some(format!("Error saving note: {}", e));
                }
            }
        } else {
            self.status_message = Some("Cannot save empty note".to_string());
        }
    }

    fn handle_view_note_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.mode = AppMode::Home;
            }
            _ => {}
        }
    }

    fn handle_help_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.mode = AppMode::Home;
            }
            _ => {}
        }
    }

    fn ui(&mut self, f: &mut Frame) {
        match self.mode {
            AppMode::Home => self.render_home(f, f.area()),
            AppMode::AddNote => self.render_add_note(f, f.area()),
            AppMode::ViewNote(note_id) => self.render_view_note(f, f.area(), note_id),
            AppMode::Help => self.render_help(f, f.area()),
        }
    }

    fn render_home(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10),
                Constraint::Min(0),
            ])
            .split(area);

        let ascii_art = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  ███████╗████████╗ █████╗ ███████╗██╗  ██╗", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  ╚══███╔╝╚══██╔══╝██╔══██╗██╔════╝██║  ██║", Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("    ███╔╝    ██║   ███████║███████╗███████║", Style::default().fg(Color::Blue)),
            ]),
            Line::from(vec![
                Span::styled("   ███╔╝     ██║   ██╔══██║╚════██║██╔══██║", Style::default().fg(Color::Blue)),
            ]),
            Line::from(vec![
                Span::styled("  ███████╗   ██║   ██║  ██║███████║██║  ██║", Style::default().fg(Color::Magenta)),
            ]),
            Line::from(vec![
                Span::styled("  ╚══════╝   ╚═╝   ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝", Style::default().fg(Color::Magenta)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("a", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" add   "),
                Span::styled("h", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" help   "),
                Span::styled("r", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" refresh   "),
                Span::styled("↑↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" navigate   "),
                Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" quit"),
            ]),
        ];

        let title_widget = Paragraph::new(ascii_art)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);

        f.render_widget(title_widget, chunks[0]);

        let notes_block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Notes ({})", self.notes.len()));

        if self.notes.is_empty() {
            let empty_message = Paragraph::new("No notes found. Press 'a' to create your first note!")
                .block(notes_block)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(empty_message, chunks[1]);
        } else {
            let items: Vec<ListItem> = self.notes
                .iter()
                .enumerate()
                .map(|(i, note)| {
                    let title = note.title.as_deref().unwrap_or("Untitled");
                    let preview = if note.content.len() > 60 {
                        format!("{}...", &note.content[..60].replace('\n', " "))
                    } else {
                        note.content.replace('\n', " ")
                    };

                    let style = if i == self.selected_note {
                        Style::default().bg(Color::Blue).fg(Color::White)
                    } else {
                        Style::default()
                    };

                    ListItem::new(vec![
                        Line::from(vec![
                            Span::styled(format!("▶ {}", title), style.add_modifier(Modifier::BOLD)),
                        ]),
                        Line::from(vec![
                            Span::styled(format!("  {}", preview), style.fg(Color::DarkGray)),
                        ]),
                        Line::from(""),
                    ])
                })
                .collect();

            let list = List::new(items)
                .block(notes_block)
                .highlight_style(Style::default().bg(Color::Blue));

            f.render_widget(list, chunks[1]);
        }
    }

    fn render_add_note(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        let title_active = matches!(self.active_field, ActiveField::Title);
        let title_style = if title_active {
            match self.editor_mode {
                EditorMode::Insert => Style::default().fg(Color::Green),
                EditorMode::Command => Style::default().fg(Color::Yellow),
            }
        } else {
            Style::default()
        };

        let title_block = Block::default()
            .borders(Borders::ALL)
            .title("Title (t to edit)")
            .style(title_style);

        let title_input = Paragraph::new(self.title_input.as_str())
            .block(title_block);

        f.render_widget(title_input, chunks[0]);

        let content_active = matches!(self.active_field, ActiveField::Content);
        let content_style = if content_active {
            match self.editor_mode {
                EditorMode::Insert => Style::default().fg(Color::Green),
                EditorMode::Command => Style::default().fg(Color::Yellow),
            }
        } else {
            Style::default()
        };

        let content_block = Block::default()
            .borders(Borders::ALL)
            .title("Content (c to edit)")
            .style(content_style);

        let mut content_editor = self.content_editor.clone();
        content_editor.set_block(content_block);
        f.render_widget(&content_editor, chunks[1]);

        let mode_indicator = match self.editor_mode {
            EditorMode::Insert => "-- INSERT --",
            EditorMode::Command => "-- COMMAND --",
        };

        let help_text = match self.editor_mode {
            EditorMode::Insert => format!("{} | Esc: Command mode", mode_indicator),
            EditorMode::Command => format!("{} | i: Insert | s: Save | q: Quit | t: Title | c: Content", mode_indicator),
        };

        let help = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center)
            .style(match self.editor_mode {
                EditorMode::Insert => Style::default().fg(Color::Green),
                EditorMode::Command => Style::default().fg(Color::Yellow),
            });

        f.render_widget(help, chunks[2]);
    }

    fn render_view_note(&mut self, f: &mut Frame, area: Rect, note_id: Uuid) {
        if let Some(note) = self.notes.iter().find(|n| n.id == note_id) {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(area);

            let title = note.title.as_deref().unwrap_or("Untitled");
            let title_block = Block::default()
                .borders(Borders::ALL)
                .title("Note");

            let title_widget = Paragraph::new(title)
                .block(title_block)
                .style(Style::default().add_modifier(Modifier::BOLD));

            f.render_widget(title_widget, chunks[0]);

            let content_block = Block::default()
                .borders(Borders::ALL);

            let content = Paragraph::new(note.content.as_str())
                .block(content_block)
                .wrap(Wrap { trim: false });

            f.render_widget(content, chunks[1]);

            let help = Paragraph::new("Esc: Back to home | q: Quit")
                .block(Block::default().borders(Borders::ALL))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));

            f.render_widget(help, chunks[2]);
        }
    }

    fn render_help(&mut self, f: &mut Frame, area: Rect) {
        let help_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Navigation:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("  ↑/↓      Navigate notes"),
            Line::from("  Enter    View selected note"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Actions:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("  a        Add new note"),
            Line::from("  r        Refresh notes list"),
            Line::from("  h        Show this help"),
            Line::from("  q        Quit application"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Modal Editor:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("  i        Enter insert mode"),
            Line::from("  Esc      Enter command mode"),
            Line::from("  s        Save note (command mode)"),
            Line::from("  t        Edit title (command mode)"),
            Line::from("  c        Edit content (command mode)"),
            Line::from("  Tab      Switch fields"),
            Line::from(""),
            Line::from("Press Esc to return to home"),
        ];

        let help_widget = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .alignment(Alignment::Left);

        f.render_widget(help_widget, area);
    }
}

pub fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();
    app.run()
}
