use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use uuid::Uuid;

use super::app::App;
use super::state::{AppMode, EditorMode, ActiveField, AiState};

pub trait Renderer {
    fn render(&mut self, f: &mut Frame);
    fn render_home(&mut self, f: &mut Frame, area: Rect);
    fn render_add_note(&mut self, f: &mut Frame, area: Rect);
    fn render_view_note(&mut self, f: &mut Frame, area: Rect, note_id: Uuid);
    fn render_help(&mut self, f: &mut Frame, area: Rect);
    fn render_settings(&mut self, f: &mut Frame, area: Rect);
    fn render_ai_rewrite(&mut self, f: &mut Frame, area: Rect, original_note_id: Uuid, rewritten_content: &Option<String>);
}

impl Renderer for App {
    fn render(&mut self, f: &mut Frame) {
        let area = f.area();

        match self.mode.clone() {
            AppMode::Home => self.render_home(f, area),
            AppMode::AddNote => self.render_add_note(f, area),
            AppMode::ViewNote(note_id) => self.render_view_note(f, area, note_id),
            AppMode::Help => self.render_help(f, area),
            AppMode::Settings => self.render_settings(f, area),
            AppMode::AiRewrite { original_note_id, rewritten_content } => {
                self.render_ai_rewrite(f, area, original_note_id, &rewritten_content)
            }
        }

        if let Some(ref message) = self.status_message {
            let status_area = Rect {
                x: area.x,
                y: area.height.saturating_sub(1),
                width: area.width,
                height: 1,
            };

            let status_widget = Paragraph::new(message.as_str())
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center);

            f.render_widget(status_widget, status_area);

            self.status_message = None;
        }
    }

    fn render_home(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(20),
                Constraint::Min(0),
            ])
            .split(area);

        // why is this the hardest part of the project?
        // TODO: make this smaller without destroying it
        let ascii_art = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(r#"                   ,----,                                    "#, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(r#"                 ,/   .`|                               ,--, "#, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled(r#"  .--.--.      ,`   .'  : ,---,       .--.--.         ,--.'| "#, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled(r#" /  /    '.  ;    ;     /'  .' \     /  /    '.    ,--,  | : "#, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled(r#"|  :  /`. /.'___,/    ,'/  ;    '.  |  :  /`. / ,---.'|  : ' "#, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled(r#";  |  |--` |    :     |:  :       \ ;  |  |--`  |   | : _' | "#, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled(r#"|  :  ;_   ;    |.';  ;:  |   /\   \|  :  ;_    :   : |.'  | "#, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled(r#" \  \    `.`----'  |  ||  :  ' ;.   :\  \    `. |   ' '  ; : "#, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled(r#"  `----.   \   '   :  ;|  |  ;/  \   \`----.   \'   |  .'. | "#, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled(r#"  __ \  \  |   |   |  |'  :  | \  \ ,'__ \  \  ||   | :  | ' "#, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled(r#" /  /`--'  /   '   :  ||  |  '  '--' /  /`--'  /'   : |  : ; "#, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled(r#"'--'.     /    ;   |.' |  :  :      '--'.     / |   | '  ,/  "#, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled(r#"  `--'---'     '---'   |  | ,'        `--'---'  ;   : ;--'   "#, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled(r#"                       `--''                    |   ,/       "#, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled(r#"                                                '---'        "#, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("a", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" add   "),
                Span::styled("h", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" help   "),
                Span::styled("s", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" settings   "),
                Span::styled("r", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" refresh   "),
                Span::styled("q", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" quit"),
            ]),
        ];

        let title_widget = Paragraph::new(ascii_art)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);

        f.render_widget(title_widget, chunks[0]);

        let notes_block = Block::default()
            .borders(Borders::ALL)
            .title(format!("notes ({}) - scroll with ↑↓ or j/k", self.notes.len()));

        if self.notes.is_empty() {
            let empty_message = Paragraph::new("no notes found. press 'a' to create your first note!")
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

                    let is_selected = i == self.selected_note;

                    let title_style = if is_selected {
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().add_modifier(Modifier::BOLD)
                    };

                    let preview_style = if is_selected {
                        Style::default().fg(Color::White)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };

                    let tags_style = if is_selected {
                        Style::default().fg(Color::LightCyan)
                    } else {
                        Style::default().fg(Color::Blue)
                    };

                    let projects_style = if is_selected {
                        Style::default().fg(Color::LightGreen)
                    } else {
                        Style::default().fg(Color::Green)
                    };

                    let mut lines = vec![
                        Line::from(vec![
                            Span::styled(format!("▶ {}", title), title_style),
                        ]),
                        Line::from(vec![
                            Span::styled(format!("  {}", preview), preview_style),
                        ]),
                    ];

                    if !note.tags.is_empty() {
                        let tags_text = note.tags.iter()
                            .map(|tag| format!("#{}", tag))
                            .collect::<Vec<_>>()
                            .join(" ");
                        lines.push(Line::from(vec![
                            Span::styled(format!("  🏷️  {}", tags_text), tags_style),
                        ]));
                    }

                    let projects = crate::store::extract_projects(&note.content);
                    if !projects.is_empty() {
                        let projects_text = projects.iter()
                            .map(|proj| format!("+{}", proj))
                            .collect::<Vec<_>>()
                            .join(" ");
                        lines.push(Line::from(vec![
                            Span::styled(format!("  📁 {}", projects_text), projects_style),
                        ]));
                    }

                    lines.push(Line::from(""));

                    ListItem::new(lines)
                })
                .collect();

            let list = List::new(items)
                .block(notes_block)
                .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD))
                .highlight_symbol("► ");

            f.render_stateful_widget(list, chunks[1], &mut self.notes_list_state);
        }
    }

    fn render_add_note(&mut self, f: &mut Frame, area: Rect) {
        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ])
            .split(area);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(main_layout[0]);

        let title_active = matches!(self.active_field, ActiveField::Title);
        let title_style = if title_active {
            match self.editor_mode {
                EditorMode::Insert => Style::default().fg(Color::Cyan),
                EditorMode::Command => Style::default().fg(Color::Yellow),
            }
        } else {
            Style::default()
        };

        let title_block = Block::default()
            .borders(Borders::ALL)
            .title("title (t to edit)")
            .style(title_style);

        let title_input = Paragraph::new(self.title_input.as_str())
            .block(title_block);

        f.render_widget(title_input, left_chunks[0]);

        let content_active = matches!(self.active_field, ActiveField::Content);
        let content_style = if content_active {
            match self.editor_mode {
                EditorMode::Insert => Style::default().fg(Color::Cyan),
                EditorMode::Command => Style::default().fg(Color::Yellow),
            }
        } else {
            Style::default()
        };

        let content_block = Block::default()
            .borders(Borders::ALL)
            .title("content (c to edit)")
            .style(content_style);

        self.content_editor.set_block(content_block);
        f.render_widget(&self.content_editor, left_chunks[1]);

        let mode_text = match self.editor_mode {
            EditorMode::Insert => "INSERT",
            EditorMode::Command => "COMMAND",
        };

        let mode_style = match self.editor_mode {
            EditorMode::Insert => Style::default().fg(Color::Green),
            EditorMode::Command => Style::default().fg(Color::Yellow),
        };

        let status_text = format!(
            "mode: {} | s:save | r:ai-rewrite | q:quit | i:insert | t:title | c:content | Esc:command",
            mode_text
        );

        let status_widget = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL))
            .style(mode_style);

        f.render_widget(status_widget, left_chunks[2]);

        self.render_metadata_preview(f, main_layout[1]);
    }

    fn render_view_note(&mut self, f: &mut Frame, area: Rect, note_id: Uuid) {
        if let Some(note) = self.notes.iter().find(|n| n.id == note_id) {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(5),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(area);

            let title = note.title.as_deref().unwrap_or("Untitled");
            let header_lines = vec![
                Line::from(vec![
                    Span::styled(title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled(format!("Created: {}", note.created.format("%Y-%m-%d %H:%M")), Style::default().fg(Color::DarkGray)),
                ]),
            ];

            let header_widget = Paragraph::new(header_lines)
                .block(Block::default().borders(Borders::ALL).title("Note Details"))
                .alignment(Alignment::Left);

            f.render_widget(header_widget, chunks[0]);

            let content_widget = Paragraph::new(note.content.as_str())
                .block(Block::default().borders(Borders::ALL).title("Content"))
                .wrap(Wrap { trim: true });

            f.render_widget(content_widget, chunks[1]);

            let help_text = "Press 'r' for AI rewrite • 'q' or Esc to go back";
            let help_widget = Paragraph::new(help_text)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);

            f.render_widget(help_widget, chunks[2]);
        }
    }

    fn render_help(&mut self, f: &mut Frame, area: Rect) {
        let help_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("🚀 stash help", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from("📝 adding notes:"),
            Line::from("  • Use #tagname for tags (e.g., #rust #webdev)"),
            Line::from("  • Use +projectname for projects (e.g., +myapp +backend)"),
            Line::from("  • Mix tags and projects in your content naturally"),
            Line::from(""),
            Line::from("🔍 search features:"),
            Line::from("  • stash search \"query\" - Basic search"),
            Line::from("  • stash search \"#rust +webapp\" - Tag and project search"),
            Line::from("  • stash search \"#rust -#old\" - Exclude tags"),
            Line::from("  • stash search --list-tags - See all tags"),
            Line::from("  • stash search --list-projects - See all projects"),
            Line::from(""),
            Line::from("⌨️  tui controls:"),
            Line::from("  Home: a=add, h=help, r=refresh, ↑↓/jk=navigate, q=quit"),
            Line::from("  Add Note: t=edit title, c=edit content, s=save, q=quit"),
            Line::from("  Editor: i=insert mode, Esc=command mode"),
            Line::from(""),
            Line::from("💡 the right panel shows live tag/project preview!"),
            Line::from(""),
        ];

        let help_widget = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("help"))
            .alignment(Alignment::Left);

        f.render_widget(help_widget, area);
    }

    fn render_settings(&mut self, f: &mut Frame, area: Rect) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),
                Constraint::Length(10),
                Constraint::Length(12),
                Constraint::Min(0),
            ])
            .split(area);

        // Header section
        let title_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("⚙️  ai configuration", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("Configure your OpenAI settings and rewrite preferences"),
        ];

        let title_widget = Paragraph::new(title_lines)
            .block(Block::default().borders(Borders::ALL).title("settings"))
            .alignment(Alignment::Center);

        f.render_widget(title_widget, main_chunks[0]);

        // API Key section
        let api_key_display = if self.api_key_input.is_empty() {
            if self.config.has_api_key() {
                "••••••••••••••••••••••••••••••••••••••••".to_string()
            } else {
                "Not configured".to_string()
            }
        } else {
            "•".repeat(self.api_key_input.len())
        };

        let api_key_style = if matches!(self.active_field, ActiveField::ApiKey) {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let status_text = if self.config.has_api_key() {
            "✅ API Key configured"
        } else {
            "❌ No API Key configured"
        };

        let api_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("OpenAI API Key: ", Style::default().fg(Color::White)),
                Span::styled(api_key_display, api_key_style),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::White)),
                Span::styled(status_text, if self.config.has_api_key() {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                }),
            ]),
            Line::from(""),
            Line::from("Get your API key from: https://platform.openai.com/account/api-keys"),
        ];

        let api_block_style = if matches!(self.active_field, ActiveField::ApiKey) {
            Block::default().borders(Borders::ALL).title("🔑 API Configuration").style(Style::default().fg(Color::Yellow))
        } else {
            Block::default().borders(Borders::ALL).title("🔑 API Configuration")
        };

        let api_widget = Paragraph::new(api_lines).block(api_block_style);

        f.render_widget(api_widget, main_chunks[1]);

        // prompt style section
        let styles = Self::get_prompt_styles();
        let current_style_name = styles[self.prompt_style_index].1;
        let current_style_key = styles[self.prompt_style_index].0;

        let prompt_style_style = if matches!(self.active_field, ActiveField::PromptStyle) {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let mut prompt_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Rewrite Style: ", Style::default().fg(Color::White)),
                Span::styled(current_style_name, prompt_style_style),
            ]),
            Line::from(""),
        ];

        // show style options
        for (i, (_, name)) in styles.iter().enumerate() {
            let style = if i == self.prompt_style_index {
                if matches!(self.active_field, ActiveField::PromptStyle) {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                }
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let prefix = if i == self.prompt_style_index { "► " } else { "  " };
            prompt_lines.push(Line::from(vec![
                Span::styled(format!("{}{}", prefix, name), style),
            ]));
        }

        let prompt_block_style = if matches!(self.active_field, ActiveField::PromptStyle) {
            Block::default().borders(Borders::ALL).title("🎨 Rewrite Style").style(Style::default().fg(Color::Yellow))
        } else {
            Block::default().borders(Borders::ALL).title("🎨 Rewrite Style")
        };

        let prompt_widget = Paragraph::new(prompt_lines).block(prompt_block_style);

        f.render_widget(prompt_widget, main_chunks[2]);

        // custom prompt section (only show if custom is selected)
        if current_style_key == "custom" {
            let custom_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(6),
                    Constraint::Min(0),
                ])
                .split(main_chunks[3]);

            let custom_prompt_display = if self.custom_prompt_input.is_empty() {
                if let Some(ref custom) = self.config.custom_ai_prompt {
                    custom.clone()
                } else {
                    "Enter your custom rewrite instructions...".to_string()
                }
            } else {
                self.custom_prompt_input.clone()
            };

            let custom_style = if matches!(self.active_field, ActiveField::CustomPrompt) {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };

            let custom_lines = vec![
                Line::from(""),
                Line::from("Custom Instructions:"),
                Line::from(""),
                Line::from(vec![
                    Span::styled(custom_prompt_display, custom_style),
                ]),
            ];

            let custom_block_style = if matches!(self.active_field, ActiveField::CustomPrompt) {
                Block::default().borders(Borders::ALL).title("✏️ Custom Prompt").style(Style::default().fg(Color::Yellow))
            } else {
                Block::default().borders(Borders::ALL).title("✏️ Custom Prompt")
            };

            let custom_widget = Paragraph::new(custom_lines)
                .block(custom_block_style)
                .wrap(Wrap { trim: true });

            f.render_widget(custom_widget, custom_chunks[0]);

            // help section
            let help_lines = vec![
                Line::from(""),
                Line::from("💡 Custom Prompt Examples:"),
                Line::from("• \"Make it more casual and fun\""),
                Line::from("• \"Use bullet points and short sentences\""),
                Line::from("• \"Add more technical details\""),
                Line::from("• \"Simplify for beginners\""),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Controls: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw("Tab=navigate • ↑↓=change style • Enter=save • Esc=back"),
                ]),
            ];

            let help_widget = Paragraph::new(help_lines)
                .block(Block::default().borders(Borders::ALL).title("help"))
                .alignment(Alignment::Left);

            f.render_widget(help_widget, custom_chunks[1]);
        } else {
            // regular help section when not using custom prompt
            let help_lines = vec![
                Line::from(""),
                Line::from("🎯 Rewrite Styles:"),
                Line::from("• Professional: Polished business writing"),
                Line::from("• Casual: Friendly, conversational tone"),
                Line::from("• Concise: Brief and to the point"),
                Line::from("• Detailed: Expanded with more context"),
                Line::from("• Technical: Precise, technical language"),
                Line::from("• Simple: Easy to understand"),
                Line::from("• Custom: Your own instructions"),
                Line::from(""),
                Line::from("⚠️  settings are saved locally in ~/.stash/config.json"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Controls: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw("Tab=navigate • ↑↓=change style • Enter=save • Esc=back"),
                ]),
            ];

            let help_widget = Paragraph::new(help_lines)
                .block(Block::default().borders(Borders::ALL).title("help"))
                .alignment(Alignment::Left);

            f.render_widget(help_widget, main_chunks[3]);
        }
    }

    fn render_ai_rewrite(&mut self, f: &mut Frame, area: Rect, original_note_id: Uuid, rewritten_content: &Option<String>) {
        let (title, original_content) = if original_note_id == Uuid::nil() {
            // this is a draft rewrite
            let title = if self.title_input.is_empty() { "Draft Note" } else { &self.title_input };
            let content = self.content_editor.lines().join("\n");
            (title.to_string(), content)
        } else if let Some(original_note) = self.notes.iter().find(|n| n.id == original_note_id) {
            // this is a saved note rewrite
            let title = original_note.title.as_deref().unwrap_or("Untitled").to_string();
            (title, original_note.content.clone())
        } else {
            return; // note not found
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        let header_text = format!("🤖 ai rewrite: {}", title);
        let header_widget = Paragraph::new(header_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);

        f.render_widget(header_widget, chunks[0]);

        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(chunks[1]);

        let original_widget = Paragraph::new(original_content.as_str())
            .block(Block::default().borders(Borders::ALL).title("📝 original"))
            .wrap(Wrap { trim: true });

        f.render_widget(original_widget, content_layout[0]);

        match (&self.ai_state, rewritten_content) {
            (AiState::Processing, _) => {
                let processing_widget = Paragraph::new("🔄 processing with ai...\n\nPlease wait while your note is being rewritten.")
                    .block(Block::default().borders(Borders::ALL).title("✨ ai rewrite"))
                    .style(Style::default().fg(Color::Yellow))
                    .alignment(Alignment::Center);

                f.render_widget(processing_widget, content_layout[1]);
            }
            (AiState::Success, Some(content)) => {
                let rewritten_widget = Paragraph::new(content.as_str())
                    .block(Block::default().borders(Borders::ALL).title("✨ ai rewrite"))
                    .wrap(Wrap { trim: true })
                    .style(Style::default().fg(Color::Green));

                f.render_widget(rewritten_widget, content_layout[1]);
            }
            (AiState::Error(error), _) => {
                let error_text = format!("❌ error: {}\n\nPress Esc to go back and try again.", error);
                let error_widget = Paragraph::new(error_text)
                    .block(Block::default().borders(Borders::ALL).title("❌ error"))
                    .style(Style::default().fg(Color::Red))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true });

                f.render_widget(error_widget, content_layout[1]);
            }
            _ => {
                let waiting_widget = Paragraph::new("⏳ starting ai rewrite...")
                    .block(Block::default().borders(Borders::ALL).title("✨ ai rewrite"))
                    .style(Style::default().fg(Color::Blue))
                    .alignment(Alignment::Center);

                f.render_widget(waiting_widget, content_layout[1]);
            }
        }

        let controls_text = match &self.ai_state {
            AiState::Success => "Enter=Accept rewrite • Esc=Reject and go back",
            AiState::Processing => "Please wait... • Esc=Cancel",
            AiState::Error(_) => "Esc=Go back",
            _ => "Processing... • Esc=Cancel",
        };

        let controls_widget = Paragraph::new(controls_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        f.render_widget(controls_widget, chunks[2]);
    }


}

impl App {
    fn render_metadata_preview(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),
                Constraint::Length(6),
                Constraint::Min(0),
            ])
            .split(area);

        let tags_block = Block::default()
            .borders(Borders::ALL)
            .title("🏷️  Tags Preview")
            .style(Style::default().fg(Color::Blue));

        let tags_content = if self.extracted_tags.is_empty() {
            vec![
                Line::from(""),
                Line::from(Span::styled("No tags found", Style::default().fg(Color::DarkGray))),
                Line::from(""),
                Line::from(Span::styled("Use #tagname in content", Style::default().fg(Color::DarkGray))),
            ]
        } else {
            let mut lines = vec![Line::from("")];
            for tag in &self.extracted_tags {
                lines.push(Line::from(vec![
                    Span::styled("• #", Style::default().fg(Color::Blue)),
                    Span::styled(tag, Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                ]));
            }
            lines
        };

        let tags_widget = Paragraph::new(tags_content)
            .block(tags_block)
            .alignment(Alignment::Left);

        f.render_widget(tags_widget, chunks[0]);

        let projects_block = Block::default()
            .borders(Borders::ALL)
            .title("📁 Projects Preview")
            .style(Style::default().fg(Color::Green));

        let projects_content = if self.extracted_projects.is_empty() {
            vec![
                Line::from(""),
                Line::from(Span::styled("No projects found", Style::default().fg(Color::DarkGray))),
                Line::from(""),
                Line::from(Span::styled("Use +projectname in content", Style::default().fg(Color::DarkGray))),
            ]
        } else {
            let mut lines = vec![Line::from("")];
            for project in &self.extracted_projects {
                lines.push(Line::from(vec![
                    Span::styled("• +", Style::default().fg(Color::Green)),
                    Span::styled(project, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                ]));
            }
            lines
        };

        let projects_widget = Paragraph::new(projects_content)
            .block(projects_block)
            .alignment(Alignment::Left);

        f.render_widget(projects_widget, chunks[1]);

        let help_block = Block::default()
            .borders(Borders::ALL)
            .title("💡 Tips")
            .style(Style::default().fg(Color::Yellow));

        let help_content = vec![
            Line::from(""),
            Line::from("Type naturally:"),
            Line::from(""),
            Line::from(vec![
                Span::raw("Working on "),
                Span::styled("#rust", Style::default().fg(Color::Blue)),
                Span::raw(" "),
                Span::styled("+webapp", Style::default().fg(Color::Green)),
            ]),
            Line::from(""),
            Line::from("Tags and projects will be"),
            Line::from("extracted automatically!"),
        ];

        let help_widget = Paragraph::new(help_content)
            .block(help_block)
            .alignment(Alignment::Left);

        f.render_widget(help_widget, chunks[2]);
    }
}