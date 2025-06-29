use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use uuid::Uuid;

use super::app::App;
use super::state::{AppMode, EditorMode, ActiveField};

pub trait Renderer {
    fn render(&mut self, f: &mut Frame);
    fn render_home(&mut self, f: &mut Frame, area: Rect);
    fn render_add_note(&mut self, f: &mut Frame, area: Rect);
    fn render_view_note(&mut self, f: &mut Frame, area: Rect, note_id: Uuid);
    fn render_help(&mut self, f: &mut Frame, area: Rect);
}

impl Renderer for App {
    fn render(&mut self, f: &mut Frame) {
        let area = f.area();

        match self.mode {
            AppMode::Home => self.render_home(f, area),
            AppMode::AddNote => self.render_add_note(f, area),
            AppMode::ViewNote(note_id) => self.render_view_note(f, area, note_id),
            AppMode::Help => self.render_help(f, area),
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

                    let mut lines = vec![
                        Line::from(vec![
                            Span::styled(format!("▶ {}", title), style.add_modifier(Modifier::BOLD)),
                        ]),
                        Line::from(vec![
                            Span::styled(format!("  {}", preview), style.fg(Color::DarkGray)),
                        ]),
                    ];

                    if !note.tags.is_empty() {
                        let tags_text = note.tags.iter()
                            .map(|tag| format!("#{}", tag))
                            .collect::<Vec<_>>()
                            .join(" ");
                        lines.push(Line::from(vec![
                            Span::styled(format!("  🏷️  {}", tags_text), style.fg(Color::Blue)),
                        ]));
                    }

                    let projects = crate::store::extract_projects(&note.content);
                    if !projects.is_empty() {
                        let projects_text = projects.iter()
                            .map(|proj| format!("+{}", proj))
                            .collect::<Vec<_>>()
                            .join(" ");
                        lines.push(Line::from(vec![
                            Span::styled(format!("  📁 {}", projects_text), style.fg(Color::Green)),
                        ]));
                    }

                    lines.push(Line::from(""));

                    ListItem::new(lines)
                })
                .collect();

            let list = List::new(items)
                .block(notes_block)
                .highlight_style(Style::default().bg(Color::Blue));

            f.render_widget(list, chunks[1]);
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
            .title("Title (t to edit)")
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
            .title("Content (c to edit)")
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
            "Mode: {} | s:save | q:quit | i:insert | t:title | c:content | Esc:command",
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

            let help_text = "Press 'q' or Esc to go back";
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
                Span::styled("🚀 Stash Help", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from("📝 Adding Notes:"),
            Line::from("  • Use #tagname for tags (e.g., #rust #webdev)"),
            Line::from("  • Use +projectname for projects (e.g., +myapp +backend)"),
            Line::from("  • Mix tags and projects in your content naturally"),
            Line::from(""),
            Line::from("🔍 Search Features:"),
            Line::from("  • stash search \"query\" - Basic search"),
            Line::from("  • stash search \"#rust +webapp\" - Tag and project search"),
            Line::from("  • stash search \"#rust -#old\" - Exclude tags"),
            Line::from("  • stash search --list-tags - See all tags"),
            Line::from("  • stash search --list-projects - See all projects"),
            Line::from(""),
            Line::from("⌨️  TUI Controls:"),
            Line::from("  Home: a=add, h=help, r=refresh, ↑↓=navigate, q=quit"),
            Line::from("  Add Note: t=edit title, c=edit content, s=save, q=quit"),
            Line::from("  Editor: i=insert mode, Esc=command mode"),
            Line::from(""),
            Line::from("💡 The right panel shows live tag/project preview!"),
            Line::from(""),
        ];

        let help_widget = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .alignment(Alignment::Left);

        f.render_widget(help_widget, area);
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