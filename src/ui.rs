use crate::config::Config;
use colored::Colorize;
use prettytable::{row, Table};

/// Print the application header
pub fn print_header() {
    println!(
        "{}",
        "╭─────────────────────────────────────╮\n\
         │     AI Commit Message Generator     │\n\
         ╰─────────────────────────────────────╯"
            .bright_blue()
    );
}

/// Print configuration in a formatted table
pub fn print_config_table(config: &Config) {
    let mut table = Table::new();
    table.add_row(row!["Setting", "Value"]);

    // API Token (with masking for security)
    let token_display = if let Some(token) = config.get("api_token") {
        if token.len() > 8 {
            format!("{}•••••", &token[0..4])
        } else {
            "•••••••".to_string()
        }
    } else {
        "<not set>".to_string()
    };
    table.add_row(row!["api_token", token_display]);

    // Other settings
    table.add_row(row!["api_base_url", config.get_api_base_url()]);
    table.add_row(row!["model", config.get_model()]);

    // System prompt (truncated if too long)
    let system_prompt = config.get_system_prompt();
    let display_system_prompt = if system_prompt.chars().count() > 12 {
        format!("{}...", system_prompt.chars().take(12).collect::<String>())
    } else {
        system_prompt.to_string()
    };
    table.add_row(row!["system_prompt", display_system_prompt]);

    // User prompt (truncated if too long)
    let user_prompt = config.get_user_prompt();
    let display_user_prompt = if user_prompt.chars().count() > 12 {
        format!("{}...", user_prompt.chars().take(12).collect::<String>())
    } else {
        user_prompt.to_string()
    };
    table.add_row(row!["user_prompt", display_user_prompt]);

    table.printstd();
}
