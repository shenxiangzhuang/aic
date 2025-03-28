use crate::config::Config;
use colored::Colorize;

/// Print the application header
pub fn print_header() {
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
}

/// Print configuration in a formatted table
pub fn print_config_table(config: &Config) {
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
}
