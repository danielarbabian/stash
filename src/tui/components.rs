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
    fn render_search(&mut self, f: &mut Frame, area: Rect);
    fn render_tag_filter(&mut self, f: &mut Frame, area: Rect);
    fn render_project_filter(&mut self, f: &mut Frame, area: Rect);
    fn render_delete_confirm(&mut self, f: &mut Frame, area: Rect, note_id: Uuid);
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
            AppMode::Search => self.render_search(f, area),
            AppMode::TagFilter => self.render_tag_filter(f, area),
            AppMode::ProjectFilter => self.render_project_filter(f, area),
            AppMode::DeleteConfirm { note_id } => self.render_delete_confirm(f, area, note_id),
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
                Span::raw(" add  "),
                Span::styled("/", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" search  "),
                Span::styled("t", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" tags  "),
                Span::styled("p", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" projects  "),
                Span::styled("h", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" help  "),
                Span::styled("s", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" settings"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("d", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" delete  "),
                Span::styled("c", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" clear  "),
                Span::styled("r", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" refresh  "),
                Span::styled("↑↓/jk", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" navigate  "),
                Span::styled("enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" view  "),
                Span::styled("q", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" quit"),
            ]),
        ];

        let title_widget = Paragraph::new(ascii_art)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);

        f.render_widget(title_widget, chunks[0]);

        let mut title = format!("notes ({}/{})", self.notes.len(), self.all_notes.len());
        let mut filters = Vec::new();

        if let Some(ref search) = self.current_search {
            filters.push(format!("search: \"{}\"", search));
        }
        if let Some(ref tag) = self.current_tag_filter {
            filters.push(format!("tag: \"{}\"", tag));
        }
        if let Some(ref project) = self.current_project_filter {
            filters.push(format!("project: \"{}\"", project));
        }

        if !filters.is_empty() {
            title.push_str(&format!(" [{}]", filters.join(", ")));
        }

        title.push_str(" - scroll with ↑↓ or j/k");

        let notes_block = Block::default()
            .borders(Borders::ALL)
            .title(title);

        if self.notes.is_empty() {
            let empty_message = Paragraph::new("no notes found. press 'a' to create your first note")
                .block(notes_block)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(empty_message, chunks[1]);
        } else {
            let items: Vec<ListItem> = self.notes
                .iter()
                .enumerate()
                .map(|(i, note)| {
                    let title = note.title.as_deref().unwrap_or("untitled");
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
                            Span::styled(format!("  {}", tags_text), tags_style),
                        ]));
                    }

                    let projects = crate::store::extract_projects(&note.content);
                    if !projects.is_empty() {
                        let projects_text = projects.iter()
                            .map(|proj| format!("+{}", proj))
                            .collect::<Vec<_>>()
                            .join(" ");
                        lines.push(Line::from(vec![
                            Span::styled(format!("  {}", projects_text), projects_style),
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
            EditorMode::Insert => "insert",
            EditorMode::Command => "command",
        };

        let mode_style = match self.editor_mode {
            EditorMode::Insert => Style::default().fg(Color::Green),
            EditorMode::Command => Style::default().fg(Color::Yellow),
        };

        let status_text = format!(
            "mode: {} | s:save | r:ai-rewrite | q:quit | i:insert | t:title | c:content | esc:command",
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

            let title = note.title.as_deref().unwrap_or("untitled");
            let header_lines = vec![
                Line::from(vec![
                    Span::styled(title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled(format!("created: {}", note.created.format("%Y-%m-%d %H:%M")), Style::default().fg(Color::DarkGray)),
                ]),
            ];

            let header_widget = Paragraph::new(header_lines)
                .block(Block::default().borders(Borders::ALL).title("note details"))
                .alignment(Alignment::Left);

            f.render_widget(header_widget, chunks[0]);

            let content_widget = Paragraph::new(note.content.as_str())
                .block(Block::default().borders(Borders::ALL).title("content"))
                .wrap(Wrap { trim: true });

            f.render_widget(content_widget, chunks[1]);

            let help_text = "press 'r' for ai rewrite • 'q' or esc to go back";
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
                Span::styled("stash help", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from("adding notes:"),
            Line::from("  • use #tagname for tags (e.g., #rust #webdev)"),
            Line::from("  • use +projectname for projects (e.g., +myapp +backend)"),
            Line::from("  • mix tags and projects in your content naturally"),
            Line::from(""),
            Line::from("search features:"),
            Line::from("  • stash search \"query\" - basic search"),
            Line::from("  • stash search \"#rust +webapp\" - tag and project search"),
            Line::from("  • stash search \"#rust -#old\" - exclude tags"),
            Line::from("  • stash search --list-tags - see all tags"),
            Line::from("  • stash search --list-projects - see all projects"),
            Line::from(""),
            Line::from("tui controls:"),
            Line::from("  home: a=add, h=help, r=refresh, ↑↓/jk=navigate, q=quit"),
            Line::from("  add note: t=edit title, c=edit content, s=save, q=quit"),
            Line::from("  editor: i=insert mode, esc=command mode"),
            Line::from(""),
            Line::from("the right panel shows live tag/project preview"),
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

        let title_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("ai configuration", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("configure your openai settings and rewrite preferences"),
        ];

        let title_widget = Paragraph::new(title_lines)
            .block(Block::default().borders(Borders::ALL).title("settings"))
            .alignment(Alignment::Center);

        f.render_widget(title_widget, main_chunks[0]);

        let api_key_display = if self.api_key_input.is_empty() {
            if self.config.has_api_key() {
                "••••••••••••••••••••••••••••••••••••••••".to_string()
            } else {
                "not configured".to_string()
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
            "api key configured"
        } else {
            "no api key configured"
        };

        let api_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("openai api key: ", Style::default().fg(Color::White)),
                Span::styled(api_key_display, api_key_style),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("status: ", Style::default().fg(Color::White)),
                Span::styled(status_text, if self.config.has_api_key() {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                }),
            ]),
            Line::from(""),
            Line::from("get your api key from: https://platform.openai.com/account/api-keys"),
        ];

        let api_block_style = if matches!(self.active_field, ActiveField::ApiKey) {
            Block::default().borders(Borders::ALL).title("api configuration").style(Style::default().fg(Color::Yellow))
        } else {
            Block::default().borders(Borders::ALL).title("api configuration")
        };

        let api_widget = Paragraph::new(api_lines).block(api_block_style);

        f.render_widget(api_widget, main_chunks[1]);

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
                Span::styled("rewrite style: ", Style::default().fg(Color::White)),
                Span::styled(current_style_name, prompt_style_style),
            ]),
            Line::from(""),
        ];

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
            Block::default().borders(Borders::ALL).title("rewrite style").style(Style::default().fg(Color::Yellow))
        } else {
            Block::default().borders(Borders::ALL).title("rewrite style")
        };

        let prompt_widget = Paragraph::new(prompt_lines).block(prompt_block_style);

        f.render_widget(prompt_widget, main_chunks[2]);

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
                    "enter your custom rewrite instructions...".to_string()
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
                Line::from("custom instructions:"),
                Line::from(""),
                Line::from(vec![
                    Span::styled(custom_prompt_display, custom_style),
                ]),
            ];

            let custom_block_style = if matches!(self.active_field, ActiveField::CustomPrompt) {
                Block::default().borders(Borders::ALL).title("custom prompt").style(Style::default().fg(Color::Yellow))
            } else {
                Block::default().borders(Borders::ALL).title("custom prompt")
            };

            let custom_widget = Paragraph::new(custom_lines)
                .block(custom_block_style)
                .wrap(Wrap { trim: true });

            f.render_widget(custom_widget, custom_chunks[0]);

            let help_lines = vec![
                Line::from(""),
                Line::from("custom prompt examples:"),
                Line::from("• \"make it more casual and fun\""),
                Line::from("• \"use bullet points and short sentences\""),
                Line::from("• \"add more technical details\""),
                Line::from("• \"simplify for beginners\""),
                Line::from(""),
                Line::from(vec![
                    Span::styled("controls: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw("tab=navigate • ↑↓=change style • enter=save • esc=back"),
                ]),
            ];

            let help_widget = Paragraph::new(help_lines)
                .block(Block::default().borders(Borders::ALL).title("help"))
                .alignment(Alignment::Left);

            f.render_widget(help_widget, custom_chunks[1]);
        } else {
            let help_lines = vec![
                Line::from(""),
                Line::from("rewrite styles:"),
                Line::from("• professional: polished business writing"),
                Line::from("• casual: friendly, conversational tone"),
                Line::from("• concise: brief and to the point"),
                Line::from("• detailed: expanded with more context"),
                Line::from("• technical: precise, technical language"),
                Line::from("• simple: easy to understand"),
                Line::from("• custom: your own instructions"),
                Line::from(""),
                Line::from("settings are saved locally in ~/.stash/config.json"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("controls: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw("tab=navigate • ↑↓=change style • enter=save • esc=back"),
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
            let title = if self.title_input.is_empty() { "draft note" } else { &self.title_input };
            let content = self.content_editor.lines().join("\n");
            (title.to_string(), content)
        } else if let Some(original_note) = self.notes.iter().find(|n| n.id == original_note_id) {
            let title = original_note.title.as_deref().unwrap_or("untitled").to_string();
            (title, original_note.content.clone())
        } else {
            return;
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        let header_text = format!("ai rewrite: {}", title);
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
            .block(Block::default().borders(Borders::ALL).title("original"))
            .wrap(Wrap { trim: true });

        f.render_widget(original_widget, content_layout[0]);

        match (&self.ai_state, rewritten_content) {
            (AiState::Processing, _) => {
                let processing_widget = Paragraph::new("processing with ai...\n\nplease wait while your note is being rewritten.")
                    .block(Block::default().borders(Borders::ALL).title("ai rewrite"))
                    .style(Style::default().fg(Color::Yellow))
                    .alignment(Alignment::Center);

                f.render_widget(processing_widget, content_layout[1]);
            }
            (AiState::Success, Some(content)) => {
                let rewritten_widget = Paragraph::new(content.as_str())
                    .block(Block::default().borders(Borders::ALL).title("ai rewrite"))
                    .wrap(Wrap { trim: true })
                    .style(Style::default().fg(Color::Green));

                f.render_widget(rewritten_widget, content_layout[1]);
            }
            (AiState::Error(error), _) => {
                let error_text = format!("error: {}\n\npress esc to go back and try again.", error);
                let error_widget = Paragraph::new(error_text)
                    .block(Block::default().borders(Borders::ALL).title("error"))
                    .style(Style::default().fg(Color::Red))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true });

                f.render_widget(error_widget, content_layout[1]);
            }
            _ => {
                let waiting_widget = Paragraph::new("starting ai rewrite...")
                    .block(Block::default().borders(Borders::ALL).title("ai rewrite"))
                    .style(Style::default().fg(Color::Blue))
                    .alignment(Alignment::Center);

                f.render_widget(waiting_widget, content_layout[1]);
            }
        }

        let controls_text = match &self.ai_state {
            AiState::Success => "enter=accept rewrite • esc=reject and go back",
            AiState::Processing => "please wait... • esc=cancel",
            AiState::Error(_) => "esc=go back",
            _ => "processing... • esc=cancel",
        };

        let controls_widget = Paragraph::new(controls_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        f.render_widget(controls_widget, chunks[2]);
    }

    fn render_search(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        let input_widget = Paragraph::new(self.search_input.as_str())
            .block(Block::default().borders(Borders::ALL).title("search notes"))
            .style(Style::default().fg(Color::Yellow));

        f.render_widget(input_widget, chunks[0]);

        let help_text = "type to search through note content and titles\npress enter to apply search, esc to cancel";
        let help_widget = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("help"))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        f.render_widget(help_widget, chunks[1]);
    }

    fn render_tag_filter(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        let input_widget = Paragraph::new(self.tag_filter_input.as_str())
            .block(Block::default().borders(Borders::ALL).title("filter by tag"))
            .style(Style::default().fg(Color::Blue));

        f.render_widget(input_widget, chunks[0]);

        let help_text = "type tag name to filter notes (without #)\npress enter to apply filter, esc to cancel";
        let help_widget = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("help"))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        f.render_widget(help_widget, chunks[1]);
    }

    fn render_project_filter(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        let input_widget = Paragraph::new(self.project_filter_input.as_str())
            .block(Block::default().borders(Borders::ALL).title("filter by project"))
            .style(Style::default().fg(Color::Green));

        f.render_widget(input_widget, chunks[0]);

        let help_text = "type project name to filter notes (without +)\npress enter to apply filter, esc to cancel";
        let help_widget = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("help"))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        f.render_widget(help_widget, chunks[1]);
    }

    fn render_delete_confirm(&mut self, f: &mut Frame, area: Rect, note_id: Uuid) {
        if let Some(note) = self.notes.iter().find(|n| n.id == note_id) {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(8),
                    Constraint::Length(6),
                    Constraint::Min(0),
                ])
                .split(area);

            let title = note.title.as_deref().unwrap_or("untitled");
            let preview = if note.content.len() > 100 {
                format!("{}...", &note.content[..100].replace('\n', " "))
            } else {
                note.content.replace('\n', " ")
            };

            let header_lines = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("delete note?", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(format!("title: {}", title), Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled(format!("preview: {}", preview), Style::default().fg(Color::DarkGray)),
                ]),
            ];

            let header_widget = Paragraph::new(header_lines)
                .block(Block::default().borders(Borders::ALL).title("confirm deletion"))
                .alignment(Alignment::Left);

            f.render_widget(header_widget, chunks[0]);

            let soft_selected = matches!(self.deletion_preference, crate::tui::app::DeletionType::Soft);
            let hard_selected = matches!(self.deletion_preference, crate::tui::app::DeletionType::Hard);

            let options_lines = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled(if soft_selected { "► " } else { "  " }, Style::default().fg(Color::Cyan)),
                    Span::styled("soft delete", if soft_selected {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    }),
                    Span::styled(" (adds deleted tag, recoverable)", Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(vec![
                    Span::styled(if hard_selected { "► " } else { "  " }, Style::default().fg(Color::Cyan)),
                    Span::styled("hard delete", if hard_selected {
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    }),
                    Span::styled(" (permanently removes file)", Style::default().fg(Color::DarkGray)),
                ]),
            ];

            let options_widget = Paragraph::new(options_lines)
                .block(Block::default().borders(Borders::ALL).title("deletion method"))
                .alignment(Alignment::Left);

            f.render_widget(options_widget, chunks[1]);

            let help_lines = vec![
                Line::from(""),
                Line::from("controls:"),
                Line::from("  up/down/tab - change deletion method"),
                Line::from("  enter/y - confirm deletion"),
                Line::from("  esc/n - cancel"),
                Line::from(""),
                Line::from("note: soft deleted notes can be recovered by removing the #deleted tag"),
            ];

            let help_widget = Paragraph::new(help_lines)
                .block(Block::default().borders(Borders::ALL).title("help"))
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Left);

            f.render_widget(help_widget, chunks[2]);
        }
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
            .title("tags")
            .style(Style::default().fg(Color::Blue));

        let tags_content = if self.extracted_tags.is_empty() {
            vec![
                Line::from(""),
                Line::from(Span::styled("no tags found", Style::default().fg(Color::DarkGray))),
                Line::from(""),
                Line::from(Span::styled("use #tagname in content", Style::default().fg(Color::DarkGray))),
            ]
        } else {
            let mut lines = vec![Line::from("")];
            for tag in &self.extracted_tags {
                lines.push(Line::from(vec![
                    Span::styled("#", Style::default().fg(Color::Blue)),
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
            .title("projects")
            .style(Style::default().fg(Color::Green));

        let projects_content = if self.extracted_projects.is_empty() {
            vec![
                Line::from(""),
                Line::from(Span::styled("no projects found", Style::default().fg(Color::DarkGray))),
                Line::from(""),
                Line::from(Span::styled("use +projectname in content", Style::default().fg(Color::DarkGray))),
            ]
        } else {
            let mut lines = vec![Line::from("")];
            for project in &self.extracted_projects {
                lines.push(Line::from(vec![
                    Span::styled("+", Style::default().fg(Color::Green)),
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
            .title("tips")
            .style(Style::default().fg(Color::Yellow));

        let help_content = vec![
            Line::from(""),
            Line::from("type naturally:"),
            Line::from(""),
            Line::from(vec![
                Span::raw("working on "),
                Span::styled("#rust", Style::default().fg(Color::Blue)),
                Span::raw(" "),
                Span::styled("+webapp", Style::default().fg(Color::Green)),
            ]),
            Line::from(""),
            Line::from("tags and projects will be"),
            Line::from("extracted automatically"),
        ];

        let help_widget = Paragraph::new(help_content)
            .block(help_block)
            .alignment(Alignment::Left);

        f.render_widget(help_widget, chunks[2]);
    }
}