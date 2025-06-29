use crossterm::event::{KeyCode, KeyModifiers};

use super::app::App;
use super::state::{AppMode, EditorMode, ActiveField};

pub trait InputHandler {
    fn handle_input(&mut self, key: KeyCode, modifiers: KeyModifiers);
    fn handle_home_input(&mut self, key: KeyCode);
    fn handle_add_note_input(&mut self, key: KeyCode, modifiers: KeyModifiers);
    fn handle_view_note_input(&mut self, key: KeyCode);
    fn handle_help_input(&mut self, key: KeyCode);
}

impl InputHandler for App {
    fn handle_input(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match self.mode {
            AppMode::Home => self.handle_home_input(key),
            AppMode::AddNote => self.handle_add_note_input(key, modifiers),
            AppMode::ViewNote(_) => self.handle_view_note_input(key),
            AppMode::Help => self.handle_help_input(key),
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
            KeyCode::Char('r') => {
                self.load_existing_notes();
                self.status_message = Some("Notes refreshed".to_string());
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
}