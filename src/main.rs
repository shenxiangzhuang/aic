mod cli;
mod config;
mod git;
mod llm;

use anyhow::{Context, Result};
use cli::{Commands, ConfigCommands};
use colored::Colorize;
use config::Config;

async fn generate_commit(
    config: &Config,
    prompt: Option<String>,
    api_base: Option<String>,
    model: Option<String>,
) -> Result<()> {
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
    let api_token = config.get_api_token()?;

    // Use custom configurations or defaults
    let system_prompt = prompt.unwrap_or_else(|| config.get_default_prompt().to_string());
    let api_base_url = api_base.unwrap_or_else(|| config.get_api_base_url().to_string());
    let model_name = model.unwrap_or_else(|| config.get_model().to_string());

    // Print configuration information
    println!("{} {}", "Using model:".blue(), model_name);

    // Generate commit message
    let commit_message =
        llm::generate_commit_message(&diff, &system_prompt, api_token, &api_base_url, &model_name)
            .await?;

    // Print the result
    println!("\n{}", "Generated commit message:".green());
    println!("{}", commit_message);

    Ok(())
}

async fn handle_config_command(cmd: &ConfigCommands) -> Result<()> {
    match cmd {
        ConfigCommands::Get { key } => {
            let config = Config::load()?;

            if let Some(value) = config.get(key) {
                println!("{}: {}", key, value);
            } else {
                println!("{}: <not set>", key);
            }
        }
        ConfigCommands::Set { key, value } => {
            let mut config = Config::load()?;

            config.set(key, value.clone())?;

            if let Some(val) = value {
                println!("Set {} to: {}", key, val);
            } else {
                println!("Unset {}", key);
            }
        }
        ConfigCommands::List => {
            let config = Config::load()?;

            println!("{}", "Configuration:".green());
            println!(
                "api_token: {}",
                config.api_token.as_deref().unwrap_or("<not set>")
            );
            println!("api_base_url: {}", config.get_api_base_url());
            println!("model: {}", config.get_model());
            println!("default_prompt: {}", config.get_default_prompt());

            println!("\n{}", "Configuration file location:".blue());
            if let Ok(path) = Config::config_path() {
                println!("{}", path.display());
            } else {
                println!("<unknown>");
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::load()?;

    // Parse CLI arguments
    let cli = cli::parse_args();

    // Process commands or default behavior
    match &cli.command {
        Some(Commands::Generate {
            prompt,
            api_base,
            model,
        }) => {
            generate_commit(&config, prompt.clone(), api_base.clone(), model.clone()).await?;
        }
        Some(Commands::Config(config_cmd)) => {
            handle_config_command(config_cmd).await?;
        }
        None => {
            // No subcommand provided, default to generate behavior
            generate_commit(&config, cli.prompt, cli.api_base, cli.model).await?;
        }
    }

    Ok(())
}
