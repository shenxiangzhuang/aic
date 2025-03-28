use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf; // Removed unused Path import

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

    // Set a configuration value by key name
    #[allow(dead_code)] // Used by CLI command handlers
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

    // Get a configuration value by key name
    #[allow(dead_code)] // Used by CLI command handlers
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::env;

    fn setup_test_env() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        env::set_var("HOME", temp_dir.path());
        temp_dir
    }

    #[test]
    fn test_default_config() {
        let config: Config = Config::default();
        assert!(config.api_token.is_none());
        assert_eq!(config.api_base_url.as_deref(), Some("https://api.openai.com"));
        assert_eq!(config.model.as_deref(), Some("gpt-3.5-turbo"));
        assert!(config.default_prompt.is_some());
    }

    #[test]
    fn test_set_and_get() {
        let _temp_dir = setup_test_env(); // Use a temporary directory for testing
        let mut config = Config::default();
        
        // Test setting values
        config.set("api_token", Some("test_token".to_string())).unwrap();
        config.set("model", Some("gpt-4".to_string())).unwrap();
        
        // Test getting values
        assert_eq!(config.get("api_token").unwrap(), "test_token");
        assert_eq!(config.get("model").unwrap(), "gpt-4");
        
        // Test setting to None
        config.set("api_token", None).unwrap();
        assert!(config.get("api_token").is_none());
        
        // Test invalid key
        assert!(config.set("invalid_key", Some("value".to_string())).is_err());
        assert!(config.get("invalid_key").is_none());
    }

    #[test]
    fn test_save_and_load() {
        let _temp_dir = setup_test_env(); // Use a temporary directory for testing
        
        // Create and save config
        let mut config = Config::default();
        config.api_token = Some("test_token".to_string());
        config.save().unwrap();
        
        // Load and verify
        let loaded_config = Config::load().unwrap();
        assert_eq!(loaded_config.api_token, Some("test_token".to_string()));
        assert_eq!(loaded_config.api_base_url, config.api_base_url);
        assert_eq!(loaded_config.model, config.model);
        assert_eq!(loaded_config.default_prompt, config.default_prompt);
    }

    #[test]
    fn test_getter_methods() {
        let mut config = Config::default();
        config.api_token = Some("test_token".to_string());
        
        assert_eq!(config.get_api_token().unwrap(), "test_token");
        assert_eq!(config.get_api_base_url(), "https://api.openai.com");
        assert_eq!(config.get_model(), "gpt-3.5-turbo");
        assert!(config.get_default_prompt().contains("You are a helpful assistant"));
    }
}
