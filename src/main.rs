mod cli;
mod config;
mod git;
mod llm;

use anyhow::{Context, Result};
use cli::Commands;
use colored::Colorize;

async fn generate_commit(prompt: Option<String>) -> Result<()> {
    println!("{}", "Generating commit message...".blue());

    // Get git diff
    let diff = git::get_diff().context("Failed to get git diff")?;

    if diff.is_empty() {
        println!(
            "{}",
            "No staged changes detected in the git repository.".yellow()
        );
        println!(
            "{}",
            "Please add your changes with 'git add' before generating a commit message.".yellow()
        );
        return Ok(());
    }

    // Get API token
    let api_token = config::get_api_token()?;

    // Use custom prompt or default
    let system_prompt = prompt.unwrap_or_else(|| config::get_default_prompt());

    // Generate commit message
    let commit_message = llm::generate_commit_message(&diff, &system_prompt, &api_token).await?;

    // Print the result
    println!("\n{}", "Generated commit message:".green());
    println!("{}", commit_message);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file if it exists
    dotenv::dotenv().ok();

    // Parse CLI arguments
    let cli = cli::parse_args();

    // Process commands or default behavior
    match &cli.command {
        Some(Commands::Generate { prompt }) => {
            generate_commit(prompt.clone()).await?;
        }
        None => {
            // No subcommand provided, default to generate behavior
            generate_commit(cli.prompt).await?;
        }
    }

    Ok(())
}
