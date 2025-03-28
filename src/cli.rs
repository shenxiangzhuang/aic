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
    },
}

pub fn parse_args() -> Cli {
    Cli::parse()
}
