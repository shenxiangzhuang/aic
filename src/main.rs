mod cli;
mod commands;
mod config;
mod git;
mod llm;
mod ui;

use anyhow::Result;
use cli::parse_args;
use commands::{generate_commit, handle_config_command};
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::load()?;

    // Parse CLI arguments
    let cli = parse_args();

    // Process commands or default behavior
    match &cli.command {
        Some(cli::Commands::Generate {
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
        Some(cli::Commands::Config(config_cmd)) => {
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
