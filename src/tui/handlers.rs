use crossterm::event::{KeyCode, KeyModifiers};

use super::app::App;
use super::state::{AppMode, EditorMode, ActiveField, AiState};

pub trait InputHandler {
    fn handle_input(&mut self, key: KeyCode, modifiers: KeyModifiers);
    fn handle_home_input(&mut self, key: KeyCode);
    fn handle_add_note_input(&mut self, key: KeyCode, modifiers: KeyModifiers);
    fn handle_view_note_input(&mut self, key: KeyCode);
    fn handle_help_input(&mut self, key: KeyCode);
    fn handle_settings_input(&mut self, key: KeyCode, modifiers: KeyModifiers);
    fn handle_ai_rewrite_input(&mut self, key: KeyCode);
    fn handle_ai_command_input(&mut self, key: KeyCode, modifiers: KeyModifiers);
}

impl InputHandler for App {
    fn handle_input(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match self.mode.clone() {
            AppMode::Home => self.handle_home_input(key),
            AppMode::AddNote => self.handle_add_note_input(key, modifiers),
            AppMode::ViewNote(_) => self.handle_view_note_input(key),
            AppMode::Help => self.handle_help_input(key),
            AppMode::Settings => self.handle_settings_input(key, modifiers),
            AppMode::AiRewrite { .. } => self.handle_ai_rewrite_input(key),
            AppMode::AiCommand { .. } => self.handle_ai_command_input(key, modifiers),
        }
    }

    fn handle_home_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('a') => {
                self.mode = AppMode::AddNote;
                self.editor_mode = EditorMode::Insert;
                self.active_field = ActiveField::Content;
                self.content_editor = tui_textarea::TextArea::default();
                self.title_input.clear();
                self.extracted_tags.clear();
                self.extracted_projects.clear();
            }
            KeyCode::Char('h') => self.mode = AppMode::Help,
            KeyCode::Char('s') => {
                self.mode = AppMode::Settings;
                self.active_field = ActiveField::ApiKey;
                self.api_key_input.clear();
                if let Some(ref custom_prompt) = self.config.custom_ai_prompt {
                    self.custom_prompt_input = custom_prompt.clone();
                } else {
                    self.custom_prompt_input.clear();
                }
            }
            KeyCode::Char('c') => {
                self.mode = AppMode::AiCommand {
                    natural_input: String::new(),
                    generated_command: None,
                    command_results: None,
                    awaiting_confirmation: false
                };
                self.ai_command_input.clear();
                self.ai_state = AiState::Idle;
            }
            KeyCode::Char('r') => {
                self.load_existing_notes();
                self.status_message = Some("notes refreshed".to_string());
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.previous_note();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.next_note();
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
                                self.update_extracted_metadata();
                            }
                            ActiveField::Title => {
                                match key {
                                    KeyCode::Char(c) => {
                                        self.title_input.push(c);
                                    }
                                    KeyCode::Backspace => {
                                        self.title_input.pop();
                                    }
                                    _ => {}
                                }
                            }
                            ActiveField::ApiKey => {
                                // apikey field should not be active in addnote mode
                            }
                            ActiveField::PromptStyle | ActiveField::CustomPrompt => {
                                // prompt fields should not be active in addnote mode
                            }
                        }
                    }
                }
            }
            EditorMode::Command => {
                match key {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.mode = AppMode::Home;
                        self.editor_mode = EditorMode::Command;
                        self.content_editor = tui_textarea::TextArea::default();
                        self.title_input.clear();
                        self.extracted_tags.clear();
                        self.extracted_projects.clear();
                    }
                    KeyCode::Char('s') => {
                        self.save_note();
                    }
                    KeyCode::Char('r') => {
                        self.start_ai_rewrite_draft();
                    }
                    KeyCode::Char('i') => {
                        self.editor_mode = EditorMode::Insert;
                    }
                    KeyCode::Char('t') => {
                        self.active_field = ActiveField::Title;
                        self.editor_mode = EditorMode::Insert;
                    }
                    KeyCode::Char('c') => {
                        self.active_field = ActiveField::Content;
                        self.editor_mode = EditorMode::Insert;
                    }
                    _ => {}
                }
            }
        }
    }

    fn handle_view_note_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.mode = AppMode::Home;
            }
            KeyCode::Char('r') => {
                if let AppMode::ViewNote(note_id) = self.mode {
                    self.start_ai_rewrite(note_id);
                }
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

    fn handle_settings_input(&mut self, key: KeyCode, _modifiers: KeyModifiers) {
        match key {
            KeyCode::Esc => {
                self.mode = AppMode::Home;
                self.api_key_input.clear();
                self.custom_prompt_input.clear();
            }
            KeyCode::Enter => {
                match self.active_field {
                    ActiveField::ApiKey => {
                        if !self.api_key_input.trim().is_empty() {
                            if let Err(_) = self.set_api_key(self.api_key_input.clone()) {
                                self.status_message = Some("failed to save api key".to_string());
                            } else {
                                if let Err(e) = self.save_prompt_settings() {
                                    self.status_message = Some(e);
                                } else {
                                    self.mode = AppMode::Home;
                                    self.api_key_input.clear();
                                    self.custom_prompt_input.clear();
                                }
                            }
                        }
                    }
                    ActiveField::PromptStyle | ActiveField::CustomPrompt => {
                        if let Err(e) = self.save_prompt_settings() {
                            self.status_message = Some(e);
                        } else {
                            self.status_message = Some("settings saved successfully".to_string());
                            self.mode = AppMode::Home;
                            self.api_key_input.clear();
                            self.custom_prompt_input.clear();
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Tab => {
                self.active_field = match self.active_field {
                    ActiveField::ApiKey => ActiveField::PromptStyle,
                    ActiveField::PromptStyle => ActiveField::CustomPrompt,
                    ActiveField::CustomPrompt => ActiveField::ApiKey,
                    _ => ActiveField::ApiKey,
                };
            }
            KeyCode::Up => {
                if let ActiveField::PromptStyle = self.active_field {
                    self.previous_prompt_style();
                }
            }
            KeyCode::Down => {
                if let ActiveField::PromptStyle = self.active_field {
                    self.next_prompt_style();
                }
            }
            KeyCode::Char(c) => {
                match self.active_field {
                    ActiveField::ApiKey => {
                        self.api_key_input.push(c);
                    }
                    ActiveField::CustomPrompt => {
                        self.custom_prompt_input.push(c);
                    }
                    _ => {}
                }
            }
            KeyCode::Backspace => {
                match self.active_field {
                    ActiveField::ApiKey => {
                        self.api_key_input.pop();
                    }
                    ActiveField::CustomPrompt => {
                        self.custom_prompt_input.pop();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn handle_ai_rewrite_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.reject_ai_rewrite();
            }
            KeyCode::Enter => {
                if let AiState::Success = self.ai_state {
                    self.accept_ai_rewrite();
                }
            }
            _ => {}
        }
    }

    fn handle_ai_command_input(&mut self, key: KeyCode, _modifiers: KeyModifiers) {
        if let AppMode::AiCommand { awaiting_confirmation, .. } = &self.mode {
            if *awaiting_confirmation {
                // We're in confirmation mode - waiting for user to approve/reject the generated command
                match key {
                    KeyCode::Enter => {
                        self.execute_ai_command();
                    }
                    KeyCode::Esc => {
                        self.cancel_ai_command();
                    }
                    _ => {}
                }
            } else if matches!(self.ai_state, AiState::Processing) {
                // We're processing - only allow cancel
                match key {
                    KeyCode::Esc => {
                        self.cancel_ai_command();
                    }
                    _ => {}
                }
            } else {
                // We're in input mode - typing the natural language query
                match key {
                    KeyCode::Esc => {
                        self.cancel_ai_command();
                    }
                    KeyCode::Enter => {
                        if !self.ai_command_input.trim().is_empty() {
                            let input = self.ai_command_input.clone();
                            self.start_ai_command(input);
                        }
                    }
                    KeyCode::Char(c) => {
                        self.ai_command_input.push(c);
                    }
                    KeyCode::Backspace => {
                        self.ai_command_input.pop();
                    }
                    _ => {}
                }
            }
        }
    }
}