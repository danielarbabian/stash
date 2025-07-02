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
use crate::config::Config;
use crate::ai::AiClient;
use super::state::{AppMode, EditorMode, ActiveField, AiState};
use super::handlers::InputHandler;
use super::components::Renderer;
use tokio::sync::mpsc;

pub struct App {
    pub mode: AppMode,
    pub editor_mode: EditorMode,
    pub notes: Vec<Note>,

    pub all_notes: Vec<Note>,
    pub selected_note: usize,
    pub notes_list_state: ListState,
    pub content_editor: TextArea<'static>,
    pub title_input: String,
    pub active_field: ActiveField,
    pub should_quit: bool,
    pub status_message: Option<String>,
    pub extracted_tags: Vec<String>,
    pub extracted_projects: Vec<String>,
    pub config: Config,
    pub ai_client: Option<AiClient>,
    pub ai_state: AiState,
    pub api_key_input: String,
    pub ai_result_receiver: Option<mpsc::UnboundedReceiver<Result<String, String>>>,
    pub prompt_style_index: usize,
    pub custom_prompt_input: String,
    pub search_input: String,
    pub tag_filter_input: String,
    pub project_filter_input: String,
    pub current_search: Option<String>,
    pub current_tag_filter: Option<String>,
    pub current_project_filter: Option<String>,
    pub deletion_preference: DeletionType,
}

#[derive(Debug, Clone)]
pub enum DeletionType {
    Soft,
    Hard,
}

impl Default for App {
    fn default() -> App {
        let content_editor = TextArea::default();
        let mut notes_list_state = ListState::default();
        notes_list_state.select(Some(0));

        let config = Config::load().unwrap_or_default();
        let ai_client = AiClient::new().ok();

        // find the index of the current prompt style
        let styles = App::get_prompt_styles();
        let prompt_style_index = styles.iter()
            .position(|(key, _)| *key == config.ai_prompt_style)
            .unwrap_or(0);

        App {
            mode: AppMode::Home,
            editor_mode: EditorMode::Command,
            notes: Vec::new(),

            all_notes: Vec::new(),
            selected_note: 0,
            notes_list_state,
            content_editor,
            title_input: String::new(),
            active_field: ActiveField::Content,
            should_quit: false,
            status_message: None,
            extracted_tags: Vec::new(),
            extracted_projects: Vec::new(),
            config,
            ai_client,
            ai_state: AiState::Idle,
            api_key_input: String::new(),
            ai_result_receiver: None,
            prompt_style_index,
            custom_prompt_input: String::new(),
            search_input: String::new(),
            tag_filter_input: String::new(),
            project_filter_input: String::new(),
            current_search: None,
            current_tag_filter: None,
            current_project_filter: None,
            deletion_preference: DeletionType::Soft,
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
        self.all_notes.clear();
        if let Some(home) = dirs::home_dir() {
            let notes_dir = home.join(".stash").join("notes");
            if let Ok(entries) = fs::read_dir(notes_dir) {
                for entry in entries.flatten() {
                    if let Some(extension) = entry.path().extension() {
                        if extension == "md" {
                            match Note::load_from_file(entry.path()) {
                                Ok(note) => {
                                    self.all_notes.push(note);
                                }
                                Err(e) => {
                                    eprintln!("Failed to load note {:?}: {}", entry.path(), e);
                                }
                            }
                        }
                    }
                }
                self.all_notes.sort_by(|a, b| b.created.cmp(&a.created));
            }
        }

        self.apply_filters();
    }

    pub fn apply_filters(&mut self) {
        self.notes = self.all_notes.clone();

        self.notes.retain(|note| {
            !note.tags.contains(&"deleted".to_string())
        });

        if let Some(ref search_term) = self.current_search {
            if !search_term.trim().is_empty() {
                self.notes.retain(|note| {
                    let content_match = note.content.to_lowercase().contains(&search_term.to_lowercase());
                    let title_match = note.title.as_ref()
                        .map(|t| t.to_lowercase().contains(&search_term.to_lowercase()))
                        .unwrap_or(false);
                    content_match || title_match
                });
            }
        }

        if let Some(ref tag_filter) = self.current_tag_filter {
            if !tag_filter.trim().is_empty() {
                self.notes.retain(|note| {
                    note.tags.iter().any(|tag| tag.to_lowercase().contains(&tag_filter.to_lowercase()))
                });
            }
        }

        if let Some(ref project_filter) = self.current_project_filter {
            if !project_filter.trim().is_empty() {
                self.notes.retain(|note| {
                    let projects = store::extract_projects(&note.content);
                    projects.iter().any(|project| project.to_lowercase().contains(&project_filter.to_lowercase()))
                });
            }
        }

        self.selected_note = 0;
        if !self.notes.is_empty() {
            self.notes_list_state.select(Some(0));
        } else {
            self.notes_list_state.select(None);
        }
    }

    pub fn clear_filters(&mut self) {
        self.current_search = None;
        self.current_tag_filter = None;
        self.current_project_filter = None;
        self.search_input.clear();
        self.tag_filter_input.clear();
        self.project_filter_input.clear();
        self.apply_filters();
        self.status_message = Some("filters cleared".to_string());
    }

    pub fn confirm_delete_current_note(&mut self) {
        if !self.notes.is_empty() && self.selected_note < self.notes.len() {
            let note_id = self.notes[self.selected_note].id;
            self.mode = AppMode::DeleteConfirm { note_id };
            self.active_field = ActiveField::DeleteOption;
        }
    }

    pub fn soft_delete_note(&mut self, note_id: uuid::Uuid) {
        if let Some(note) = self.all_notes.iter_mut().find(|n| n.id == note_id) {
            if !note.tags.contains(&"deleted".to_string()) {
                note.tags.push("deleted".to_string());
                note.updated = Some(chrono::Utc::now());

                if let Some(home) = dirs::home_dir() {
                    let notes_dir = home.join(".stash").join("notes");
                    let file_path = notes_dir.join(format!("{}.md", note.id));
                    if let Err(e) = note.save_to_file(&file_path) {
                        self.status_message = Some(format!("error saving note: {}", e));
                        return;
                    }
                }

                self.status_message = Some("note moved to trash (soft delete)".to_string());
                self.load_existing_notes();
            }
        }
        self.mode = AppMode::Home;
    }

    pub fn hard_delete_note(&mut self, note_id: uuid::Uuid) {
        if let Some(home) = dirs::home_dir() {
            let notes_dir = home.join(".stash").join("notes");
            let filename = format!("{}.md", note_id);
            let file_path = notes_dir.join(filename);

            match fs::remove_file(&file_path) {
                Ok(()) => {
                    self.status_message = Some("note permanently deleted".to_string());
                    self.load_existing_notes();
                }
                Err(e) => {
                    self.status_message = Some(format!("error deleting note: {}", e));
                }
            }
        }
        self.mode = AppMode::Home;
    }

    pub fn toggle_deletion_preference(&mut self) {
        self.deletion_preference = match self.deletion_preference {
            DeletionType::Soft => DeletionType::Hard,
            DeletionType::Hard => DeletionType::Soft,
        };
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

    pub fn start_new_note(&mut self) {
        self.mode = AppMode::AddNote;
        self.editor_mode = EditorMode::Insert;
        self.active_field = ActiveField::Content;
        self.content_editor = tui_textarea::TextArea::default();
        self.title_input.clear();
        self.extracted_tags.clear();
        self.extracted_projects.clear();
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
                    self.status_message = Some("note saved successfully".to_string());
                    self.load_existing_notes();
                    self.mode = AppMode::Home;
                    self.editor_mode = EditorMode::Command;
                    self.content_editor = TextArea::default();
                    self.title_input.clear();
                    self.extracted_tags.clear();
                    self.extracted_projects.clear();
                }
                Err(e) => {
                    self.status_message = Some(format!("error saving note: {}", e));
                }
            }
        } else {
            self.status_message = Some("cannot save empty note".to_string());
        }
    }

    pub fn start_edit_note(&mut self, note_id: uuid::Uuid) {
        if let Some(note) = self.notes.iter().find(|n| n.id == note_id) {
            self.mode = AppMode::EditNote(note_id);
            self.editor_mode = EditorMode::Insert;
            self.active_field = ActiveField::Content;

            self.content_editor = tui_textarea::TextArea::from(note.content.lines().collect::<Vec<_>>());
            self.title_input = note.title.clone().unwrap_or_default();

            self.update_extracted_metadata();
            self.status_message = Some("editing note".to_string());
        }
    }

    pub fn save_edited_note(&mut self) {
        if let AppMode::EditNote(note_id) = self.mode {
            let content = self.content_editor.lines().join("\n");

            if !content.trim().is_empty() {
                if let Some(note) = self.notes.iter_mut().find(|n| n.id == note_id) {
                    note.content = content;
                    note.title = if self.title_input.is_empty() {
                        None
                    } else {
                        Some(self.title_input.clone())
                    };
                    note.updated = Some(chrono::Utc::now());
                    note.tags = crate::store::extract_tags(&note.content);
                    note.projects = crate::store::extract_projects(&note.content);

                    if let Some(home) = dirs::home_dir() {
                        let notes_dir = home.join(".stash").join("notes");
                        let file_path = notes_dir.join(format!("{}.md", note.id));

                        match note.save_to_file(&file_path) {
                            Ok(()) => {
                                self.status_message = Some("note updated successfully".to_string());
                                self.load_existing_notes();
                                self.mode = AppMode::ViewNote(note_id);
                                self.editor_mode = EditorMode::Command;
                                self.content_editor = TextArea::default();
                                self.title_input.clear();
                                self.extracted_tags.clear();
                                self.extracted_projects.clear();
                            }
                            Err(e) => {
                                self.status_message = Some(format!("error saving note: {}", e));
                            }
                        }
                    }
                }
            } else {
                self.status_message = Some("cannot save empty note".to_string());
            }
        }
    }

    pub fn set_api_key(&mut self, api_key: String) -> Result<(), String> {
        match self.config.set_api_key(api_key) {
            Ok(()) => {
                self.ai_client = AiClient::new().ok();
                self.status_message = Some("api key saved successfully".to_string());
                Ok(())
            }
            Err(e) => {
                self.status_message = Some(format!("error saving api key: {}", e));
                Err(e.to_string())
            }
        }
    }

    pub fn start_ai_rewrite(&mut self, note_id: uuid::Uuid) {
        if let Some(note) = self.notes.iter().find(|n| n.id == note_id) {
            if let Some(ai_client) = &self.ai_client {
                if !ai_client.is_configured() {
                    self.status_message = Some("please configure your openai api key first (press 's' for settings)".to_string());
                    return;
                }

                self.ai_state = AiState::Processing;
                self.mode = AppMode::AiRewrite {
                    original_note_id: note_id,
                    rewritten_content: None
                };

                let (tx, rx) = mpsc::unbounded_channel();
                self.ai_result_receiver = Some(rx);

                let note_clone = note.clone();
                let ai_client = match AiClient::new() {
                    Ok(client) => client,
                    Err(e) => {
                        self.ai_state = AiState::Error(format!("Failed to create AI client: {}", e));
                        return;
                    }
                };

                tokio::spawn(async move {
                    let result = match ai_client.rewrite_note(&note_clone).await {
                        Ok(content) => Ok(content),
                        Err(e) => Err(e.to_string()),
                    };
                    let _ = tx.send(result);
                });
            } else {
                self.status_message = Some("ai client not available. please check your configuration.".to_string());
            }
        }
    }

    pub fn check_ai_result(&mut self) {
        if let Some(receiver) = &mut self.ai_result_receiver {
            if let Ok(result) = receiver.try_recv() {
                match result {
                    Ok(rewritten_content) => {
                        if let AppMode::AiRewrite { original_note_id, .. } = self.mode {
                            self.mode = AppMode::AiRewrite {
                                original_note_id,
                                rewritten_content: Some(rewritten_content)
                            };
                            self.ai_state = AiState::Success;
                        }
                    }
                    Err(error) => {
                        self.ai_state = AiState::Error(error);
                    }
                }
                self.ai_result_receiver = None;
            }
        }
    }

    pub fn accept_ai_rewrite(&mut self) {
        if let AppMode::AiRewrite { original_note_id, rewritten_content: Some(ref content) } = &self.mode {
            if *original_note_id == uuid::Uuid::nil() {
                // this is a draft rewrite - update the content editor and go back to AddNote mode
                self.content_editor = tui_textarea::TextArea::from(content.lines().collect::<Vec<_>>());
                self.update_extracted_metadata();

                self.status_message = Some("draft updated with ai rewrite".to_string());
                self.mode = AppMode::AddNote;
                self.ai_state = AiState::Idle;
            } else {
                // this is a saved note rewrite - update the saved note
                if let Some(note) = self.notes.iter_mut().find(|n| n.id == *original_note_id) {
                    note.content = content.clone();
                    note.updated = Some(chrono::Utc::now());

                    if let Some(home) = dirs::home_dir() {
                        let notes_dir = home.join(".stash").join("notes");
                        let file_path = notes_dir.join(format!("{}.md", note.id));
                        if let Err(e) = note.save_to_file(&file_path) {
                            self.status_message = Some(format!("error saving note: {}", e));
                            return;
                        }
                    }

                    self.status_message = Some("note updated with ai rewrite".to_string());
                    self.mode = AppMode::ViewNote(*original_note_id);
                    self.ai_state = AiState::Idle;
                }
            }
        }
    }

    pub fn reject_ai_rewrite(&mut self) {
        if let AppMode::AiRewrite { original_note_id, .. } = self.mode {
            if original_note_id == uuid::Uuid::nil() {
                self.mode = AppMode::AddNote;
            } else {
                self.mode = AppMode::ViewNote(original_note_id);
            }
            self.ai_state = AiState::Idle;
            self.status_message = Some("ai rewrite rejected".to_string());
        }
    }

    pub fn start_ai_rewrite_draft(&mut self) {
        let current_content = self.content_editor.lines().join("\n");

        if current_content.trim().is_empty() {
            self.status_message = Some("cannot rewrite empty content".to_string());
            return;
        }

        if let Some(ai_client) = &self.ai_client {
            if !ai_client.is_configured() {
                self.status_message = Some("please configure your openai api key first (press 's' for settings)".to_string());
                return;
            }

            self.ai_state = AiState::Processing;
            self.mode = AppMode::AiRewrite {
                original_note_id: uuid::Uuid::nil(), // use nil UUID to indicate this is a draft
                rewritten_content: None
            };

            let (tx, rx) = mpsc::unbounded_channel();
            self.ai_result_receiver = Some(rx);

            // create a temporary note for AI processing
            let temp_note = Note {
                id: uuid::Uuid::nil(),
                title: if self.title_input.is_empty() { None } else { Some(self.title_input.clone()) },
                tags: self.extracted_tags.clone(),
                projects: self.extracted_projects.clone(),
                links_to: Vec::new(),
                created: chrono::Utc::now(),
                updated: None,
                source: crate::models::NoteSource::UI,
                content: current_content,
            };

            let ai_client = match AiClient::new() {
                Ok(client) => client,
                Err(e) => {
                    self.ai_state = AiState::Error(format!("Failed to create AI client: {}", e));
                    return;
                }
            };

            tokio::spawn(async move {
                let result = match ai_client.rewrite_note(&temp_note).await {
                    Ok(content) => Ok(content),
                    Err(e) => Err(e.to_string()),
                };
                let _ = tx.send(result);
            });
        } else {
            self.status_message = Some("ai client not available. please check your configuration.".to_string());
        }
    }

    pub fn get_prompt_styles() -> Vec<(&'static str, &'static str)> {
        vec![
            ("professional", "Professional & Polished"),
            ("casual", "Casual & Conversational"),
            ("concise", "Concise & Brief"),
            ("detailed", "Detailed & Expanded"),
            ("technical", "Technical & Precise"),
            ("simple", "Simple & Clear"),
            ("custom", "Custom Prompt"),
        ]
    }

    pub fn next_prompt_style(&mut self) {
        let styles = Self::get_prompt_styles();
        self.prompt_style_index = (self.prompt_style_index + 1) % styles.len();
    }

    pub fn previous_prompt_style(&mut self) {
        let styles = Self::get_prompt_styles();
        if self.prompt_style_index == 0 {
            self.prompt_style_index = styles.len() - 1;
        } else {
            self.prompt_style_index -= 1;
        }
    }

    pub fn save_prompt_settings(&mut self) -> Result<(), String> {
        let styles = Self::get_prompt_styles();
        let selected_style = styles[self.prompt_style_index].0.to_string();

        if let Err(e) = self.config.set_prompt_style(selected_style.clone()) {
            return Err(format!("Failed to save prompt style: {}", e));
        }

        if selected_style == "custom" && !self.custom_prompt_input.trim().is_empty() {
            if let Err(e) = self.config.set_custom_prompt(Some(self.custom_prompt_input.clone())) {
                return Err(format!("Failed to save custom prompt: {}", e));
            }
        } else if selected_style != "custom" {
            if let Err(e) = self.config.set_custom_prompt(None) {
                return Err(format!("Failed to clear custom prompt: {}", e));
            }
        }

        // reload AI client with updated config
        self.ai_client = AiClient::new().ok();
        Ok(())
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
                    self.check_ai_result();
            terminal.draw(|f| self.ui(f))?;

            if crossterm::event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_input(key.code, key.modifiers);
                    }
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