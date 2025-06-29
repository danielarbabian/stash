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
        #[arg(help = "The search query")]
        query: String,
    },
}