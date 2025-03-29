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
    api_base_url: &str,
    model: &str,
) -> Result<String> {
    let client = Client::new();

    // Prepare the request to OpenAI API
    let request = OpenAIRequest {
        model: model.to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: format!(
                    "{}\n\nYou are an expert at writing clear and concise commit messages. \
                    Follow these rules strictly:\n\n\
                    1. Start with a type: feat, fix, docs, style, refactor, perf, test, build, ci, chore, or revert\n\
                    2. Optionally add a scope in parentheses after the type\n\
                    3. Write a brief description in imperative mood (e.g., 'add' not 'added')\n\
                    4. Keep the first line under 72 characters\n\
                    5. Use the body to explain what and why, not how\n\
                    6. Reference issues and pull requests liberally\n\
                    7. Consider starting the body with 'This commit' to make it clear what the commit does\n\n\
                    Example format:\n\
                    type(scope): subject\n\n\
                    body\n\n\
                    footer",
                    system_prompt
                ),
            },
            Message {
                role: "user".to_string(),
                content: format!(
                    "Here is the git diff of the staged changes. Generate a commit message that \
                    follows the conventional commit format and best practices. Focus on what changed \
                    and why, not how it changed:\n\n\
                    ```diff\n{}\n```",
                    diff
                ),
            },
        ],
    };

    // Construct the full API endpoint URL
    let endpoint = format!("{}/v1/chat/completions", api_base_url.trim_end_matches('/'));

    // Send the request to the API
    let response = client
        .post(&endpoint)
        .header("Authorization", format!("Bearer {}", api_token))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .context(format!("Failed to send request to API at {}", endpoint))?;

    // Parse the response
    let response_status = response.status();
    let response_text = response.text().await?;

    if !response_status.is_success() {
        return Err(anyhow::anyhow!(
            "API request failed ({}): {}",
            response_status,
            response_text
        ));
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

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        matchers::{header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[tokio::test]
    async fn test_generate_commit_message() -> Result<()> {
        // Start a mock server
        let mock_server = MockServer::start().await;

        // Create a mock response
        let mock_response = serde_json::json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "feat: improve greeting message with username support"
                }
            }]
        });

        // Set up the mock expectation
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(header("Authorization", "Bearer test_token"))
            .and(header("Content-Type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
            .mount(&mock_server)
            .await;

        // Sample git diff
        let diff = r#"
            diff --git a/src/main.rs b/src/main.rs
            index 1234567..89abcdef 100644
            --- a/src/main.rs
            +++ b/src/main.rs
            @@ -1,3 +1,4 @@
            +use std::env;
            fn main() {
            -    println!("Hello, world!");
            +    println!("Hello, {}!", env::var("USER").unwrap_or("world".to_string()));
            }
            "#;

        let system_prompt = "You are a helpful assistant.";
        let model = "gpt-3.5-turbo";

        // Use the mock server URL instead of the real OpenAI API
        let commit_message =
            generate_commit_message(diff, system_prompt, "test_token", &mock_server.uri(), model)
                .await?;

        // Verify the response
        assert_eq!(
            commit_message,
            "feat: improve greeting message with username support"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_generate_commit_message_api_error() -> Result<()> {
        // Start a mock server
        let mock_server = MockServer::start().await;

        // Set up the mock to return an error
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&mock_server)
            .await;

        // Attempt to generate a commit message
        let result = generate_commit_message(
            "some diff",
            "system prompt",
            "invalid_token",
            &mock_server.uri(),
            "gpt-3.5-turbo",
        )
        .await;

        // Verify that we get an error
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("API request failed"));
        assert!(err.contains("401"));
        assert!(err.contains("Unauthorized"));

        Ok(())
    }
}
