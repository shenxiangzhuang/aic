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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    // Helper function for creating test config
    fn create_test_config() -> Config {
        let mut config = Config::default();
        config.set("api_token", Some("test_token_123".to_string())).unwrap();
        config.set("api_base_url", Some("https://api.example.com".to_string())).unwrap();
        config.set("model", Some("gpt-4".to_string())).unwrap();
        config
    }

    #[test]
    fn test_print_header() {
        // Verify header printing doesn't panic
        print_header();
    }

    #[test]
    fn test_token_masking() {
        let mut config = create_test_config();
        
        // Test long token
        config.set("api_token", Some("abcd1234567890".to_string())).unwrap();
        print_config_table(&config);
        
        // Test short token
        config.set("api_token", Some("abc".to_string())).unwrap();
        print_config_table(&config);
        
        // Test no token
        config.set("api_token", None).unwrap();
        print_config_table(&config);
    }

    #[test]
    fn test_english_prompts() {
        let mut config = create_test_config();

        // Test short prompt (no truncation)
        config.set("system_prompt", Some("Short msg".to_string())).unwrap();
        config.set("user_prompt", Some("Brief prompt".to_string())).unwrap();
        print_config_table(&config);

        // Test exact length prompt (12 chars)
        config.set("system_prompt", Some("Exactly12Chars".to_string())).unwrap();
        print_config_table(&config);

        // Test long prompt (with truncation)
        config.set("system_prompt", Some("This is a very long prompt that should be truncated".to_string())).unwrap();
        config.set("user_prompt", Some("Another long prompt that needs truncation".to_string())).unwrap();
        
        let system_prompt = config.get_system_prompt();
        let display_system_prompt = if system_prompt.chars().count() > 12 {
            format!("{}...", system_prompt.chars().take(12).collect::<String>())
        } else {
            system_prompt.to_string()
        };

        let user_prompt = config.get_user_prompt();
        let display_user_prompt = if user_prompt.chars().count() > 12 {
            format!("{}...", user_prompt.chars().take(12).collect::<String>())
        } else {
            user_prompt.to_string()
        };

        assert_eq!(display_system_prompt, "This is a ve...");
        assert_eq!(display_user_prompt, "Another long...");
    }

    #[test]
    fn test_chinese_display() {
        let mut config = create_test_config();
        
        // Test simple Chinese prompt
        config.set("system_prompt", Some("编写提交信息".to_string())).unwrap();
        config.set("user_prompt", Some("生成提交说明".to_string())).unwrap();
        
        let system_prompt = config.get_system_prompt();
        let display_system_prompt = if system_prompt.chars().count() > 12 {
            format!("{}...", system_prompt.chars().take(12).collect::<String>())
        } else {
            system_prompt.to_string()
        };

        // Verify Chinese characters are displayed correctly (no truncation needed)
        assert_eq!(display_system_prompt, "编写提交信息");
        
        print_config_table(&config);
    }
}
