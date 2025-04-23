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
use tempfile::Builder;
use uuid::Uuid;

/// Generate a commit message using AI and optionally execute it and push
pub async fn generate_commit(
    config: &Config,
    auto_add: bool,
    auto_commit: bool,
    auto_push: bool,
) -> Result<()> {
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
    let diff: String = git::get_diff().context("Failed to get git diff")?;

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
        // Push changes if auto_push is enabled
        if auto_push {
            git::push_changes()?;
        }
    } else {
        handle_commit_options(&commit_message, auto_push)?;
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
fn handle_commit_options(commit_message: &str, auto_push: bool) -> Result<()> {
    // Present options including a new "modify" option
    print!("\n{} ", "Execute this commit? [Y/m/n]:".yellow().bold());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    if input.is_empty() || input.starts_with('y') {
        // Execute directly
        execute_commit(commit_message)?;
        // Push if auto_push is enabled and commit was successful
        if auto_push {
            git::push_changes()?;
        }
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
            // Push if auto_push is enabled and commit was successful
            if auto_push {
                git::push_changes()?;
            }
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
    let tmp_dir = Builder::new().prefix("edit_commit").tempdir()?;
    let tmp_file_path = tmp_dir
        .path()
        .join(format!("aic_commit_message_{}.txt", Uuid::new_v4()));

    fs::write(&tmp_file_path, commit_message)
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
        .arg(&tmp_file_path)
        .status()
        .context(format!("Failed to open editor ({})", editor))?;

    if !edit_status.success() {
        return Err(anyhow::anyhow!("Editor exited with non-zero status"));
    }

    // Read the modified message
    let modified_message =
        fs::read_to_string(&tmp_file_path).context("Failed to read modified commit message")?;

    // drop tmp file
    drop(tmp_file_path);
    tmp_dir.close()?;

    Ok(modified_message)
}

/// Handle configuration commands
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
            system_prompt,
            user_prompt,
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

            if let Some(system_prompt) = system_prompt {
                config.set("system_prompt", Some(system_prompt.clone()))?;
                println!("✓ Set system_prompt to: {}", system_prompt);
                changes += 1;
            }

            if let Some(users_prompt) = user_prompt {
                config.set("user_prompt", Some(users_prompt.clone()))?;
                println!("✓ Set user_prompt to: {}", users_prompt);
                changes += 1;
            }

            if changes == 0 {
                println!("{}", "ℹ️ No changes made to configuration.".blue());
            } else {
                println!("{}", "✨ Configuration updated successfully.".green());
            }
        }
        ConfigCommands::Show => {
            // Load configuration
            let config = Config::load()?;

            // Get paths for the different config files
            let global_config_path = Config::config_path()?;
            let project_config_path = Config::find_project_config()?;

            // Use the UI module to display configuration information
            ui::print_config_sources(&global_config_path, &project_config_path);
            ui::print_config_table(&config);
        }
        ConfigCommands::List => {
            // This is just an alias for Show in this implementation
            // Use Box::pin to avoid infinitely sized future from recursion
            Box::pin(handle_config_command(&ConfigCommands::Show)).await?;
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
    use crate::cli::ConfigCommands;
    use std::env;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use tempfile::Builder;
    use toml;

    #[tokio::test]
    async fn test_generate_commit_no_staged_changes() {
        let tmp_dir = Builder::new()
            .prefix("test_generate_commit_no_staged_changes")
            .tempdir()
            .unwrap();
        env::set_current_dir(&tmp_dir).unwrap();

        let result = generate_commit(&Config::default(), false, false, false).await;

        assert!(result.is_ok());
        assert!(matches!(result, Ok(())));
    }

    #[tokio::test]
    async fn test_generate_commit_no_staged_changes_with_add() {
        let tmp_dir = Builder::new()
            .prefix("test_generate_commit_no_staged_changes_with_add")
            .tempdir()
            .unwrap();
        env::set_current_dir(&tmp_dir).unwrap();

        let result = generate_commit(&Config::default(), true, false, false).await;
        assert!(result.is_err());

        // Match and check the error message
        if let Err(err) = result {
            assert_eq!(err.to_string(), "Failed to stage changes with git add");
        }
    }

    #[test]
    fn test_execute_commit_success() {
        let tmp_dir = Builder::new()
            .prefix("test_execute_commit_success")
            .tempdir()
            .unwrap();
        env::set_current_dir(&tmp_dir).unwrap();

        // Initialize git repository
        Command::new("git")
            .args(["init"])
            .current_dir(&tmp_dir)
            .output()
            .unwrap();

        // Create new file
        Command::new("touch")
            .args(["hello.py"])
            .current_dir(&tmp_dir)
            .output()
            .unwrap();

        // Add
        Command::new("git")
            .args(["add", "."])
            .current_dir(&tmp_dir)
            .output()
            .unwrap();

        let status: std::result::Result<(), anyhow::Error> = execute_commit("Test commit message");
        assert!(status.is_ok());
    }

    #[test]
    fn test_edit_commit_message() {
        let tmp_dir = Builder::new()
            .prefix("test_edit_commit_message")
            .tempdir()
            .unwrap();
        let tmp_file_path = tmp_dir.path().join("aic_commit_message.txt");
        env::set_current_dir(&tmp_dir).unwrap();

        // Write a test commit message to the temporary file
        fs::write(&tmp_file_path, "Test commit message").unwrap();

        // Mock the editor command to simulate editing
        env::set_var("EDITOR", "true");

        let result = edit_commit_message("New test commit message");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "New test commit message");
    }

    #[test]
    fn test_handle_config_command_invalid_key() {
        let mut config = Config::default();
        assert!(config
            .set("random_key", Some("random_value".to_string()))
            .is_err());
    }

    #[tokio::test]
    async fn test_handle_config_command_get() {
        let tmp_dir = Builder::new()
            .prefix("test_handle_config_command_get")
            .tempdir()
            .unwrap();
        let config_dir = tmp_dir.path().join(".config").join("aic");
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");

        // Set the HOME environment variable to the temporary directory
        env::set_var("HOME", tmp_dir.path());

        // Set current directory to test directory
        env::set_current_dir(tmp_dir.path()).expect("Failed to change directory");

        // Initialize a git repository to have a proper .git boundary
        Command::new("git")
            .args(["init"])
            .output()
            .expect("Failed to init git repo");

        // Test getting a default key
        let result = handle_config_command(&ConfigCommands::Get {
            key: "system_prompt".to_string(),
        })
        .await;
        assert!(result.is_ok());

        // Test getting a non-existent key
        let result = handle_config_command(&ConfigCommands::Get {
            key: "non_existent".to_string(),
        })
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_config_command_set() {
        let tmp_dir = Builder::new()
            .prefix("test_handle_config_command_set")
            .tempdir()
            .unwrap();
        let config_dir = tmp_dir.path().join(".config").join("aic");
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");

        // Set the HOME environment variable to the temporary directory
        env::set_var("HOME", tmp_dir.path());

        // Set current directory to test directory
        env::set_current_dir(tmp_dir.path()).expect("Failed to change directory");

        // Initialize a git repository to have a proper .git boundary
        Command::new("git")
            .args(["init"])
            .output()
            .expect("Failed to init git repo");

        // Test setting a value
        let result = handle_config_command(&ConfigCommands::Set {
            key: "model".to_string(),
            value: Some("test_model".to_string()),
        })
        .await;
        assert!(result.is_ok());

        // Verify the value was set
        let config = Config::load().unwrap();
        assert_eq!(config.get("model"), Some(&"test_model".to_string()));

        // Test unsetting a value
        let result = handle_config_command(&ConfigCommands::Set {
            key: "model".to_string(),
            value: None,
        })
        .await;
        assert!(result.is_ok());

        // Verify the value was unset
        let config = Config::load().unwrap();
        assert_eq!(config.get("model"), None);
    }

    #[tokio::test]
    async fn test_handle_config_command_setup() {
        let tmp_dir = Builder::new()
            .prefix("test_handle_config_command_setup")
            .tempdir()
            .unwrap();
        let config_dir = tmp_dir.path().join(".config").join("aic");
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");

        // Set the HOME environment variable to the temporary directory
        env::set_var("HOME", tmp_dir.path());

        // Set current directory to test directory
        env::set_current_dir(tmp_dir.path()).expect("Failed to change directory");

        // Initialize a git repository to have a proper .git boundary
        Command::new("git")
            .args(["init"])
            .output()
            .expect("Failed to init git repo");

        // Test setting multiple values
        let result = handle_config_command(&ConfigCommands::Setup {
            api_token: Some("test_token".to_string()),
            api_base_url: Some("https://test.api".to_string()),
            model: Some("test-model".to_string()),
            system_prompt: Some("test system prompt".to_string()),
            user_prompt: Some("test user prompt".to_string()),
        })
        .await;
        assert!(result.is_ok());

        // Verify the values were set
        let config = Config::load().unwrap();
        assert_eq!(config.get("api_token"), Some(&"test_token".to_string()));
        assert_eq!(
            config.get("api_base_url"),
            Some(&"https://test.api".to_string())
        );
        assert_eq!(config.get("model"), Some(&"test-model".to_string()));
        assert_eq!(
            config.get("system_prompt"),
            Some(&"test system prompt".to_string())
        );
        assert_eq!(
            config.get("user_prompt"),
            Some(&"test user prompt".to_string())
        );

        // Test setup with no values (should not error)
        let result = handle_config_command(&ConfigCommands::Setup {
            api_token: None,
            api_base_url: None,
            model: None,
            system_prompt: None,
            user_prompt: None,
        })
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_config_command_list() {
        let tmp_dir = Builder::new()
            .prefix("test_handle_config_command_list")
            .tempdir()
            .unwrap();
        let config_dir = tmp_dir.path().join(".config").join("aic");
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");

        // Set the HOME environment variable to the temporary directory
        env::set_var("HOME", tmp_dir.path());

        // Set current directory to test directory
        env::set_current_dir(tmp_dir.path()).expect("Failed to change directory");

        // Initialize a git repository to have a proper .git boundary
        Command::new("git")
            .args(["init"])
            .output()
            .expect("Failed to init git repo");

        // Create a test config with some values
        let mut config = Config::default();
        config.set("model", Some("test_model".to_string())).unwrap();
        config.save().unwrap();

        // Test listing configuration
        let result = handle_config_command(&ConfigCommands::List).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_config_command_show() {
        let tmp_dir = Builder::new()
            .prefix("test_handle_config_show")
            .tempdir()
            .unwrap();

        // Create a home directory with a global config
        let home_dir = tmp_dir.path().join("home");
        let config_dir = home_dir.join(".config").join("aic");
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");

        // Create a project directory with a project config
        let project_dir = tmp_dir.path().join("project");
        fs::create_dir_all(&project_dir).expect("Failed to create project directory");

        // Set HOME to our test directory
        env::set_var("HOME", &home_dir);

        // Create a global config
        let global_config = Config {
            api_token: Some("global-token".to_string()),
            api_base_url: Some("https://global-api.com".to_string()),
            model: Some("global-model".to_string()),
            system_prompt: Some("global system prompt".to_string()),
            user_prompt: Some("global user prompt".to_string()),
        };

        let global_config_path = config_dir.join("config.toml");
        let toml_string = toml::to_string_pretty(&global_config).unwrap();
        let mut file = File::create(&global_config_path).unwrap();
        file.write_all(toml_string.as_bytes()).unwrap();

        // Create a project config
        let project_config = Config {
            api_token: None,
            api_base_url: None,
            model: Some("project-model".to_string()),
            system_prompt: Some("project system prompt".to_string()),
            user_prompt: None,
        };

        let project_config_path = project_dir.join(".aic.toml");
        let toml_string = toml::to_string_pretty(&project_config).unwrap();
        let mut file = File::create(&project_config_path).unwrap();
        file.write_all(toml_string.as_bytes()).unwrap();

        // Set current directory to project
        env::set_current_dir(&project_dir).expect("Failed to change directory");

        // Test the show command - we can only verify it executes without errors
        // Actual output would need to be captured and verified in a more complex test
        let result = handle_config_command(&ConfigCommands::Show).await;
        assert!(result.is_ok());
    }
}
