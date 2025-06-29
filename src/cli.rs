use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "stash")]
#[command(about = "A command-line tool for managing your stash")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Ui,
    Add {
        #[arg(short, long)]
        title: Option<String>,
        #[arg(help = "The content of the note")]
        content: String,
    },
    New,
    Search {
        #[arg(help = "Search query (supports #tags, +projects, and regular text search)")]
        query: String,
        #[arg(short, long, help = "Filter by specific tags (comma-separated)")]
        tags: Option<String>,
        #[arg(short, long, help = "Filter by specific projects (comma-separated)")]
        projects: Option<String>,
        #[arg(long, help = "Show all available tags")]
        list_tags: bool,
        #[arg(long, help = "Show all available projects")]
        list_projects: bool,
        #[arg(long, help = "Case-sensitive search")]
        case_sensitive: bool,
    },
}