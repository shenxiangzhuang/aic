use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_token: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_base_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_prompt: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_token: None,
            api_base_url: Some("https://api.openai.com".to_string()),
            model: Some("gpt-3.5-turbo".to_string()),
            default_prompt: Some(
                "You are a helpful assistant specialized in writing git commit messages. \
                Follow these guidelines: \
                1. Use the imperative mood (e.g., 'Add' not 'Added'). \
                2. Keep it concise but descriptive. \
                3. Include the scope of the change when relevant. \
                4. Explain WHY the change was made, not just WHAT was changed. \
                5. Use conventional commit format when appropriate."
                    .to_string(),
            ),
        }
    }
}

impl Config {
    pub fn config_dir() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().context("Could not find home directory")?;
        let config_dir = home_dir.join(".config").join("aic");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
        }

        Ok(config_dir)
    }

    pub fn config_path() -> Result<PathBuf> {
        let config_dir = Self::config_dir()?;
        Ok(config_dir.join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let mut file = File::open(&config_path).context("Could not open config file")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .context("Could not read config file")?;

        let config: Config = toml::from_str(&contents).context("Failed to parse config file")?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let toml_string =
            toml::to_string_pretty(self).context("Failed to serialize config to TOML")?;

        let mut file = File::create(&config_path).context("Could not create config file")?;
        file.write_all(toml_string.as_bytes())
            .context("Failed to write to config file")?;

        Ok(())
    }

    pub fn set(&mut self, key: &str, value: Option<String>) -> Result<()> {
        match key {
            "api_token" => self.api_token = value,
            "api_base_url" => self.api_base_url = value,
            "model" => self.model = value,
            "default_prompt" => self.default_prompt = value,
            _ => return Err(anyhow::anyhow!("Unknown configuration key: {}", key)),
        }

        self.save()?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        match key {
            "api_token" => self.api_token.as_ref(),
            "api_base_url" => self.api_base_url.as_ref(),
            "model" => self.model.as_ref(),
            "default_prompt" => self.default_prompt.as_ref(),
            _ => None,
        }
    }

    pub fn get_api_token(&self) -> Result<&String> {
        self.api_token.as_ref().context(
            "API token not found. Please set it using 'aic config set api_token YOUR_TOKEN'",
        )
    }

    pub fn get_api_base_url(&self) -> &str {
        self.api_base_url
            .as_deref()
            .unwrap_or("https://api.openai.com")
    }

    pub fn get_model(&self) -> &str {
        self.model.as_deref().unwrap_or("gpt-3.5-turbo")
    }

    pub fn get_default_prompt(&self) -> &str {
        self.default_prompt.as_deref().unwrap_or(
            "You are a helpful assistant specialized in writing git commit messages. \
            Follow these guidelines: \
            1. Use the imperative mood (e.g., 'Add' not 'Added'). \
            2. Keep it concise but descriptive. \
            3. Include the scope of the change when relevant. \
            4. Explain WHY the change was made, not just WHAT was changed. \
            5. Use conventional commit format when appropriate.",
        )
    }
}
