use anyhow::{Context, Result};
use std::env;

/// Get the API token from environment variables
pub fn get_api_token() -> Result<String> {
    env::var("AIC_API_TOKEN")
        .context("API token not found. Please set the AIC_API_TOKEN environment variable.")
}

/// Get the API base URL from environment variable or use default
pub fn get_api_base_url() -> String {
    env::var("AIC_API_BASE_URL").unwrap_or_else(|_| "https://api.openai.com".to_string())
}

/// Get the model name from environment variable or use default
pub fn get_model() -> String {
    env::var("AIC_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string())
}

/// Get the default system prompt for commit message generation
pub fn get_default_prompt() -> String {
    "You are a helpful assistant specialized in writing git commit messages. \
    Follow these guidelines: \
    1. Use the imperative mood (e.g., 'Add' not 'Added'). \
    2. Keep it concise but descriptive. \
    3. Include the scope of the change when relevant. \
    4. Explain WHY the change was made, not just WHAT was changed. \
    5. Use conventional commit format when appropriate."
        .to_string()
}
