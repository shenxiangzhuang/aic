use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    author,
    version,
    about = "AI-powered commit message generator",
    long_about = "A CLI tool that uses AI to generate meaningful commit messages based on git diffs."
)]
pub struct Cli {
    /// Custom system prompt for commit message style when used without subcommand
    #[arg(
        short,
        long,
        help = "Custom system prompt for commit message style",
        long_help = "Provide a custom instruction to guide the AI in generating commit messages.\n\
        For example: \"Write commit messages in conventional commit format\" or\n\
        \"Focus on explaining why changes were made rather than what was changed.\""
    )]
    pub prompt: Option<String>,

    /// Base URL for the OpenAI-compatible API
    #[arg(
        long,
        help = "Base URL for the OpenAI-compatible API",
        long_help = "Specify a custom base URL for the OpenAI-compatible API.\n\
        This allows using alternative providers like DeepSeek or local models."
    )]
    pub api_base: Option<String>,

    /// Model to use for generating commit messages
    #[arg(
        long,
        help = "Model to use for generating commit messages",
        long_help = "Specify the model to use for generating commit messages."
    )]
    pub model: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a commit message based on git diff
    #[command(
        long_about = "Analyzes your staged git changes and generates a meaningful commit message using AI.\n\
        Make sure to stage your changes with 'git add' before running this command.\n\
        You can customize the style of commit messages using the --prompt option."
    )]
    Generate {
        /// Custom system prompt for commit message style
        #[arg(
            short,
            long,
            help = "Custom system prompt for commit message style",
            long_help = "Provide a custom instruction to guide the AI in generating commit messages.\n\
            For example: \"Write commit messages in conventional commit format\" or\n\
            \"Focus on explaining why changes were made rather than what was changed.\""
        )]
        prompt: Option<String>,

        /// Base URL for the OpenAI-compatible API
        #[arg(
            long,
            help = "Base URL for the OpenAI-compatible API",
            long_help = "Specify a custom base URL for the OpenAI-compatible API.\n\
            This allows using alternative providers like DeepSeek or local models."
        )]
        api_base: Option<String>,

        /// Model to use for generating commit messages
        #[arg(
            long,
            help = "Model to use for generating commit messages",
            long_help = "Specify the model to use for generating commit messages."
        )]
        model: Option<String>,
    },

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

        /// Default system prompt for commit message generation
        #[arg(long, help = "Default system prompt for commit message generation")]
        default_prompt: Option<String>,
    },

    /// List all configuration values
    List,
}

pub fn parse_args() -> Cli {
    Cli::parse()
}
