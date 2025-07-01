use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("home directory not found")]
    HomeNotFound,
    #[error("api key not configured")]
    ApiKeyNotSet,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub openai_api_key: Option<String>,
    pub ai_enabled: bool,
    pub ai_prompt_style: String,
    pub custom_ai_prompt: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            openai_api_key: None,
            ai_enabled: false,
            ai_prompt_style: "professional".to_string(),
            custom_ai_prompt: None,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = Self::config_file_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let config_content = fs::read_to_string(&config_path)?;
        let config: Config = serde_json::from_str(&config_content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let config_path = Self::config_file_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let config_content = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, config_content)?;
        Ok(())
    }

    pub fn set_api_key(&mut self, api_key: String) -> Result<(), ConfigError> {
        self.openai_api_key = Some(api_key);
        self.ai_enabled = true;
        self.save()
    }

    pub fn get_api_key(&self) -> Result<&str, ConfigError> {
        self.openai_api_key.as_deref().ok_or(ConfigError::ApiKeyNotSet)
    }

    pub fn has_api_key(&self) -> bool {
        self.openai_api_key.is_some() && !self.openai_api_key.as_ref().unwrap().is_empty()
    }

    pub fn clear_api_key(&mut self) -> Result<(), ConfigError> {
        self.openai_api_key = None;
        self.ai_enabled = false;
        self.save()
    }

    pub fn set_prompt_style(&mut self, style: String) -> Result<(), ConfigError> {
        self.ai_prompt_style = style;
        self.save()
    }

    pub fn set_custom_prompt(&mut self, prompt: Option<String>) -> Result<(), ConfigError> {
        self.custom_ai_prompt = prompt;
        self.save()
    }

    pub fn get_ai_system_prompt(&self) -> String {
        let base_instruction = "You are an expert writing assistant. Your task is to clean up and improve notes while preserving their original meaning and structure. Keep the same tone but make the text clearer, fix grammar, improve organization, and ensure proper markdown formatting. Do not add new information or change the core content. Return only the improved text without any additional commentary, introductions, or explanations.";

        let style_instruction = match self.ai_prompt_style.as_str() {
            "professional" => " Make the writing more professional and polished.",
            "casual" => " Keep the writing casual and conversational.",
            "concise" => " Make the writing more concise and to the point.",
            "detailed" => " Expand on ideas and add more detail where appropriate.",
            "technical" => " Use more technical language and precise terminology.",
            "simple" => " Simplify the language and make it easier to understand.",
            "custom" => {
                if let Some(ref custom) = self.custom_ai_prompt {
                    return format!("{} {}", base_instruction, custom);
                } else {
                    ""
                }
            }
            _ => "",
        };

        format!("{}{}", base_instruction, style_instruction)
    }

    fn config_file_path() -> Result<PathBuf, ConfigError> {
        let home = dirs::home_dir().ok_or(ConfigError::HomeNotFound)?;
        Ok(home.join(".stash").join("config.json"))
    }
}