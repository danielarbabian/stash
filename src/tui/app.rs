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
    pub ai_command_input: String,
    pub ai_command_receiver: Option<mpsc::UnboundedReceiver<Result<String, String>>>,
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
            ai_command_input: String::new(),
            ai_command_receiver: None,
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
            self.check_ai_command_result();
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

    pub fn start_ai_command(&mut self, input: String) {
        if let Some(ai_client) = &self.ai_client {
            if !ai_client.is_configured() {
                self.status_message = Some("please configure your openai api key first (press 's' for settings)".to_string());
                return;
            }

            self.ai_state = AiState::Processing;
            self.mode = AppMode::AiCommand {
                natural_input: input.clone(),
                generated_command: None,
                command_results: None,
                awaiting_confirmation: false
            };

            let (tx, rx) = mpsc::unbounded_channel();
            self.ai_command_receiver = Some(rx);

            let ai_client = match AiClient::new() {
                Ok(client) => client,
                Err(e) => {
                    self.ai_state = AiState::Error(format!("Failed to create AI client: {}", e));
                    return;
                }
            };

            tokio::spawn(async move {
                let result = match ai_client.parse_natural_command(&input).await {
                    Ok(command) => Ok(command),
                    Err(e) => Err(e.to_string()),
                };
                let _ = tx.send(result);
            });
        } else {
            self.status_message = Some("ai client not available. please check your configuration.".to_string());
        }
    }

    pub fn check_ai_command_result(&mut self) {
        if let Some(receiver) = &mut self.ai_command_receiver {
            if let Ok(result) = receiver.try_recv() {
                match result {
                    Ok(generated_args) => {
                        if let AppMode::AiCommand { natural_input, .. } = &self.mode {
                            self.mode = AppMode::AiCommand {
                                natural_input: natural_input.clone(),
                                generated_command: Some(generated_args),
                                command_results: None,
                                awaiting_confirmation: true
                            };
                            self.ai_state = AiState::Success;
                        }
                    }
                    Err(error) => {
                        self.ai_state = AiState::Error(error);
                    }
                }
                self.ai_command_receiver = None;
            }
        }
    }

    pub fn execute_ai_command(&mut self) {
        if let AppMode::AiCommand { natural_input, generated_command: Some(ref command), .. } = self.mode.clone() {
            // parse the generated arguments (no longer expect "stash search" prefix)
            let args: Vec<&str> = command.split_whitespace().collect();
            let results = self.execute_search_command(&args);

            self.mode = AppMode::AiCommand {
                natural_input,
                generated_command: Some(format!("stash search {}", command)),
                command_results: Some(results),
                awaiting_confirmation: false
            };
        }
    }

    fn execute_search_command(&mut self, args: &[&str]) -> Vec<String> {
        use crate::store::SearchOptions;

        let mut query = String::new();
        let mut filter_tags = None;
        let mut filter_projects = None;
        let mut list_tags = false;
        let mut list_projects = false;
        let mut case_sensitive = false;

        let mut i = 0;
        while i < args.len() {
            match args[i] {
                "--list-tags" => list_tags = true,
                "--list-projects" => list_projects = true,
                "--case-sensitive" => {
                    case_sensitive = true;
                    if i + 1 < args.len() {
                        i += 1;
                        query = args[i].trim_matches('"').to_string();
                    }
                }
                "--filter-tags" => {
                    if i + 1 < args.len() {
                        i += 1;
                        filter_tags = Some(args[i].to_string());
                    }
                }
                "--filter-projects" => {
                    if i + 1 < args.len() {
                        i += 1;
                        filter_projects = Some(args[i].to_string());
                    }
                }
                arg => {
                    if !query.is_empty() {
                        query.push(' ');
                    }
                    query.push_str(arg.trim_matches('"'));
                }
            }
            i += 1;
        }

        let search_options = SearchOptions {
            query,
            filter_tags,
            filter_projects,
            list_tags,
            list_projects,
            case_sensitive,
        };

        match crate::store::search_notes_return_results(search_options) {
            Ok(results) => {
                if results.is_empty() {
                    vec!["No notes found matching your query.".to_string()]
                } else {
                    let mut formatted_results = vec![format!("🔍 Found {} note(s):", results.len())];

                    for (i, result) in results.iter().enumerate() {
                        let title = result.note.title.as_deref().unwrap_or("Untitled");

                        let mut note_info = format!("\n{}. {}", i + 1, title);

                        if !result.content_snippets.is_empty() {
                            note_info.push_str("\n   Content matches:");
                            for snippet in &result.content_snippets {
                                note_info.push_str(&format!("\n     {}", snippet));
                            }
                        }

                        note_info.push_str(&format!("\n   Created: {}", result.note.created.format("%Y-%m-%d %H:%M")));

                        if !result.note.tags.is_empty() {
                            note_info.push_str(&format!("\n   All tags: {}",
                                result.note.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ")));
                        }

                        let content_projects = crate::store::extract_projects(&result.note.content);
                        if !content_projects.is_empty() {
                            note_info.push_str(&format!("\n   All projects: {}",
                                content_projects.iter().map(|p| format!("+{}", p)).collect::<Vec<_>>().join(" ")));
                        }

                        formatted_results.push(note_info);
                    }

                    formatted_results
                }
            }
            Err(e) => {
                vec![format!("Search error: {}", e)]
            }
        }
    }

    pub fn cancel_ai_command(&mut self) {
        self.mode = AppMode::Home;
        self.ai_state = AiState::Idle;
        self.ai_command_input.clear();
        self.ai_command_receiver = None;
    }
}