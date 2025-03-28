mod cli;
mod config;
mod git;
mod llm;

use anyhow::{Context, Result};
use cli::{Commands, ConfigCommands}; // Removed unused Cli import
use colored::Colorize;
use config::Config;
use std::io::{self, Write};
use std::process::Command;

async fn generate_commit(
    config: &Config,
    prompt: Option<String>,
    api_base: Option<String>,
    model: Option<String>,
    execute: bool,
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

    // Format git commit command for display
    let escaped_message = commit_message.replace("\"", "\\\"");
    let commit_command = format!("git commit -m \"{}\"", escaped_message);

    println!("\n{}", "Git command:".green());
    println!("{}", commit_command);

    if execute {
        println!("\n{}", "Executing git commit...".blue());

        // Execute the git commit command
        let status = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(commit_message)
            .status()
            .context("Failed to execute git commit command")?;

        if status.success() {
            println!("{}", "Commit created successfully.".green());
        } else {
            println!("{}", "Git commit command failed.".red());
            if let Some(code) = status.code() {
                println!("Exit code: {}", code);
            }
        }
    } else {
        // Ask if the user wants to execute the command
        println!("\n{}", "Execute this commit command? [y/N]: ".yellow());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() == "y" {
            // Execute the git commit command
            let status = Command::new("git")
                .arg("commit")
                .arg("-m")
                .arg(commit_message)
                .status()
                .context("Failed to execute git commit command")?;

            if status.success() {
                println!("{}", "Commit created successfully.".green());
            } else {
                println!("{}", "Git commit command failed.".red());
                if let Some(code) = status.code() {
                    println!("Exit code: {}", code);
                }
            }
        } else {
            println!(
                "{}",
                "Command not executed. You can copy the command above and run it manually.".blue()
            );
        }
    }

    Ok(())
}

async fn handle_config_command(config_cmd: &ConfigCommands) -> Result<()> {
    // Renamed parameter to avoid warning
    match config_cmd {
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
        ConfigCommands::Setup {
            api_token,
            api_base_url,
            model,
            default_prompt,
        } => {
            let mut config = Config::load()?;
            let mut changes = 0;

            // Update each value if provided
            if let Some(token) = api_token {
                config.set("api_token", Some(token.clone()))?;
                println!("Set api_token to: {}", token);
                changes += 1;
            }

            if let Some(url) = api_base_url {
                config.set("api_base_url", Some(url.clone()))?;
                println!("Set api_base_url to: {}", url);
                changes += 1;
            }

            if let Some(model_name) = model {
                config.set("model", Some(model_name.clone()))?;
                println!("Set model to: {}", model_name);
                changes += 1;
            }

            if let Some(prompt) = default_prompt {
                config.set("default_prompt", Some(prompt.clone()))?;
                println!("Set default_prompt to: {}", prompt);
                changes += 1;
            }

            if changes == 0 {
                println!(
                    "{}",
                    "No configuration values were provided to set.".yellow()
                );
                println!("Usage: aic config setup --api-token <TOKEN> --api-base-url <URL> --model <MODEL>");
            } else {
                println!("{}", "Configuration updated successfully.".green());
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
            execute,
        }) => {
            generate_commit(
                &config,
                prompt.clone(),
                api_base.clone(),
                model.clone(),
                *execute,
            )
            .await?;
        }
        Some(Commands::Config(config_cmd)) => {
            handle_config_command(config_cmd).await?;
        }
        None => {
            // No subcommand provided, default to generate behavior using cli directly
            generate_commit(
                &config,
                cli.prompt.clone(),
                cli.api_base.clone(),
                cli.model.clone(),
                cli.execute.unwrap_or(false),
            )
            .await?;
        }
    }

    Ok(())
}
