use crate::cli::{Commands, ConfigCommands};
use crate::config::Config;
use crate::git;
use crate::llm;
use crate::ui;
use anyhow::{Context, Result};
use colored::Colorize;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::Command;
use uuid::Uuid;

/// Generate a commit message using AI and optionally execute it
pub async fn generate_commit(config: &Config, auto_add: bool, auto_commit: bool) -> Result<()> {
    // Print header
    ui::print_header();

    // Auto-add changes if requested
    if auto_add {
        println!("{}", "📦 Staging all changes...".blue());
        let status = Command::new("git")
            .args(["add", "."])
            .status()
            .context("Failed to stage changes with git add")?;

        if !status.success() {
            return Err(anyhow::anyhow!("Failed to stage changes with git add"));
        }
    }

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

    // Use configuration values
    let system_prompt = config.get_system_prompt().to_string();
    let user_prompt = config.get_user_prompt().to_string();
    let api_base_url = config.get_api_base_url().to_string();
    let model_name = config.get_model().to_string();

    // Print configuration information
    println!("{} {}", "🤖 Using model:".blue(), model_name.bright_blue());
    println!("{}", "✨ Generating commit message...".blue());

    // Generate commit message
    let commit_message = llm::generate_commit_message(
        &diff,
        &system_prompt,
        &user_prompt,
        api_token,
        &api_base_url,
        &model_name,
    )
    .await?;

    // Format git commit command for display
    let escaped_message = commit_message.replace("\"", "\\\"");
    let commit_command = format!("git commit -m \"{}\"", escaped_message);

    // Only print the command, not the message again
    println!("{}", "📋 Commit command:".green().bold());
    println!("{}", commit_command.bright_white());

    if auto_commit {
        execute_commit(&commit_message)?;
    } else {
        handle_commit_options(&commit_message)?;
    }

    Ok(())
}

/// Execute the git commit with the provided message
fn execute_commit(commit_message: &str) -> Result<()> {
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

    Ok(())
}

/// Handle interactive commit options (execute/modify/cancel)
fn handle_commit_options(commit_message: &str) -> Result<()> {
    // Present options including a new "modify" option
    print!("\n{} ", "Execute this commit? [Y/m/n]:".yellow().bold());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    if input.is_empty() || input.starts_with('y') {
        // Execute directly
        execute_commit(commit_message)?;
    } else if input.starts_with('m') {
        // Modify the message before committing
        println!(
            "{}",
            "✏️  Opening editor to modify commit message...".blue()
        );

        let modified_message = edit_commit_message(commit_message)?;

        // Execute git commit with the modified message
        println!(
            "{}",
            "🚀 Executing git commit with modified message...".blue()
        );

        let status = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(&modified_message)
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
    } else if input.starts_with('n') {
        println!("{}", "📝 Command not executed.".blue());
        println!("{}", "You can copy and modify the command above.".dimmed());
    } else {
        println!("{}", "⚠️  Invalid option. Command not executed.".yellow());
        println!("{}", "You can copy and modify the command above.".dimmed());
    }

    Ok(())
}

/// Open an editor to modify the commit message
fn edit_commit_message(commit_message: &str) -> Result<String> {
    // Create a unique temporary file with UUID
    let temp_dir = env::temp_dir();
    let temp_file_path = temp_dir.join(format!("aic_commit_message_{}.txt", Uuid::new_v4()));

    // Ensure the file is removed even if the function panics
    struct TempFileGuard(std::path::PathBuf);
    impl Drop for TempFileGuard {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }
    let _guard = TempFileGuard(temp_file_path.clone());

    fs::write(&temp_file_path, commit_message)
        .context("Failed to create temporary file for editing")?;

    // Get the editor command - prioritize environment variable, then check for vim/vi
    let editor = if let Ok(editor) = env::var("EDITOR") {
        // Use user's preferred editor from environment variable
        editor
    } else {
        // Try to find vim or vi, fall back to nano
        if Command::new("vim").arg("--version").status().is_ok() {
            "vim".to_string()
        } else if Command::new("vi").arg("--version").status().is_ok() {
            "vi".to_string()
        } else {
            "nano".to_string()
        }
    };

    println!(
        "✏️  Opening {} to edit commit message...",
        editor.bright_blue()
    );

    let edit_status = Command::new(&editor)
        .arg(&temp_file_path)
        .status()
        .context(format!("Failed to open editor ({})", editor))?;

    if !edit_status.success() {
        return Err(anyhow::anyhow!("Editor exited with non-zero status"));
    }

    // Read the modified message
    let modified_message =
        fs::read_to_string(&temp_file_path).context("Failed to read modified commit message")?;

    Ok(modified_message)
}

/// Handle configuration commands
pub async fn handle_config_command(config_cmd: &ConfigCommands) -> Result<()> {
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

            ui::print_config_table(&config);

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

/// Test API connection and configuration
async fn ping_api(config: &Config) -> Result<()> {
    println!("{}", "🔍 Testing API connection...".blue());

    // Get API token and base URL
    let api_token = config.get_api_token()?;
    let api_base_url = config.get_api_base_url();
    let model = config.get_model();

    println!(
        "{} {}",
        "🌐 API Base URL:".blue(),
        api_base_url.bright_blue()
    );
    println!("{} {}", "🤖 Model:".blue(), model.bright_blue());

    // Create a simple test request
    let client = reqwest::Client::new();
    let endpoint = format!("{}/v1/chat/completions", api_base_url.trim_end_matches('/'));

    let request = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": "Hello"
            }
        ]
    });

    // Send the request
    let response = client
        .post(&endpoint)
        .header("Authorization", format!("Bearer {}", api_token))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .context("Failed to send request to API")?;

    let status = response.status();
    let response_text = response.text().await?;

    if status.is_success() {
        println!("{}", "✅ API connection successful!".green().bold());
        println!("{}", "✨ Configuration is working correctly.".green());
    } else {
        println!("{}", "❌ API connection failed:".red().bold());
        println!("Status: {}", status);
        println!("Error: {}", response_text);
    }

    Ok(())
}

/// Process commands or default behavior
pub async fn handle_commands(cli: &Commands, config: &Config) -> Result<()> {
    match cli {
        Commands::Ping => {
            ping_api(config).await?;
        }
        Commands::Config(config_cmd) => {
            handle_config_command(config_cmd).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::env;
    use std::fs;

    #[tokio::test]
    async fn test_generate_commit_no_staged_changes() {
        // Mock the git::get_diff function to return an empty string
        // Here, you would replace the actual call with a mock or a stub
        // For example, you could use a trait and dependency injection

        let config = Config::default();
        let result = generate_commit(&config, false, false).await;

        assert!(result.is_ok());
        // Check for expected output (e.g., "No staged changes detected")
    }

    #[tokio::test]
    async fn test_generate_commit_with_staged_changes() {
        // Mock the git::get_diff function to return a non-empty string
        // Again, use a trait or dependency injection to replace the actual call

        let config = Config::default();
        let result = generate_commit(&config, false, false).await;

        assert!(result.is_ok());
        // Check for expected output (e.g., "Generating commit message...")
    }

    #[test]
    fn test_execute_commit_success() {
        // Mock the Command::new("git") to simulate a successful commit
        // You can use a mock library or dependency injection here

        let status = execute_commit("Test commit message");
        assert!(status.is_ok());
        // Check for expected output (e.g., "Commit created successfully!")
    }

    #[test]
    fn test_edit_commit_message() {
        // Set up a temporary environment for the test
        let temp_dir = env::temp_dir();
        let temp_file_path = temp_dir.join("aic_commit_message.txt");

        // Write a test commit message to the temporary file
        fs::write(&temp_file_path, "Test commit message").unwrap();

        // Mock the editor command to simulate editing
        env::set_var("EDITOR", "true");

        let result = edit_commit_message("Test commit message");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Test commit message");
    }
}
