//! LLM client for OpenAI-compatible endpoints
//!
//! Supports:
//! - OpenAI API
//! - Ollama (http://localhost:11434/v1)
//! - Any OpenAI-compatible service

use crate::message::Message;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Configuration for LLM client
#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// Base URL for the API (e.g., "http://localhost:11434/v1" for Ollama)
    pub base_url: String,

    /// API key (optional for local services like Ollama)
    pub api_key: Option<String>,

    /// Model name (e.g., "gpt-4", "llama3.2:1b")
    pub model: String,

    /// Temperature for sampling (0.0 - 2.0)
    pub temperature: f32,

    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434/v1".to_string(),
            api_key: None,
            model: "llama3.2:1b".to_string(),
            temperature: 0.7,
            max_tokens: None,
        }
    }
}

impl LlmConfig {
    /// Create config for Ollama
    pub fn ollama(model: impl Into<String>) -> Self {
        Self {
            base_url: "http://localhost:11434/v1".to_string(),
            api_key: None,
            model: model.into(),
            temperature: 0.7,
            max_tokens: None,
        }
    }

    /// Create config for OpenAI
    pub fn openai(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: Some(api_key.into()),
            model: model.into(),
            temperature: 0.7,
            max_tokens: None,
        }
    }

    /// Create config for custom endpoint
    pub fn custom(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: None,
            model: model.into(),
            temperature: 0.7,
            max_tokens: None,
        }
    }
}

/// LLM client for making chat completion requests
pub struct LlmClient {
    config: LlmConfig,
    client: reqwest::Client,
}

impl LlmClient {
    /// Create a new LLM client
    pub fn new(config: LlmConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { config, client })
    }

    /// Send a chat completion request
    pub async fn chat_completion(&self, messages: Vec<Message>) -> Result<String> {
        let request = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages,
            temperature: Some(self.config.temperature),
            max_tokens: self.config.max_tokens,
            stream: false,
        };

        let url = format!("{}/chat/completions", self.config.base_url);

        let mut req_builder = self.client.post(&url).json(&request);

        // Add authorization header if API key is present
        if let Some(api_key) = &self.config.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req_builder
            .send()
            .await
            .context("Failed to send request to LLM API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("LLM API error: {} - {}", status, body);
        }

        let completion: ChatCompletionResponse = response
            .json()
            .await
            .context("Failed to parse LLM response")?;

        completion
            .choices
            .first()
            .and_then(|choice| Some(choice.message.content.clone()))
            .ok_or_else(|| anyhow::anyhow!("No response from LLM"))
    }

    /// Get the current configuration
    pub fn config(&self) -> &LlmConfig {
        &self.config
    }
}

/// OpenAI Chat Completion Request
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    stream: bool,
}

/// OpenAI Chat Completion Response
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = LlmConfig::ollama("llama3.2:1b");
        assert_eq!(config.model, "llama3.2:1b");
        assert_eq!(config.base_url, "http://localhost:11434/v1");
        assert!(config.api_key.is_none());
    }

    #[test]
    fn test_openai_config() {
        let config = LlmConfig::openai("sk-test", "gpt-4");
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.base_url, "https://api.openai.com/v1");
        assert_eq!(config.api_key, Some("sk-test".to_string()));
    }

    #[test]
    fn test_client_creation() {
        let config = LlmConfig::default();
        let client = LlmClient::new(config);
        assert!(client.is_ok());
    }

    // Integration test - requires Ollama running locally
    #[tokio::test]
    #[ignore] // Run with: cargo test -- --ignored
    async fn test_ollama_chat_completion() {
        let config = LlmConfig::ollama("llama3.2:1b");
        let client = LlmClient::new(config).unwrap();

        let messages = vec![
            Message::system("You are a helpful assistant."),
            Message::user("Say 'Hello World' and nothing else."),
        ];

        let response = client.chat_completion(messages).await;
        assert!(response.is_ok());

        let text = response.unwrap();
        assert!(!text.is_empty());
        println!("LLM Response: {}", text);
    }
}
