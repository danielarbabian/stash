use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Note {
    pub id: Uuid,
    pub title: Option<String>,
    pub tags: Vec<String>,
    pub links_to: Vec<Uuid>,
    pub created: DateTime<Utc>,
    pub updated: Option<DateTime<Utc>>,
    pub source: NoteSource,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct NoteFrontMatter {
    pub id: Uuid,
    pub title: Option<String>,
    pub tags: Vec<String>,
    pub links_to: Vec<Uuid>,
    pub created: DateTime<Utc>,
    pub updated: Option<DateTime<Utc>>,
    pub source: NoteSource,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NoteSource {
    QuickCapture,
    Editor,
    UI,
}

#[derive(Error, Debug)]
pub enum NoteError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Invalid frontmatter format")]
    InvalidFrontmatter,
    #[error("Missing frontmatter")]
    MissingFrontmatter,
}

impl Note {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Note, NoteError> {
        let content = fs::read_to_string(path)?;
        Self::from_markdown_string(&content)
    }

    pub fn from_markdown_string(content: &str) -> Result<Note, NoteError> {
        let (frontmatter_str, markdown_content) = split_frontmatter(content)?;

        let frontmatter: NoteFrontMatter = serde_yaml::from_str(&frontmatter_str)?;

        let note = Note {
            id: frontmatter.id,
            title: frontmatter.title,
            tags: frontmatter.tags,
            links_to: frontmatter.links_to,
            created: frontmatter.created,
            updated: frontmatter.updated,
            source: frontmatter.source,
            content: markdown_content.to_string(),
        };

        Ok(note)
    }

    pub fn to_markdown_string(&self) -> Result<String, NoteError> {
        let frontmatter = NoteFrontMatter {
            id: self.id,
            title: self.title.clone(),
            tags: self.tags.clone(),
            links_to: self.links_to.clone(),
            created: self.created,
            updated: self.updated,
            source: self.source.clone(),
        };

        let frontmatter_yaml = serde_yaml::to_string(&frontmatter)?;
        let frontmatter_content = frontmatter_yaml.trim();

        Ok(format!("---\n{}\n---\n{}", frontmatter_content, self.content))
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), NoteError> {
        let markdown_content = self.to_markdown_string()?;
        fs::write(path, markdown_content)?;
        Ok(())
    }
}

fn split_frontmatter(content: &str) -> Result<(String, &str), NoteError> {
    if !content.starts_with("---\n") {
        return Err(NoteError::MissingFrontmatter);
    }

    let content_without_first_delimiter = &content[4..];

    if let Some(end_pos) = content_without_first_delimiter.find("\n---\n") {
        let frontmatter = &content_without_first_delimiter[..end_pos];
        let markdown_content = &content_without_first_delimiter[end_pos + 5..];

        Ok((frontmatter.to_string(), markdown_content))
    } else {
        Err(NoteError::InvalidFrontmatter)
    }
}