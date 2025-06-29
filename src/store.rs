use std::fs;
use std::path::PathBuf;
use std::io::{self, Write};
use chrono::{DateTime, Utc};
use regex::Regex;
use thiserror::Error;
use uuid::Uuid;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use console::{Style, Term};

use crate::models::{Note, NoteError};

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Home directory not found")]
    HomeNotFound,
    #[error("Note error: {0}")]
    Note(#[from] NoteError),
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub note: Note,
    pub score: i64,
    pub title_match: bool,
    pub content_snippets: Vec<String>,
    pub file_path: PathBuf,
}

pub fn search_notes(query: &str) -> Result<(), StoreError> {
    let stash_dir = get_stash_notes_dir()?;

    if !stash_dir.exists() {
        println!("No stash directory found at {:?}", stash_dir);
        return Ok(());
    }

    let results = find_matching_notes(&stash_dir, query)?;

    if results.is_empty() {
        println!("No notes found matching '{}'", query);
        return Ok(());
    }

    display_search_results(&results, query)?;
    Ok(())
}

fn find_matching_notes(stash_dir: &PathBuf, query: &str) -> Result<Vec<SearchResult>, StoreError> {
    let matcher = SkimMatcherV2::default();
    let mut results = Vec::new();

    let entries = fs::read_dir(stash_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Ok(note) = Note::load_from_file(&path) {
                let mut best_score = 0i64;
                let mut title_match = false;
                let mut content_snippets = Vec::new();

                if let Some(title) = &note.title {
                    if let Some(score) = matcher.fuzzy_match(title, query) {
                        best_score = best_score.max(score);
                        title_match = true;
                    }
                }

                let content_lines: Vec<&str> = note.content.lines().collect();
                for (line_num, line) in content_lines.iter().enumerate() {
                    if let Some(score) = matcher.fuzzy_match(line, query) {
                        best_score = best_score.max(score);
                        content_snippets.push(format!("Line {}: {}", line_num + 1, line.trim()));
                    }
                }

                if best_score > 0 {
                    content_snippets.truncate(3);
                    results.push(SearchResult {
                        note,
                        score: best_score,
                        title_match,
                        content_snippets,
                        file_path: path,
                    });
                }
            }
        }
    }

    results.sort_by(|a, b| b.score.cmp(&a.score));
    Ok(results)
}

fn display_search_results(results: &[SearchResult], query: &str) -> Result<(), StoreError> {
    let _term = Term::stdout();
    let title_style = Style::new().bold().cyan();
    let snippet_style = Style::new().dim();
    let match_style = Style::new().bold().yellow();
    let prompt_style = Style::new().bold().green();

    println!("\nFound {} note(s) matching '{}':\n", results.len(), query);

    for (i, result) in results.iter().enumerate() {
        println!("{}. {}",
            i + 1,
            title_style.apply_to(result.note.title.as_deref().unwrap_or("Untitled"))
        );

        if result.title_match {
            println!("   {} Title match", match_style.apply_to("✓"));
        }

        if !result.content_snippets.is_empty() {
            println!("   Content matches:");
            for snippet in &result.content_snippets {
                println!("   {}", snippet_style.apply_to(format!("  {}", snippet)));
            }
        }

        println!("   Created: {}", result.note.created.format("%Y-%m-%d %H:%M"));
        if !result.note.tags.is_empty() {
            println!("   Tags: {}", result.note.tags.join(", "));
        }
        println!();
    }

    loop {
        print!("{}", prompt_style.apply_to("Enter note number to open (or 'q' to quit): "));
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "q" || input.is_empty() {
            break;
        }

        if let Ok(index) = input.parse::<usize>() {
            if index > 0 && index <= results.len() {
                let result = &results[index - 1];
                display_note_content(&result.note, &result.file_path)?;
            } else {
                println!("Invalid note number. Please try again.");
            }
        } else {
            println!("Invalid input. Please enter a number or 'q' to quit.");
        }
    }

    Ok(())
}

fn display_note_content(note: &Note, file_path: &PathBuf) -> Result<(), StoreError> {
    let term = Term::stdout();
    let title_style = Style::new().bold().cyan();
    let content_style = Style::new().white();
    let separator_style = Style::new().dim();

    term.clear_screen()?;

    println!("{}", separator_style.apply_to("=".repeat(80)));
    println!("{}", title_style.apply_to(note.title.as_deref().unwrap_or("Untitled")));
    println!("{}", separator_style.apply_to(format!("File: {}", file_path.display())));
    println!("{}", separator_style.apply_to(format!("Created: {}", note.created.format("%Y-%m-%d %H:%M"))));
    if !note.tags.is_empty() {
        println!("{}", separator_style.apply_to(format!("Tags: {}", note.tags.join(", "))));
    }
    println!("{}", separator_style.apply_to("=".repeat(80)));
    println!();

    println!("{}", content_style.apply_to(&note.content));

    println!();
    println!("{}", separator_style.apply_to("=".repeat(80)));

    print!("Press Enter to continue...");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(())
}

pub fn save_quick_note(content: String, title: Option<String>) -> Result<(), StoreError> {
    let stash_dir = get_stash_notes_dir()?;
    ensure_directory_exists(&stash_dir)?;

    let note_id = Uuid::new_v4();
    let tags = extract_tags(&content);
    let links_to = extract_links(&content);
    let created = Utc::now();

    let frontmatter = create_frontmatter(&note_id, &title, &tags, &links_to, &created);
    let file_content = format!("{}\n{}", frontmatter, content);

    let file_path = stash_dir.join(format!("{}.md", note_id));
    fs::write(file_path, file_content)?;

    Ok(())
}

fn get_stash_notes_dir() -> Result<PathBuf, StoreError> {
    let home = dirs::home_dir().ok_or(StoreError::HomeNotFound)?;
    Ok(home.join(".stash").join("notes"))
}

fn ensure_directory_exists(path: &PathBuf) -> Result<(), StoreError> {
    fs::create_dir_all(path)?;
    Ok(())
}

fn extract_tags(content: &str) -> Vec<String> {
    let tag_regex = Regex::new(r"#(\w+)").unwrap();
    tag_regex
        .captures_iter(content)
        .map(|cap| cap[1].to_string())
        .collect()
}

fn extract_links(content: &str) -> Vec<String> {
    let link_regex = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    link_regex
        .captures_iter(content)
        .map(|cap| cap[1].to_string())
        .collect()
}

fn create_frontmatter(
    id: &Uuid,
    title: &Option<String>,
    tags: &[String],
    links_to: &[String],
    created: &DateTime<Utc>,
) -> String {
    let tags_yaml = if tags.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", tags.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(", "))
    };

    let links_yaml = if links_to.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", links_to.iter().map(|l| format!("\"{}\"", l)).collect::<Vec<_>>().join(", "))
    };

    let title_yaml = match title {
        Some(t) => format!("\"{}\"", t),
        None => "null".to_string(),
    };

    format!(
        "---\nid: {}\ntitle: {}\ntags: {}\nlinks_to: {}\ncreated: {}\nupdated: null\nsource: \"QuickCapture\"\n---",
        id, title_yaml, tags_yaml, links_yaml, created.format("%Y-%m-%dT%H:%M:%S%.3fZ")
    )
}