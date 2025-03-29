use crate::config::Config;
use colored::Colorize;

/// Print the application header
pub fn print_header() -> String {
    let header = format!(
        "{}\n{}\n{}",
        "╭─────────────────────────────────────╮".bright_blue(),
        "│     AI Commit Message Generator     │".bright_blue(),
        "╰─────────────────────────────────────╯".bright_blue()
    );
    println!("{}", header);
    header
}

/// Print configuration in a formatted table
pub fn print_config_table(config: &Config) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "{}\n",
        "┌───────────────┬──────────────────────────────────────┐".dimmed()
    ));

    // API Token (with masking for security)
    output.push_str(&format!("│ {:<13} │ ", "api_token".bright_blue()));
    if let Some(token) = &config.api_token {
        if token.len() > 8 {
            output.push_str(&format!("{:<36} │\n", format!("{}•••••", &token[0..4])));
        } else {
            output.push_str(&format!("{:<36} │\n", "•••••••"));
        }
    } else {
        output.push_str(&format!("{:<36} │\n", "<not set>".dimmed()));
    }

    // Base URL
    output.push_str(&format!(
        "│ {:<13} │ {:<36} │\n",
        "api_base_url".bright_blue(),
        config.get_api_base_url()
    ));

    // Model
    output.push_str(&format!(
        "│ {:<13} │ {:<36} │\n",
        "model".bright_blue(),
        config.get_model()
    ));

    // Default prompt (truncated if too long)
    let prompt = config.get_default_prompt();
    let display_prompt = if prompt.len() > 36 {
        format!("{}...", &prompt[0..33])
    } else {
        prompt.to_string()
    };
    output.push_str(&format!(
        "│ {:<13} │ {:<36} │\n",
        "default_prompt".bright_blue(),
        display_prompt
    ));

    output.push_str(&format!(
        "{}",
        "└───────────────┴──────────────────────────────────────┘".dimmed()
    ));

    println!("{}", output);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_header() {
        let output = print_header();
        assert!(output.contains("AI Commit Message Generator"));
    }

    #[test]
    fn test_print_config_table() {
        let config = Config {
            api_token: Some("test_token123".to_string()),
            api_base_url: None,
            model: None,
            default_prompt: Some("Write a commit message".to_string()),
        };

        let output = print_config_table(&config);

        // Test that output contains key elements
        assert!(output.contains("api_token"));
        assert!(output.contains("test•••••"));
        assert!(output.contains("api_base_url"));
        assert!(output.contains("model"));
        assert!(output.contains("Write a commit message"));
    }
}
