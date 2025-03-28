mod cli;
mod config;
mod git;
mod llm;

use anyhow::{Context, Result};
use cli::{Commands, ConfigCommands};
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
    // Print header with current time
    println!(
        "{}",
        "╭─────────────────────────────────────╮".bright_blue()
    );
    println!(
        "{}",
        "│     AI Commit Message Generator     │".bright_blue()
    );
    println!(
        "{}",
        "╰─────────────────────────────────────╯".bright_blue()
    );

    println!("{}", "🔍 Analyzing staged changes...".blue());

    // Get git diff
    let diff = git::get_diff().context("Failed to get git diff")?;

    if diff.is_empty() {
        println!(
            "{}",
            "⚠️  No staged changes detected in the git repository.".yellow()
        );
        println!(
            "{}",
            "   Please add your changes with 'git add' first.".yellow()
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
    println!("{} {}", "🤖 Using model:".blue(), model_name.bright_blue());
    println!("{}", "✨ Generating commit message...".blue());

    // Generate commit message
    let commit_message =
        llm::generate_commit_message(&diff, &system_prompt, api_token, &api_base_url, &model_name)
            .await?;

    // Format git commit command for display
    let escaped_message = commit_message.replace("\"", "\\\"");
    let commit_command = format!("git commit -m \"{}\"", escaped_message);

    // Only print the command, not the message again
    println!("{}", "📋 Commit command:".green().bold());
    println!("{}", commit_command.bright_white());

    if execute {
        println!("\n{}", "🚀 Executing git commit...".blue());

        // Execute the git commit command
        let status = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(commit_message)
            .status()
            .context("Failed to execute git commit command")?;

        if status.success() {
            println!("{}", "🎉 Commit created successfully!".green().bold());
        } else {
            println!("{}", "❌ Git commit command failed:".red().bold());
            if let Some(code) = status.code() {
                println!("Exit code: {}", code);
            }
        }
    } else {
        // Ask if the user wants to execute the command - on same line with Y as default
        print!("\n{} ", "Execute this commit? [Y/n]:".yellow().bold());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        // Execute if input is empty (just Enter) or starts with 'y'/'Y'
        let should_execute =
            input.trim().is_empty() || input.trim().to_lowercase().starts_with('y');

        if should_execute {
            println!("{}", "🚀 Executing git commit...".blue());

            // Execute the git commit command
            let status = Command::new("git")
                .arg("commit")
                .arg("-m")
                .arg(commit_message)
                .status()
                .context("Failed to execute git commit command")?;

            if status.success() {
                println!("{}", "🎉 Commit created successfully!".green().bold());
            } else {
                println!("{}", "❌ Git commit command failed:".red().bold());
                if let Some(code) = status.code() {
                    println!("Exit code: {}", code);
                }
            }
        }
    }

    Ok(())
}

async fn handle_config_command(config_cmd: &ConfigCommands) -> Result<()> {
    match config_cmd {
        ConfigCommands::Get { key } => {
            let config = Config::load()?;

            if let Some(value) = config.get(key) {
                println!("{}: {}", key.bright_blue(), value);
            } else {
                println!("{}: {}", key.bright_blue(), "<not set>".dimmed());
            }
        }
        ConfigCommands::Set { key, value } => {
            let mut config = Config::load()?;

            config.set(key, value.clone())?;

            if let Some(val) = value {
                println!("✓ Set {} to: {}", key.bright_blue(), val);
            } else {
                println!("✓ Unset {}", key.bright_blue());
            }
        }
        ConfigCommands::Setup {
            api_token,
            api_base_url,
            model,
            default_prompt,
        } => {
            println!("{}", "⚙️  Updating configuration...".blue());

            let mut config = Config::load()?;
            let mut changes = 0;

            // Update each value if provided
            if let Some(token) = api_token {
                config.set("api_token", Some(token.clone()))?;
                // Don't print the full token for security
                let masked_token = if token.len() > 8 {
                    format!("{}•••••", &token[0..4])
                } else {
                    "•••••••".to_string()
                };
                println!("✓ Set api_token to: {}", masked_token);
                changes += 1;
            }

            if let Some(url) = api_base_url {
                config.set("api_base_url", Some(url.clone()))?;
                println!("✓ Set api_base_url to: {}", url);
                changes += 1;
            }

            if let Some(model_name) = model {
                config.set("model", Some(model_name.clone()))?;
                println!("✓ Set model to: {}", model_name);
                changes += 1;
            }

            if let Some(prompt) = default_prompt {
                config.set("default_prompt", Some(prompt.clone()))?;
                println!("✓ Set default_prompt to: {}", prompt);
                changes += 1;
            }

            if changes == 0 {
                println!(
                    "{}",
                    "⚠️  No configuration values were provided to set.".yellow()
                );
                println!("{}", "Usage examples:".bright_blue());
                println!("  aic config setup --api-token <TOKEN> --api-base-url <URL>");
                println!(
                    "  aic config setup --model gpt-4-turbo --api-base-url https://api.openai.com"
                );
            } else {
                println!(
                    "{}",
                    "🎉 Configuration updated successfully!".green().bold()
                );
            }
        }
        ConfigCommands::List => {
            println!("{}", "⚙️  Current Configuration:".green().bold());
            let config = Config::load()?;

            println!(
                "{}",
                "┌───────────────┬──────────────────────────────────────┐".dimmed()
            );

            // API Token (with masking for security)
            print!("│ {:<13} │ ", "api_token".bright_blue());
            if let Some(token) = &config.api_token {
                if token.len() > 8 {
                    println!("{:<36} │", format!("{}•••••", &token[0..4]));
                } else {
                    println!("{:<36} │", "•••••••");
                }
            } else {
                println!("{:<36} │", "<not set>".dimmed());
            }

            // Base URL
            println!(
                "│ {:<13} │ {:<36} │",
                "api_base_url".bright_blue(),
                config.get_api_base_url()
            );

            // Model
            println!(
                "│ {:<13} │ {:<36} │",
                "model".bright_blue(),
                config.get_model()
            );

            // Default prompt (truncated if too long)
            let prompt = config.get_default_prompt();
            let display_prompt = if prompt.len() > 36 {
                format!("{}...", &prompt[0..33])
            } else {
                prompt.to_string()
            };
            println!(
                "│ {:<13} │ {:<36} │",
                "default_prompt".bright_blue(),
                display_prompt
            );

            println!(
                "{}",
                "└───────────────┴──────────────────────────────────────┘".dimmed()
            );

            println!("\n{}", "📁 Configuration file location:".blue());
            if let Ok(path) = Config::config_path() {
                println!("   {}", path.display());
            } else {
                println!("   <unknown>");
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
