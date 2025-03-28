use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

/// Generate a commit message based on the git diff
pub async fn generate_commit_message(
    diff: &str,
    system_prompt: &str,
    api_token: &str,
) -> Result<String> {
    let client = Client::new();

    // Prepare the request to OpenAI API
    let request = OpenAIRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: format!("{} Generate a concise and informative commit message for the following changes.", system_prompt),
            },
            Message {
                role: "user".to_string(),
                content: format!("Here is the git diff of the staged changes. Please generate a commit message based on these changes:\n\n```diff\n{}\n```", diff),
            },
        ],
    };

    // Send the request to OpenAI API
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_token))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .context("Failed to send request to OpenAI API")?;

    // Parse the response
    let response_status = response.status();
    let response_text = response.text().await?;

    if !response_status.is_success() {
        return Err(anyhow::anyhow!("API request failed: {}", response_text));
    }

    let response: OpenAIResponse =
        serde_json::from_str(&response_text).context("Failed to parse API response")?;

    // Extract the commit message
    let commit_message = response
        .choices
        .first()
        .context("No response from API")?
        .message
        .content
        .clone();

    Ok(commit_message)
}
