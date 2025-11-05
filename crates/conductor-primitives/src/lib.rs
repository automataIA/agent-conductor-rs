//! # Conductor Primitives
//!
//! Core primitives for the Agent Conductor framework.
//!
//! This crate provides the foundational building blocks:
//! - **Message**: Chat message types (OpenAI-compatible)
//! - **State**: Immutable state management with reducers
//! - **LlmClient**: HTTP client for OpenAI-compatible LLM APIs
//!
//! ## Example
//!
//! ```rust
//! use conductor_primitives::{Message, State, LlmClient, LlmConfig};
//! use serde_json::json;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create a message
//! let msg = Message::user("Hello, world!");
//!
//! // Create immutable state
//! let state = State::new()
//!     .set("count", json!(42))
//!     .set("messages", json!([msg]));
//!
//! // Create LLM client (Ollama)
//! let config = LlmConfig::ollama("llama3.2:1b");
//! let client = LlmClient::new(config)?;
//!
//! // Make a completion request
//! let messages = vec![
//!     Message::system("You are helpful."),
//!     Message::user("Hi!"),
//! ];
//! let response = client.chat_completion(messages).await?;
//! # Ok(())
//! # }
//! ```

pub mod llm_client;
pub mod message;
pub mod state;

// Re-export main types
pub use llm_client::{LlmClient, LlmConfig};
pub use message::Message;
pub use state::{AppendReducer, MergeReducer, Reducer, ReplaceReducer, State};
