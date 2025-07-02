use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "stash")]
#[command(about = "a command-line tool for managing your stash")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Add {
        #[arg(short, long)]
        title: Option<String>,
        #[arg(help = "the content of the note")]
        content: String,
    },
    New,
    Search {
        #[arg(help = "search query (supports #tags, +projects, and regular text search)")]
        query: String,
        #[arg(short, long, help = "filter by specific tags (comma-separated)")]
        tags: Option<String>,
        #[arg(short, long, help = "filter by specific projects (comma-separated)")]
        projects: Option<String>,
        #[arg(long, help = "show all available tags")]
        list_tags: bool,
        #[arg(long, help = "show all available projects")]
        list_projects: bool,
        #[arg(long, help = "case-sensitive search")]
        case_sensitive: bool,
    },
    Ai {
        #[arg(help = "natural language query to search for notes")]
        query: String,
    },
}