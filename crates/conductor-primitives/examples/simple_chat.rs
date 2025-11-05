//! Simple chat example using conductor-primitives
//!
//! This example demonstrates:
//! - Creating an LLM client (Ollama)
//! - Sending chat completion requests
//! - Using immutable state management
//!
//! Prerequisites:
//! - Ollama running locally: `ollama serve`
//! - Model downloaded: `ollama pull llama3.2:1b`
//!
//! Run with:
//! ```bash
//! cargo run --example simple_chat --package conductor-primitives
//! ```

use conductor_primitives::{AppendReducer, LlmClient, LlmConfig, Message, State};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🤖 Agent Conductor - Simple Chat Example\n");

    // 1. Configure LLM client for Ollama
    let config = LlmConfig::ollama("llama3.2:1b");
    let client = LlmClient::new(config)?;
    println!("✅ LLM Client configured: {}", client.config().base_url);

    // 2. Create initial state with system message
    let initial_messages = vec![Message::system(
        "You are a helpful AI assistant. Keep responses concise.",
    )];

    let mut state = State::new().set("messages", json!(initial_messages));
    println!("✅ Initial state created\n");

    // 3. User sends a message
    let user_message = Message::user("What is Rust and why is it popular?");
    println!("👤 User: {}\n", user_message.content);

    // 4. Update state with user message (using AppendReducer)
    let reducer = AppendReducer;
    state = state.update("messages", json!([user_message.clone()]), &reducer);

    // 5. Get current messages and send to LLM
    let messages = state.get_messages();
    println!("📤 Sending {} messages to LLM...\n", messages.len());

    let response = client.chat_completion(messages).await?;

    // 6. Update state with assistant response
    let assistant_message = Message::assistant(&response);
    state = state.update("messages", json!([assistant_message]), &reducer);

    println!("🤖 Assistant: {}\n", response);

    // 7. Demonstrate state immutability
    println!("📊 State contains {} messages", state.get_messages().len());
    println!("✅ State is immutable - all updates created new states");

    // 8. Show state inspection
    println!("\n🔍 State keys: {:?}", state.keys());
    println!("📦 State size: {} bytes", serde_json::to_string(&state)?.len());

    Ok(())
}
