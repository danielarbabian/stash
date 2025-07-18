use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::time::{timeout, Duration};

use crate::config::{Config, ConfigError};
use crate::models::Note;

#[derive(Error, Debug)]
pub enum AiError {
    #[error("config error: {0}")]
    Config(#[from] ConfigError),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("api error: {status} - {message}")]
    Api { status: u16, message: String },
    #[error("timeout error: request took longer than 30 seconds")]
    Timeout,
    #[error("invalid response format")]
    InvalidResponse,
}

#[derive(Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
}

#[derive(Deserialize)]
struct OpenAiResponseMessage {
    content: String,
}

pub struct AiClient {
    client: Client,
    config: Config,
}

impl AiClient {
    pub fn new() -> Result<Self, AiError> {
        let config = Config::load()?;
        let client = Client::new();

        Ok(Self { client, config })
    }

    pub fn is_configured(&self) -> bool {
        self.config.has_api_key()
    }

    pub async fn rewrite_note(&self, note: &Note) -> Result<String, AiError> {
        if !self.is_configured() {
            return Err(AiError::Config(ConfigError::ApiKeyNotSet));
        }

        let api_key = self.config.get_api_key()?;
        let system_prompt = self.config.get_ai_system_prompt();

        let prompt = self.create_rewrite_prompt(note);

        let request = OpenAiRequest {
            model: "gpt-4o-mini".to_string(),
            messages: vec![
                OpenAiMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                OpenAiMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            max_tokens: 2000,
            temperature: 0.3,
        };

        let response_future = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send();

        let response = timeout(Duration::from_secs(30), response_future)
            .await
            .map_err(|_| AiError::Timeout)?
            .map_err(AiError::Http)?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AiError::Api {
                status,
                message: error_text,
            });
        }

        let ai_response: OpenAiResponse = response.json().await.map_err(AiError::Http)?;

        ai_response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content.trim().to_string())
            .ok_or(AiError::InvalidResponse)
    }



    pub async fn parse_natural_command(&self, input: &str) -> Result<String, AiError> {
        if !self.is_configured() {
            return Err(AiError::Config(ConfigError::ApiKeyNotSet));
        }

        let api_key = self.config.get_api_key()?;

        let system_prompt = "You are a command parser for the 'stash' note-taking application. Your job is to convert natural language queries into valid stash search commands.

IMPORTANT: Return ONLY the search arguments, NOT the full command. Do not include 'stash search' in your response. Do not wrap your response in quotes.

Available search patterns:
- text search: just the search term (e.g., rust, async await)
- tag search: #tagname (e.g., #rust, #webdev)
- project search: +projectname (e.g., +myapp, +backend)
- combined: #tag +project text (e.g., #rust +webapp error handling)
- exclude: -#tagname or -+projectname (e.g., -#old)
- list options: --list-tags or --list-projects
- case sensitive: --case-sensitive followed by search term

Examples:
- find rust notes → #rust
- show me my webapp project → +webapp
- notes about rust in my webapp → #rust +webapp
- math notes → math
- find my old javascript code → #javascript
- list all my tags → --list-tags
- find notes with javascript but not old stuff → #javascript -#old

Return ONLY the search arguments that would come after 'stash search'. Do not use quotes around your response.";

        let user_prompt = format!("Convert this natural language query to stash search arguments: {}", input);

        let request = OpenAiRequest {
            model: "gpt-4o-mini".to_string(),
            messages: vec![
                OpenAiMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                OpenAiMessage {
                    role: "user".to_string(),
                    content: user_prompt,
                },
            ],
            max_tokens: 100,
            temperature: 0.1,
        };

        let response_future = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send();

        let response = timeout(Duration::from_secs(10), response_future)
            .await
            .map_err(|_| AiError::Timeout)?
            .map_err(AiError::Http)?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AiError::Api {
                status,
                message: error_text,
            });
        }

        let ai_response: OpenAiResponse = response.json().await.map_err(AiError::Http)?;

        let args = ai_response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content.trim().to_string())
            .ok_or(AiError::InvalidResponse)?;

        let cleaned_args = args
            .trim_start_matches('`')
            .trim_end_matches('`')
            .trim_start_matches("stash search ")
            .trim_start_matches("search ")
            .trim_start_matches('"')
            .trim_end_matches('"')
            .trim_start_matches('\'')
            .trim_end_matches('\'')
            .trim()
            .to_string();

        Ok(cleaned_args)
    }

    fn create_rewrite_prompt(&self, note: &Note) -> String {
        format!(
            "Please clean up and improve the following note content. Keep the same meaning and tone, but make it clearer, fix any grammar issues, and ensure proper markdown formatting:\n\n{}",
            note.content
        )
    }
}