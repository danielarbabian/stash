mod cli;
mod store;
mod models;
mod tui;
mod config;
mod ai;

use clap::Parser;
use cli::{Cli, Commands};
use console::Style;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        None => {
            if let Err(e) = tui::run_tui() {
                eprintln!("tui error: {}", e);
            }
        },
        Some(Commands::Add { content, title }) => {
            match store::save_quick_note(content, title) {
                Ok(()) => println!("note saved successfully"),
                Err(e) => eprintln!("error saving note: {}", e),
            }
        },
        Some(Commands::New) => {
            if let Err(e) = tui::run_tui_new_note() {
                eprintln!("tui error: {}", e);
            }
        },
        Some(Commands::Search { query, tags, projects, list_tags, list_projects, case_sensitive }) => {
            let search_options = store::SearchOptions {
                query,
                filter_tags: tags,
                filter_projects: projects,
                list_tags,
                list_projects,
                case_sensitive,
            };

            if let Err(e) = store::search_notes_advanced(search_options) {
                eprintln!("search error: {}", e);
            }
        },
        Some(Commands::Ai { query }) => {
            if let Err(e) = ai_search_cli(&query).await {
                eprintln!("ai search error: {}", e);
            }
        },
    }
}

async fn ai_search_cli(natural_query: &str) -> Result<(), Box<dyn std::error::Error>> {
    let ai_client = match ai::AiClient::new() {
        Ok(client) => client,
        Err(e) => {
            eprintln!("failed to initialize ai client: {}", e);
            eprintln!("please run 'stash ui' and press 's' to configure your openai api key");
            return Ok(());
        }
    };

    if !ai_client.is_configured() {
        eprintln!("openai api key not configured");
        eprintln!("please run 'stash ui' and press 's' to configure your api key");
        return Ok(());
    }

    let loading_style = Style::new().bold().cyan();
    let success_style = Style::new().bold().green();

    println!("{} translating your query with ai...", loading_style.apply_to("🤖"));

    let search_args = match ai_client.parse_natural_command(&natural_query).await {
        Ok(args) => args,
        Err(e) => {
            eprintln!("failed to translate query: {}", e);
            return Ok(());
        }
    };

    println!("{} generated search: {}", success_style.apply_to("✓"), search_args);
    println!();

    let search_options = store::SearchOptions {
        query: search_args,
        filter_tags: None,
        filter_projects: None,
        list_tags: false,
        list_projects: false,
        case_sensitive: false,
    };

    if let Err(e) = store::search_notes_advanced(search_options) {
        eprintln!("search error: {}", e);
    }

    Ok(())
}


