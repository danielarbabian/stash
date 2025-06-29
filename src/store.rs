use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use regex::Regex;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Home directory not found")]
    HomeNotFound,
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