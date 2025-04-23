use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    author,
    version,
    about = "AI-powered commit message generator",
    long_about = "A CLI tool that uses AI to generate meaningful commit messages based on git diffs."
)]
pub struct Cli {
    /// Automatically stage all changes before generating commit message
    #[arg(
        short = 'a',
        long = "add",
        help = "Automatically stage all changes before generating commit message",
        long_help = "When provided, automatically stage all changes with 'git add .' before generating the commit message."
    )]
    pub auto_add: bool,

    /// Execute the git commit command automatically without confirmation
    #[arg(
        short = 'c',
        long = "commit",
        help = "Execute the git commit command automatically without confirmation",
        long_help = "When provided, automatically execute the git commit command without asking for confirmation."
    )]
    pub auto_commit: bool,

    /// Execute the git push command automatically after a successful commit
    #[arg(
        short = 'p',
        long = "push",
        help = "Execute the git push command automatically after a successful commit",
        long_help = "When provided, automatically execute 'git push' after a successful commit."
    )]
    pub auto_push: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Test API connection and configuration
    #[command(long_about = "Test the API connection and configuration settings.\n\
        This command will attempt to connect to the configured API endpoint and verify the token.")]
    Ping,

    /// Manage configuration settings
    #[command(subcommand)]
    Config(ConfigCommands),
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Get a configuration value
    Get {
        /// Configuration key to get (api_token, api_base_url, model, default_prompt)
        key: String,
    },

    /// Set a configuration value
    Set {
        /// Configuration key to set (api_token, api_base_url, model, default_prompt)
        key: String,

        /// Value to set for the key (omit to unset)
        value: Option<String>,
    },

    /// Set multiple configuration values at once for quick setup
    Setup {
        /// API token for authentication
        #[arg(long, help = "API token for authentication")]
        api_token: Option<String>,

        /// Base URL for the OpenAI-compatible API
        #[arg(long, help = "Base URL for the OpenAI-compatible API")]
        api_base_url: Option<String>,

        /// Model to use for generating commit messages
        #[arg(long, help = "Model to use for generating commit messages")]
        model: Option<String>,

        /// System prompt for commit message generation
        #[arg(long, help = "System prompt for commit message generation")]
        system_prompt: Option<String>,

        /// User prompt for commit message generation
        #[arg(long, help = "User prompt for commit message generation")]
        user_prompt: Option<String>,
    },

    /// Show current active configuration (merged global and project config if exists)
    Show,

    /// List all configuration values
    List,
}

pub fn parse_args() -> Cli {
    Cli::parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_command() {
        let args = Cli::parse_from(["program"]);
        assert!(args.command.is_none());
        assert!(!args.auto_commit);
        assert!(!args.auto_add);
        assert!(!args.auto_push);
    }

    #[test]
    fn test_auto_flags() {
        let args = Cli::parse_from(["program", "-a", "-c", "-p"]);
        assert!(args.auto_add);
        assert!(args.auto_commit);
        assert!(args.auto_push);
    }

    #[test]
    fn test_config_get() {
        let args = Cli::parse_from(["program", "config", "get", "api_token"]);
        assert!(!args.auto_add);
        assert!(!args.auto_commit);
        assert!(!args.auto_push);

        match args.command {
            Some(Commands::Config(ConfigCommands::Get { key })) => {
                assert_eq!(key, "api_token");
            }
            _ => panic!("Expected Config Get command"),
        }
    }

    #[test]
    fn test_config_set() {
        let args = Cli::parse_from(["program", "config", "set", "api_token", "test-token"]);
        assert!(!args.auto_add);
        assert!(!args.auto_commit);
        assert!(!args.auto_push);

        match args.command {
            Some(Commands::Config(ConfigCommands::Set { key, value })) => {
                assert_eq!(key, "api_token");
                assert_eq!(value, Some("test-token".to_string()));
            }
            _ => panic!("Expected Config Set command"),
        }
    }

    #[test]
    fn test_config_setup() {
        let args = Cli::parse_from([
            "program",
            "config",
            "setup",
            "--api-token",
            "test-token",
            "--model",
            "gpt-4",
            "--api-base-url",
            "https://api.example.com",
            "--system-prompt",
            "Test system prompt",
            "--user-prompt",
            "Test user prompt",
        ]);
        assert!(!args.auto_add);
        assert!(!args.auto_commit);
        assert!(!args.auto_push);

        match args.command {
            Some(Commands::Config(ConfigCommands::Setup {
                api_token,
                model,
                api_base_url,
                system_prompt,
                user_prompt,
            })) => {
                assert_eq!(api_token, Some("test-token".to_string()));
                assert_eq!(model, Some("gpt-4".to_string()));
                assert_eq!(api_base_url, Some("https://api.example.com".to_string()));
                assert_eq!(system_prompt, Some("Test system prompt".to_string()));
                assert_eq!(user_prompt, Some("Test user prompt".to_string()));
            }
            _ => panic!("Expected Config Setup command"),
        }
    }
}
