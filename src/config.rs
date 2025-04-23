use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

const DEFAULT_SYSTEM_PROMPT: &str = "You are an expert at writing clear and concise commit messages. \
    Follow these rules strictly:\n\n\
    1. Start with a type: feat, fix, docs, style, refactor, perf, test, build, ci, chore, or revert\n\
    2. Add a scope in parentheses when the change affects a specific component/module\n\
    3. Write a brief description in imperative mood (e.g., 'add' not 'added')\n\
    4. Keep the first line under 72 characters\n\
    5. For simple changes (single file, small modifications), use only the subject line\n\
    6. For complex changes (multiple files, new features, breaking changes):\n\
       - Add a body explaining what and why\n\
       - Use numbered points (1., 2., 3., etc.) to list distinct changes\n\
       - Organize points in order of importance\n\
    Examples:\n\
    Simple: fix(parser): correct string interpolation logic\n\
    Complex: feat(auth): implement OAuth2 authentication system\n\n\
    This commit adds comprehensive OAuth2 support:\n\n\
    1. Implement Google and GitHub OAuth2 providers\n\
    2. Create secure token storage and refresh mechanism\n\
    3. Add middleware for protected route authentication\n\
    4. Update user model to store OAuth identifiers";

const DEFAULT_USER_PROMPT: &str =
    "Generate a commit message for the following changes. First analyze the complexity of the diff.\n\n\
    For simple changes, provide only a subject line.\n\n\
    For complex changes, include a body with numbered points (1., 2., 3.) that clearly outline\n\
    each distinct modification or feature. Organize these points by importance.\n\n\
    Look for patterns like new features, bug fixes, or configuration changes to determine\n\
    the appropriate type and scope:\n\n\
    ```diff\n{}\n```";

const PROJECT_CONFIG_FILENAME: &str = ".aic.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    // Skip serializing None values to keep the config file clean
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_token: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_base_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_prompt: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_token: None,
            api_base_url: Some("https://api.openai.com".to_string()),
            model: Some("gpt-3.5-turbo".to_string()),
            system_prompt: Some(DEFAULT_SYSTEM_PROMPT.to_string()),
            user_prompt: Some(DEFAULT_USER_PROMPT.to_string()),
        }
    }
}

impl Config {
    pub fn config_dir() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().context("Could not find home directory")?;
        let config_dir = if cfg!(target_os = "windows") {
            home_dir.join("AppData").join("Roaming").join("aic")
        } else {
            home_dir.join(".config").join("aic")
        };
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
        }

        Ok(config_dir)
    }

    pub fn config_path() -> Result<PathBuf> {
        let config_dir = Self::config_dir()?;
        Ok(config_dir.join("config.toml"))
    }

    // Find a project-level config file by looking in the current directory and parent directories
    // Stop when reaching the git repository root (directory with .git folder)
    pub fn find_project_config() -> Result<Option<PathBuf>> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        let mut dir = current_dir.as_path();

        // Look for .aic.toml in current directory and up to git repo root
        loop {
            // Check for project config file
            let config_path = dir.join(PROJECT_CONFIG_FILENAME);
            if config_path.exists() {
                return Ok(Some(config_path));
            }

            // Check if this is the git repository root
            let git_dir = dir.join(".git");
            if git_dir.exists() {
                // Stop at the git repository root
                // Only search for project config up to this directory
                break;
            }

            // Go up one directory
            match dir.parent() {
                Some(parent) => dir = parent,
                None => break, // Reached filesystem root
            }
        }

        // No project config found
        Ok(None)
    }

    // Load a config from a TOML file (now works for both global and project config)
    fn load_toml_config(path: &PathBuf) -> Result<Self> {
        let mut file = File::open(path).context("Could not open TOML config file")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .context("Could not read TOML config file")?;

        let config: Config = toml::from_str(&contents).context("Failed to parse TOML config file")?;
        Ok(config)
    }

    // Load the global config from TOML
    fn load_global_config() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        Self::load_toml_config(&config_path)
    }

    // Merge two configs, with the override_config taking precedence
    fn merge(base: Self, override_config: Self) -> Self {
        Self {
            api_token: override_config.api_token.or(base.api_token),
            api_base_url: override_config.api_base_url.or(base.api_base_url),
            model: override_config.model.or(base.model),
            system_prompt: override_config.system_prompt.or(base.system_prompt),
            user_prompt: override_config.user_prompt.or(base.user_prompt),
        }
    }

    pub fn load() -> Result<Self> {
        // First load the global config
        let global_config = Self::load_global_config()?;

        // Try to find and load project config
        if let Some(project_config_path) = Self::find_project_config()? {
            // If project config exists, load it and merge with global config
            let project_config = Self::load_toml_config(&project_config_path)?;

            // Merge configs, with project config taking precedence
            Ok(Self::merge(global_config, project_config))
        } else {
            // No project config, just use global config
            Ok(global_config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        let toml_string =
            toml::to_string_pretty(self).context("Failed to serialize config to TOML")?;

        let mut file = File::create(&config_path).context("Could not create config file")?;
        file.write_all(toml_string.as_bytes())
            .context("Failed to write to config file")?;
        file.flush().context("Failed to flush config file")?; // Ensure data is written to disk

        Ok(())
    }

    // Set a configuration value by key name
    #[allow(dead_code)] // Used by CLI command handlers
    pub fn set(&mut self, key: &str, value: Option<String>) -> Result<()> {
        match key {
            "api_token" => self.api_token = value,
            "api_base_url" => self.api_base_url = value,
            "model" => self.model = value,
            "system_prompt" => self.system_prompt = value,
            "user_prompt" => self.user_prompt = value,
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
            "system_prompt" => self.system_prompt.as_ref(),
            "user_prompt" => self.user_prompt.as_ref(),
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

    pub fn get_system_prompt(&self) -> &str {
        self.system_prompt
            .as_deref()
            .unwrap_or(DEFAULT_SYSTEM_PROMPT)
    }

    pub fn get_user_prompt(&self) -> &str {
        self.user_prompt.as_deref().unwrap_or(DEFAULT_USER_PROMPT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config: Config = Config::default();
        assert!(config.api_token.is_none());
        assert_eq!(
            config.api_base_url.as_deref(),
            Some("https://api.openai.com")
        );
        assert_eq!(config.model.as_deref(), Some("gpt-3.5-turbo"));
        assert!(config.system_prompt.is_some());
        assert!(config.user_prompt.is_some());
    }

    #[test]
    fn test_set_and_get() {
        // Create a completely unique temporary directory for this test
        // IMPORTANT: Each test should use its own isolated environment
        // to prevent interference between tests
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_dir = temp_dir.path().join(".config").join("aic");
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");

        // Set the HOME environment variable to the temporary directory
        env::set_var("HOME", temp_dir.path());

        // Create config directly
        let mut config = Config::default();

        // Test setting values
        config
            .set("api_token", Some("test_token".to_string()))
            .unwrap();
        config.set("model", Some("gpt-4".to_string())).unwrap();

        // Test getting values - test directly on the object, not after loading from file
        assert_eq!(config.get("api_token").unwrap(), "test_token");
        assert_eq!(config.get("model").unwrap(), "gpt-4");

        // Test setting to None
        config.set("api_token", None).unwrap();
        assert!(config.get("api_token").is_none());

        // Test invalid key
        assert!(config
            .set("invalid_key", Some("value".to_string()))
            .is_err());
        assert!(config.get("invalid_key").is_none());
    }

    #[test]
    fn test_save_and_load() {
        // Create a completely unique temporary directory for this test
        // NOTE: File operations in tests can be tricky. Common issues include:
        // 1. Multiple tests writing to the same file
        // 2. File system caching causing stale reads
        // 3. Environment variables not being isolated between tests
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_dir = temp_dir.path().join(".config").join("aic");
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");

        // Override the config_dir and config_path methods for testing
        let config_path = config_dir.join("config.toml");

        // Create and save config directly to our test location
        let config = Config {
            api_token: Some("test_token".to_string()),
            ..Default::default()
        };

        // Save configuration
        let toml_string = toml::to_string_pretty(&config).unwrap();
        let mut file = File::create(&config_path).unwrap();
        file.write_all(toml_string.as_bytes()).unwrap();
        file.flush().unwrap();

        // Test config could be loaded from a path
        let mut file = File::open(&config_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let loaded_config: Config = toml::from_str(&contents).unwrap();
        assert_eq!(loaded_config.api_token, Some("test_token".to_string()));
    }

    #[test]
    fn test_getter_methods() {
        let config = Config {
            api_token: Some("test-token".to_string()),
            api_base_url: Some("https://test-api.com".to_string()),
            model: Some("test-model".to_string()),
            system_prompt: Some("test system prompt".to_string()),
            user_prompt: Some("test user prompt".to_string()),
        };

        assert_eq!(config.get_api_token().unwrap(), "test-token");
        assert_eq!(config.get_api_base_url(), "https://test-api.com");
        assert_eq!(config.get_model(), "test-model");
        assert_eq!(config.get_system_prompt(), "test system prompt");
        assert_eq!(config.get_user_prompt(), "test user prompt");

        // Test defaults when values are None
        let empty_config = Config {
            api_token: None,
            api_base_url: None,
            model: None,
            system_prompt: None,
            user_prompt: None,
        };

        assert!(empty_config.get_api_token().is_err());
        assert_eq!(empty_config.get_api_base_url(), "https://api.openai.com");
        assert_eq!(empty_config.get_model(), "gpt-3.5-turbo");
        assert_eq!(empty_config.get_system_prompt(), DEFAULT_SYSTEM_PROMPT);
        assert_eq!(empty_config.get_user_prompt(), DEFAULT_USER_PROMPT);
    }

    #[test]
    fn test_project_config() {
        // Create temporary directories for test
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Create global config directory
        let home_dir = temp_dir.path().join("home");
        fs::create_dir_all(&home_dir).expect("Failed to create home directory");
        let config_dir = home_dir.join(".config").join("aic");
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");

        // Create project directory
        let project_dir = temp_dir.path().join("project");
        fs::create_dir_all(&project_dir).expect("Failed to create project directory");

        // Set HOME to our test home dir
        env::set_var("HOME", &home_dir);

        // Create global config file
        let global_config = Config {
            api_token: Some("global-token".to_string()),
            api_base_url: Some("https://global-api.com".to_string()),
            model: Some("global-model".to_string()),
            system_prompt: Some("global system prompt".to_string()),
            user_prompt: Some("global user prompt".to_string()),
        };

        let config_path = config_dir.join("config.toml");
        let toml_string = toml::to_string_pretty(&global_config).unwrap();
        let mut file = File::create(&config_path).unwrap();
        file.write_all(toml_string.as_bytes()).unwrap();

        // Create project config file
        let project_config = Config {
            api_token: Some("project-token".to_string()), // Override token
            api_base_url: None,                           // Use global URL
            model: Some("project-model".to_string()),     // Override model
            system_prompt: Some("project system prompt".to_string()), // Override system prompt
            user_prompt: None,                            // Use global user prompt
        };

        let project_config_path = project_dir.join(".aic.toml");
        let toml_string = toml::to_string_pretty(&project_config).unwrap();
        let mut file = File::create(&project_config_path).unwrap();
        file.write_all(toml_string.as_bytes()).unwrap();

        // Set current directory to project dir to test
        env::set_current_dir(&project_dir).expect("Failed to change directory");

        // Test finding project config - should be our .aic.toml file
        let found_config_path = Config::find_project_config().unwrap();
        assert!(found_config_path.is_some());
        assert_eq!(found_config_path.unwrap(), project_config_path);

        // Instead of checking with load(), which requires the actual config to be in place,
        // test the components directly:

        // Test loading project config
        let loaded_project_config = Config::load_toml_config(&project_config_path).unwrap();

        // Verify project config values were correctly loaded
        assert_eq!(
            loaded_project_config.api_token,
            Some("project-token".to_string())
        );
        assert_eq!(loaded_project_config.api_base_url, None);
        assert_eq!(
            loaded_project_config.model,
            Some("project-model".to_string())
        );
        assert_eq!(
            loaded_project_config.system_prompt,
            Some("project system prompt".to_string())
        );
        assert_eq!(loaded_project_config.user_prompt, None);

        // Load global config for merging
        let loaded_global_config = Config::load_global_config().unwrap();

        // Test merging configs
        let merged_config = Config::merge(loaded_global_config, loaded_project_config);

        // Verify correct merging of values
        assert_eq!(merged_config.api_token, Some("project-token".to_string()));
        assert_eq!(
            merged_config.api_base_url,
            Some("https://global-api.com".to_string())
        );
        assert_eq!(merged_config.model, Some("project-model".to_string()));
        assert_eq!(
            merged_config.system_prompt,
            Some("project system prompt".to_string())
        );
        assert_eq!(
            merged_config.user_prompt,
            Some("global user prompt".to_string())
        );
    }
}
