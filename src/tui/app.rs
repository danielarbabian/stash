use std::io;
use std::fs;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Frame, Terminal,
    widgets::ListState,
};
use tui_textarea::TextArea;

use crate::models::Note;
use crate::store;
use super::state::{AppMode, EditorMode, ActiveField};
use super::handlers::InputHandler;
use super::components::Renderer;

pub struct App {
    pub mode: AppMode,
    pub editor_mode: EditorMode,
    pub notes: Vec<Note>,
    pub selected_note: usize,
    pub notes_list_state: ListState,
    pub content_editor: TextArea<'static>,
    pub title_input: String,
    pub active_field: ActiveField,
    pub should_quit: bool,
    pub status_message: Option<String>,
    pub extracted_tags: Vec<String>,
    pub extracted_projects: Vec<String>,
}

impl Default for App {
    fn default() -> App {
        let content_editor = TextArea::default();
        let mut notes_list_state = ListState::default();
        notes_list_state.select(Some(0));

        App {
            mode: AppMode::Home,
            editor_mode: EditorMode::Command,
            notes: Vec::new(),
            selected_note: 0,
            notes_list_state,
            content_editor,
            title_input: String::new(),
            active_field: ActiveField::Content,
            should_quit: false,
            status_message: None,
            extracted_tags: Vec::new(),
            extracted_projects: Vec::new(),
        }
    }
}

impl App {
    pub fn new() -> App {
        let mut app = App::default();
        app.load_existing_notes();
        app
    }

    pub fn load_existing_notes(&mut self) {
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

        self.selected_note = 0;
        if !self.notes.is_empty() {
            self.notes_list_state.select(Some(0));
        } else {
            self.notes_list_state.select(None);
        }
    }

    pub fn next_note(&mut self) {
        if !self.notes.is_empty() {
            self.selected_note = (self.selected_note + 1) % self.notes.len();
            self.notes_list_state.select(Some(self.selected_note));
        }
    }

    pub fn previous_note(&mut self) {
        if !self.notes.is_empty() {
            if self.selected_note == 0 {
                self.selected_note = self.notes.len() - 1;
            } else {
                self.selected_note -= 1;
            }
            self.notes_list_state.select(Some(self.selected_note));
        }
    }

    pub fn update_extracted_metadata(&mut self) {
        let content = self.content_editor.lines().join("\n");
        self.extracted_tags = crate::store::extract_tags(&content);
        self.extracted_projects = crate::store::extract_projects(&content);
    }

    pub fn save_note(&mut self) {
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
                    self.extracted_tags.clear();
                    self.extracted_projects.clear();
                }
                Err(e) => {
                    self.status_message = Some(format!("Error saving note: {}", e));
                }
            }
        } else {
            self.status_message = Some("Cannot save empty note".to_string());
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
                    self.handle_input(key.code, key.modifiers);
                }
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    fn ui(&mut self, f: &mut Frame) {
        self.render(f);
    }
}