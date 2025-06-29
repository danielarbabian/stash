use std::fs;
use std::path::PathBuf;
use std::io::{self, Write};
use std::collections::{HashMap, HashSet};
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
    pub tag_matches: Vec<String>,
    pub project_matches: Vec<String>,
}

#[derive(Debug)]
pub struct SearchOptions {
    pub query: String,
    pub filter_tags: Option<String>,
    pub filter_projects: Option<String>,
    pub list_tags: bool,
    pub list_projects: bool,
    pub case_sensitive: bool,
}

#[derive(Debug)]
struct ParsedQuery {
    text_query: String,
    required_tags: Vec<String>,
    required_projects: Vec<String>,
    excluded_tags: Vec<String>,
    excluded_projects: Vec<String>,
}

pub fn search_notes_advanced(options: SearchOptions) -> Result<(), StoreError> {
    let stash_dir = get_stash_notes_dir()?;

    if !stash_dir.exists() {
        println!("No stash directory found at {:?}", stash_dir);
        println!("Try creating some notes first with 'stash add \"your note content\"'");
        return Ok(());
    }

    let all_notes = load_all_notes(&stash_dir)?;

    if options.list_tags {
        display_all_tags(&all_notes);
        return Ok(());
    }

    if options.list_projects {
        display_all_projects(&all_notes);
        return Ok(());
    }

    let parsed_query = parse_search_query(&options.query);
    let results = find_matching_notes_advanced(&all_notes, &parsed_query, &options)?;

    if results.is_empty() {
        display_no_results_help(&options.query, &parsed_query);
        return Ok(());
    }

    display_search_results_advanced(&results, &options)?;
    Ok(())
}

fn load_all_notes(stash_dir: &PathBuf) -> Result<Vec<(Note, PathBuf)>, StoreError> {
    let mut notes = Vec::new();
    let entries = fs::read_dir(stash_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Ok(note) = Note::load_from_file(&path) {
                notes.push((note, path));
            }
        }
    }

    Ok(notes)
}

fn parse_search_query(query: &str) -> ParsedQuery {
    let mut required_tags = Vec::new();
    let mut required_projects = Vec::new();
    let mut excluded_tags = Vec::new();
    let mut excluded_projects = Vec::new();

    let tag_regex = Regex::new(r"(-?)#(\w+)").unwrap();
    let project_regex = Regex::new(r"(-?)\+(\w+)").unwrap();

    let mut remaining_text = query.to_string();

    for cap in tag_regex.captures_iter(query) {
        let is_excluded = &cap[1] == "-";
        let tag = cap[2].to_string();

        if is_excluded {
            excluded_tags.push(tag);
        } else {
            required_tags.push(tag);
        }

        remaining_text = remaining_text.replace(&cap[0], "");
    }

    for cap in project_regex.captures_iter(query) {
        let is_excluded = &cap[1] == "-";
        let project = cap[2].to_string();

        if is_excluded {
            excluded_projects.push(project);
        } else {
            required_projects.push(project);
        }

        remaining_text = remaining_text.replace(&cap[0], "");
    }

    let text_query = remaining_text.trim().to_string();

    ParsedQuery {
        text_query,
        required_tags,
        required_projects,
        excluded_tags,
        excluded_projects,
    }
}

fn find_matching_notes_advanced(
    notes: &[(Note, PathBuf)],
    parsed_query: &ParsedQuery,
    options: &SearchOptions
) -> Result<Vec<SearchResult>, StoreError> {
    let matcher = SkimMatcherV2::default();
    let mut results = Vec::new();

    let filter_tags: HashSet<String> = options.filter_tags
        .as_ref()
        .map(|tags| tags.split(',').map(|t| t.trim().to_lowercase()).collect())
        .unwrap_or_default();

    let filter_projects: HashSet<String> = options.filter_projects
        .as_ref()
        .map(|projects| projects.split(',').map(|p| p.trim().to_lowercase()).collect())
        .unwrap_or_default();

    for (note, path) in notes {
        let note_tags: HashSet<String> = note.tags.iter().map(|t| t.to_lowercase()).collect();
        let note_projects = extract_projects(&note.content);
        let note_projects_set: HashSet<String> = note_projects.iter().map(|p| p.to_lowercase()).collect();

        if !filter_tags.is_empty() && !filter_tags.iter().any(|tag| note_tags.contains(tag)) {
            continue;
        }

        if !filter_projects.is_empty() && !filter_projects.iter().any(|proj| note_projects_set.contains(proj)) {
            continue;
        }

        if !parsed_query.required_tags.is_empty() {
            let required_tags_lower: HashSet<String> = parsed_query.required_tags.iter().map(|t| t.to_lowercase()).collect();
            if !required_tags_lower.iter().all(|tag| note_tags.contains(tag)) {
                continue;
            }
        }

        if !parsed_query.required_projects.is_empty() {
            let required_projects_lower: HashSet<String> = parsed_query.required_projects.iter().map(|p| p.to_lowercase()).collect();
            if !required_projects_lower.iter().all(|proj| note_projects_set.contains(proj)) {
                continue;
            }
        }

        if !parsed_query.excluded_tags.is_empty() {
            let excluded_tags_lower: HashSet<String> = parsed_query.excluded_tags.iter().map(|t| t.to_lowercase()).collect();
            if excluded_tags_lower.iter().any(|tag| note_tags.contains(tag)) {
                continue;
            }
        }

        if !parsed_query.excluded_projects.is_empty() {
            let excluded_projects_lower: HashSet<String> = parsed_query.excluded_projects.iter().map(|p| p.to_lowercase()).collect();
            if excluded_projects_lower.iter().any(|proj| note_projects_set.contains(proj)) {
                continue;
            }
        }

        let mut best_score = 0i64;
        let mut title_match = false;
        let mut content_snippets = Vec::new();
        let mut tag_matches = Vec::new();
        let mut project_matches = Vec::new();

        if !parsed_query.text_query.is_empty() {
            if let Some(title) = &note.title {
                let title_to_search = if options.case_sensitive { title.clone() } else { title.to_lowercase() };
                let query_to_use = if options.case_sensitive { parsed_query.text_query.clone() } else { parsed_query.text_query.to_lowercase() };

                if let Some(score) = matcher.fuzzy_match(&title_to_search, &query_to_use) {
                    best_score = best_score.max(score);
                    title_match = true;
                }
            }

            let content_lines: Vec<&str> = note.content.lines().collect();
            for (line_num, line) in content_lines.iter().enumerate() {
                let line_to_search = if options.case_sensitive { line.to_string() } else { line.to_lowercase() };
                let query_to_use = if options.case_sensitive { parsed_query.text_query.clone() } else { parsed_query.text_query.to_lowercase() };

                if let Some(score) = matcher.fuzzy_match(&line_to_search, &query_to_use) {
                    best_score = best_score.max(score);
                    content_snippets.push(format!("Line {}: {}", line_num + 1, line.trim()));
                }
            }
        } else {
            best_score = 100;
        }

        for tag in &parsed_query.required_tags {
            if note_tags.contains(&tag.to_lowercase()) {
                tag_matches.push(tag.clone());
            }
        }

        for project in &parsed_query.required_projects {
            if note_projects_set.contains(&project.to_lowercase()) {
                project_matches.push(project.clone());
            }
        }

        if best_score > 0 || !tag_matches.is_empty() || !project_matches.is_empty() {
            content_snippets.truncate(3);
            results.push(SearchResult {
                note: note.clone(),
                score: best_score,
                title_match,
                content_snippets,
                file_path: path.clone(),
                tag_matches,
                project_matches,
            });
        }
    }

    results.sort_by(|a, b| {
        let a_special_matches = a.tag_matches.len() + a.project_matches.len();
        let b_special_matches = b.tag_matches.len() + b.project_matches.len();

        if a_special_matches != b_special_matches {
            b_special_matches.cmp(&a_special_matches)
        } else {
            b.score.cmp(&a.score)
        }
    });

    Ok(results)
}

fn display_all_tags(notes: &[(Note, PathBuf)]) {
    let mut tag_counts: HashMap<String, usize> = HashMap::new();

    for (note, _) in notes {
        for tag in &note.tags {
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }

    if tag_counts.is_empty() {
        println!("No tags found in your notes.");
        println!("Add tags to your notes using #tagname syntax.");
        return;
    }

    let mut sorted_tags: Vec<_> = tag_counts.into_iter().collect();
    sorted_tags.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    let tag_style = Style::new().bold().cyan();
    let count_style = Style::new().dim();

    println!("\n{} Available Tags:", tag_style.apply_to("📋"));
    println!("{}", "─".repeat(50));

    for (tag, count) in sorted_tags {
        println!("#{} {}",
            tag_style.apply_to(&tag),
            count_style.apply_to(format!("({} note{})", count, if count == 1 { "" } else { "s" }))
        );
    }

    println!("\n💡 Usage examples:");
    println!("  stash search \"#rust\"           - Find notes with rust tag");
    println!("  stash search \"#rust #web\"      - Find notes with both tags");
    println!("  stash search \"#rust -#old\"     - Find rust notes, exclude old ones");
    println!("  stash search --tags rust,web    - Filter by specific tags");
}

fn display_all_projects(notes: &[(Note, PathBuf)]) {
    let mut project_counts: HashMap<String, usize> = HashMap::new();

    for (note, _) in notes {
        let projects = extract_projects(&note.content);
        for project in projects {
            *project_counts.entry(project).or_insert(0) += 1;
        }
    }

    if project_counts.is_empty() {
        println!("No projects found in your notes.");
        println!("Add projects to your notes using +projectname syntax.");
        return;
    }

    let mut sorted_projects: Vec<_> = project_counts.into_iter().collect();
    sorted_projects.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    let project_style = Style::new().bold().green();
    let count_style = Style::new().dim();

    println!("\n{} Available Projects:", project_style.apply_to("📁"));
    println!("{}", "─".repeat(50));

    for (project, count) in sorted_projects {
        println!("+{} {}",
            project_style.apply_to(&project),
            count_style.apply_to(format!("({} note{})", count, if count == 1 { "" } else { "s" }))
        );
    }

    println!("\n💡 Usage examples:");
    println!("  stash search \"+myapp\"          - Find notes for myapp project");
    println!("  stash search \"+web +backend\"   - Find notes for web and backend");
    println!("  stash search \"+web -+old\"      - Find web notes, exclude old project");
    println!("  stash search --projects web,api  - Filter by specific projects");
}

fn display_no_results_help(_original_query: &str, parsed_query: &ParsedQuery) {
    let help_style = Style::new().bold().yellow();
    let suggestion_style = Style::new().cyan();

    println!("No notes found matching your search criteria.");
    println!();

    if !parsed_query.text_query.is_empty() {
        println!("🔍 Text search: \"{}\"", parsed_query.text_query);
    }

    if !parsed_query.required_tags.is_empty() {
        println!("🏷️  Required tags: {}", parsed_query.required_tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" "));
    }

    if !parsed_query.required_projects.is_empty() {
        println!("📁 Required projects: {}", parsed_query.required_projects.iter().map(|p| format!("+{}", p)).collect::<Vec<_>>().join(" "));
    }

    println!();
    println!("{}", help_style.apply_to("💡 Search Tips:"));
    println!("  • Use {} to search by tags", suggestion_style.apply_to("#tagname"));
    println!("  • Use {} to search by projects", suggestion_style.apply_to("+projectname"));
    println!("  • Use {} to exclude tags/projects", suggestion_style.apply_to("-#tag or -+project"));
    println!("  • Combine: {} searches for rust notes in web project", suggestion_style.apply_to("\"#rust +web API\""));
    println!("  • Try {} to see all available tags", suggestion_style.apply_to("stash search \"\" --list-tags"));
    println!("  • Try {} to see all available projects", suggestion_style.apply_to("stash search \"\" --list-projects"));
}

fn display_search_results_advanced(results: &[SearchResult], options: &SearchOptions) -> Result<(), StoreError> {
    let _term = Term::stdout();
    let title_style = Style::new().bold().cyan();
    let snippet_style = Style::new().dim();
    let match_style = Style::new().bold().yellow();
    let tag_style = Style::new().bold().blue();
    let project_style = Style::new().bold().green();
    let prompt_style = Style::new().bold().magenta();

    println!("\n{} Found {} note(s):",
        match_style.apply_to("🔍"),
        results.len()
    );

    if !options.query.is_empty() {
        println!("   Query: \"{}\"", options.query);
    }

    if let Some(tags) = &options.filter_tags {
        println!("   Tags filter: {}", tags);
    }

    if let Some(projects) = &options.filter_projects {
        println!("   Projects filter: {}", projects);
    }

    println!();

    for (i, result) in results.iter().enumerate() {
        println!("{}. {}",
            i + 1,
            title_style.apply_to(result.note.title.as_deref().unwrap_or("Untitled"))
        );

        if result.title_match {
            println!("   {} Title match", match_style.apply_to("✓"));
        }

        if !result.tag_matches.is_empty() {
            println!("   {} Tag matches: {}",
                match_style.apply_to("🏷️"),
                result.tag_matches.iter().map(|t| tag_style.apply_to(format!("#{}", t)).to_string()).collect::<Vec<_>>().join(" ")
            );
        }

        if !result.project_matches.is_empty() {
            println!("   {} Project matches: {}",
                match_style.apply_to("📁"),
                result.project_matches.iter().map(|p| project_style.apply_to(format!("+{}", p)).to_string()).collect::<Vec<_>>().join(" ")
            );
        }

        if !result.content_snippets.is_empty() {
            println!("   Content matches:");
            for snippet in &result.content_snippets {
                println!("   {}", snippet_style.apply_to(format!("  {}", snippet)));
            }
        }

        println!("   Created: {}", result.note.created.format("%Y-%m-%d %H:%M"));

        if !result.note.tags.is_empty() {
            println!("   All tags: {}",
                result.note.tags.iter().map(|t| tag_style.apply_to(format!("#{}", t)).to_string()).collect::<Vec<_>>().join(" ")
            );
        }

        let projects = extract_projects(&result.note.content);
        if !projects.is_empty() {
            println!("   All projects: {}",
                projects.iter().map(|p| project_style.apply_to(format!("+{}", p)).to_string()).collect::<Vec<_>>().join(" ")
            );
        }

        println!();
    }

    loop {
        print!("{}", prompt_style.apply_to("Enter note number to open, 'h' for help, or 'q' to quit: "));
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input {
            "q" | "" => break,
            "h" => {
                display_interactive_help();
                continue;
            }
            _ => {
                if let Ok(index) = input.parse::<usize>() {
                    if index > 0 && index <= results.len() {
                        let result = &results[index - 1];
                        display_note_content_advanced(&result.note, &result.file_path)?;
                    } else {
                        println!("Invalid note number. Please try again.");
                    }
                } else {
                    println!("Invalid input. Enter a number, 'h' for help, or 'q' to quit.");
                }
            }
        }
    }

    Ok(())
}

fn display_interactive_help() {
    let help_style = Style::new().bold().cyan();
    let command_style = Style::new().bold().yellow();

    println!();
    println!("{}", help_style.apply_to("🚀 Interactive Search Help"));
    println!("{}", "─".repeat(50));
    println!("• Enter a {} to open and read that note", command_style.apply_to("number"));
    println!("• Press {} to return to search results", command_style.apply_to("Enter"));
    println!("• Type {} to quit", command_style.apply_to("'q'"));
    println!("• Type {} for this help", command_style.apply_to("'h'"));
    println!();
}

fn display_note_content_advanced(note: &Note, file_path: &PathBuf) -> Result<(), StoreError> {
    let term = Term::stdout();
    let title_style = Style::new().bold().cyan();
    let content_style = Style::new().white();
    let separator_style = Style::new().dim();
    let tag_style = Style::new().bold().blue();
    let project_style = Style::new().bold().green();

    term.clear_screen()?;

    println!("{}", separator_style.apply_to("═".repeat(80)));
    println!("{}", title_style.apply_to(note.title.as_deref().unwrap_or("Untitled")));
    println!("{}", separator_style.apply_to(format!("📄 File: {}", file_path.display())));
    println!("{}", separator_style.apply_to(format!("📅 Created: {}", note.created.format("%Y-%m-%d %H:%M"))));

    if !note.tags.is_empty() {
        println!("{} {}",
            separator_style.apply_to("🏷️  Tags:"),
            note.tags.iter().map(|t| tag_style.apply_to(format!("#{}", t)).to_string()).collect::<Vec<_>>().join(" ")
        );
    }

    let projects = extract_projects(&note.content);
    if !projects.is_empty() {
        println!("{} {}",
            separator_style.apply_to("📁 Projects:"),
            projects.iter().map(|p| project_style.apply_to(format!("+{}", p)).to_string()).collect::<Vec<_>>().join(" ")
        );
    }

    println!("{}", separator_style.apply_to("═".repeat(80)));
    println!();

    println!("{}", content_style.apply_to(&note.content));

    println!();
    println!("{}", separator_style.apply_to("═".repeat(80)));

    print!("Press Enter to continue...");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(())
}

pub fn search_notes(query: &str) -> Result<(), StoreError> {
    let options = SearchOptions {
        query: query.to_string(),
        filter_tags: None,
        filter_projects: None,
        list_tags: false,
        list_projects: false,
        case_sensitive: false,
    };

    search_notes_advanced(options)
}

pub fn save_quick_note(content: String, title: Option<String>) -> Result<(), StoreError> {
    let stash_dir = get_stash_notes_dir()?;
    ensure_directory_exists(&stash_dir)?;

    let note_id = Uuid::new_v4();
    let tags = extract_tags(&content);
    let projects = extract_projects(&content);
    let links_to = extract_links(&content);
    let created = Utc::now();

    let frontmatter = create_frontmatter(&note_id, &title, &tags, &projects, &links_to, &created);
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

pub fn extract_tags(content: &str) -> Vec<String> {
    let tag_regex = Regex::new(r"#(\w+)").unwrap();
    tag_regex
        .captures_iter(content)
        .map(|cap| cap[1].to_string())
        .collect()
}

pub fn extract_projects(content: &str) -> Vec<String> {
    let project_regex = Regex::new(r"\+(\w+)").unwrap();
    project_regex
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
    projects: &[String],
    links_to: &[String],
    created: &DateTime<Utc>,
) -> String {
    let tags_yaml = if tags.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", tags.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(", "))
    };

    let projects_yaml = if projects.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", projects.iter().map(|p| format!("\"{}\"", p)).collect::<Vec<_>>().join(", "))
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
        "---\nid: {}\ntitle: {}\ntags: {}\nprojects: {}\nlinks_to: {}\ncreated: {}\nupdated: null\nsource: \"QuickCapture\"\n---",
        id, title_yaml, tags_yaml, projects_yaml, links_yaml, created.format("%Y-%m-%dT%H:%M:%S%.3fZ")
    )
}