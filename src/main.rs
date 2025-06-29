mod cli;
mod store;
mod models;
mod tui;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ui => {
            if let Err(e) = tui::run_tui() {
                eprintln!("TUI error: {}", e);
            }
        },
        Commands::Add { content, title } => {
            match store::save_quick_note(content, title) {
                Ok(()) => println!("note saved successfully"),
                Err(e) => eprintln!("error saving note: {}", e),
            }
        },
        Commands::New => {
            println!("creating new stash...");
        },
    }
}
