mod cli;
mod store;
mod models;
mod tui;
mod config;
mod ai;

use clap::Parser;
use cli::{Cli, Commands};
use console::{Style, Term};
use std::io::{self, Write};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ui => {
            if let Err(e) = tui::run_tui() {
                eprintln!("tui error: {}", e);
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
        Commands::Search { query, tags, projects, list_tags, list_projects, case_sensitive } => {
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
        Commands::Ai { query } => {
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

    let results = match store::search_notes_return_results(search_options) {
        Ok(results) => results,
        Err(e) => {
            eprintln!("search error: {}", e);
            return Ok(());
        }
    };

    if results.is_empty() {
        println!("no notes found matching your query.");
        return Ok(());
    }

    display_ai_search_results(&results, natural_query)?;
    Ok(())
}

fn display_ai_search_results(results: &[store::SearchResult], original_query: &str) -> Result<(), Box<dyn std::error::Error>> {
    let title_style = Style::new().bold().cyan();
    let snippet_style = Style::new().dim();
    let match_style = Style::new().bold().yellow();
    let tag_style = Style::new().bold().blue();
    let project_style = Style::new().bold().green();
    let prompt_style = Style::new().bold().magenta();

    println!("{} found {} note(s) for: \"{}\"",
        match_style.apply_to("🔍"),
        results.len(),
        original_query
    );
    println!();

    for (i, result) in results.iter().enumerate() {
        println!("{}. {}",
            i + 1,
            title_style.apply_to(result.note.title.as_deref().unwrap_or("untitled"))
        );

        if result.title_match {
            println!("   {} title match", match_style.apply_to("✓"));
        }

        if !result.tag_matches.is_empty() {
            println!("   {} tag matches: {}",
                match_style.apply_to("🏷️"),
                result.tag_matches.iter().map(|t| tag_style.apply_to(format!("#{}", t)).to_string()).collect::<Vec<_>>().join(" ")
            );
        }

        if !result.project_matches.is_empty() {
            println!("   {} project matches: {}",
                match_style.apply_to("📁"),
                result.project_matches.iter().map(|p| project_style.apply_to(format!("+{}", p)).to_string()).collect::<Vec<_>>().join(" ")
            );
        }

        if !result.content_snippets.is_empty() {
            println!("   content matches:");
            for snippet in &result.content_snippets {
                println!("   {}", snippet_style.apply_to(format!("  {}", snippet)));
            }
        }

        println!("   created: {}", result.note.created.format("%Y-%m-%d %H:%M"));

        if !result.note.tags.is_empty() {
            println!("   all tags: {}",
                result.note.tags.iter().map(|t| tag_style.apply_to(format!("#{}", t)).to_string()).collect::<Vec<_>>().join(" ")
            );
        }

        let projects = store::extract_projects(&result.note.content);
        if !projects.is_empty() {
            println!("   all projects: {}",
                projects.iter().map(|p| project_style.apply_to(format!("+{}", p)).to_string()).collect::<Vec<_>>().join(" ")
            );
        }

        println!();
    }

    loop {
        print!("{}", prompt_style.apply_to("enter note number to open, 'h' for help, or 'q' to quit: "));
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input {
            "q" | "" => break,
            "h" => {
                display_ai_help();
                continue;
            }
            _ => {
                if let Ok(index) = input.parse::<usize>() {
                    if index > 0 && index <= results.len() {
                        let result = &results[index - 1];
                        display_note_content(&result.note, &result.file_path)?;
                    } else {
                        println!("invalid note number. please try again.");
                    }
                } else {
                    println!("invalid input. enter a number, 'h' for help, or 'q' to quit.");
                }
            }
        }
    }

    Ok(())
}

fn display_ai_help() {
    let help_style = Style::new().bold().cyan();
    let command_style = Style::new().bold().yellow();

    println!();
    println!("{}", help_style.apply_to("🤖 ai search help"));
    println!("{}", "─".repeat(50));
    println!("• enter a {} to open and read that note", command_style.apply_to("number"));
    println!("• press {} to return to search results", command_style.apply_to("enter"));
    println!("• type {} to quit", command_style.apply_to("'q'"));
    println!("• type {} for this help", command_style.apply_to("'h'"));
    println!();
    println!("💡 you searched with ai using natural language!");
    println!("   try queries like:");
    println!("   • \"find my rust programming notes\"");
    println!("   • \"show me work meeting notes from this week\"");
    println!("   • \"personal project ideas with the backend tag\"");
    println!();
}

fn display_note_content(note: &models::Note, file_path: &std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let term = Term::stdout();
    let title_style = Style::new().bold().cyan();
    let content_style = Style::new().white();
    let separator_style = Style::new().dim();
    let tag_style = Style::new().bold().blue();
    let project_style = Style::new().bold().green();

    term.clear_screen()?;

    println!("{}", separator_style.apply_to("═".repeat(80)));
    println!("{}", title_style.apply_to(note.title.as_deref().unwrap_or("untitled")));
    println!("{}", separator_style.apply_to(format!("📄 file: {}", file_path.display())));
    println!("{}", separator_style.apply_to(format!("📅 created: {}", note.created.format("%Y-%m-%d %H:%M"))));

    if !note.tags.is_empty() {
        println!("{} {}",
            separator_style.apply_to("🏷️  tags:"),
            note.tags.iter().map(|t| tag_style.apply_to(format!("#{}", t)).to_string()).collect::<Vec<_>>().join(" ")
        );
    }

    let projects = store::extract_projects(&note.content);
    if !projects.is_empty() {
        println!("{} {}",
            separator_style.apply_to("📁 projects:"),
            projects.iter().map(|p| project_style.apply_to(format!("+{}", p)).to_string()).collect::<Vec<_>>().join(" ")
        );
    }

    println!("{}", separator_style.apply_to("═".repeat(80)));
    println!();

    println!("{}", content_style.apply_to(&note.content));

    println!();
    println!("{}", separator_style.apply_to("═".repeat(80)));

    print!("press enter to continue...");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(())
}
