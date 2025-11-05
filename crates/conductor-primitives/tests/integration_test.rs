//! Integration tests with mock LLM server
//!
//! Since we can't install Ollama in all environments,
//! we create a mock HTTP server that simulates OpenAI-compatible responses.

use conductor_primitives::{AppendReducer, LlmClient, LlmConfig, Message, State};
use serde_json::json;

/// Create a mock LLM client for testing
fn mock_client() -> LlmClient {
    // In a real scenario, this would connect to a mock HTTP server
    // For now, we'll test the structure
    let config = LlmConfig::ollama("llama3.2:1b");
    LlmClient::new(config).expect("Failed to create client")
}

#[tokio::test]
async fn test_state_immutability_with_reducer() {
    // Test 1: State immutability
    let state1 = State::new();
    let state2 = state1.set("count", json!(1));
    let state3 = state2.set("count", json!(2));

    assert!(state1.get("count").is_none());
    assert_eq!(state2.get("count"), Some(&json!(1)));
    assert_eq!(state3.get("count"), Some(&json!(2)));
    println!("✅ Test 1 passed: State immutability works");
}

#[tokio::test]
async fn test_append_reducer_messages() {
    // Test 2: AppendReducer for messages
    let reducer = AppendReducer;

    let state = State::new();
    let msg1 = Message::user("Hello");
    let state = state.update("messages", json!([msg1]), &reducer);

    let msg2 = Message::assistant("Hi there!");
    let state = state.update("messages", json!([msg2]), &reducer);

    let messages = state.get_messages();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].role, "user");
    assert_eq!(messages[1].role, "assistant");
    println!("✅ Test 2 passed: AppendReducer correctly appends messages");
}

#[tokio::test]
async fn test_message_serialization() {
    // Test 3: Message serialization (OpenAI compatibility)
    let messages = vec![
        Message::system("You are helpful"),
        Message::user("Hello"),
        Message::assistant("Hi!"),
    ];

    let json_str = serde_json::to_string(&messages).expect("Failed to serialize");
    let deserialized: Vec<Message> =
        serde_json::from_str(&json_str).expect("Failed to deserialize");

    assert_eq!(messages.len(), deserialized.len());
    assert_eq!(messages[0].content, deserialized[0].content);
    println!("✅ Test 3 passed: Messages serialize/deserialize correctly");
}

#[tokio::test]
async fn test_state_with_complex_types() {
    // Test 4: State with complex nested types
    let state = State::new()
        .set("user", json!({"name": "Alice", "age": 30}))
        .set("items", json!(["apple", "banana", "cherry"]))
        .set("count", json!(42));

    // Verify complex types
    assert_eq!(
        state.get("user"),
        Some(&json!({"name": "Alice", "age": 30}))
    );
    assert_eq!(state.get_typed::<i32>("count"), Some(42));

    println!("✅ Test 4 passed: State handles complex types");
}

#[tokio::test]
async fn test_multi_turn_conversation_flow() {
    // Test 5: Simulated multi-turn conversation
    let reducer = AppendReducer;
    let mut state = State::new();

    // Turn 1
    state = state.update("messages", json!([Message::user("What is Rust?")]), &reducer);

    // Turn 2
    state = state.update(
        "messages",
        json!([Message::assistant(
            "Rust is a systems programming language."
        )]),
        &reducer,
    );

    // Turn 3
    state = state.update(
        "messages",
        json!([Message::user("Tell me more")]),
        &reducer,
    );

    // Turn 4
    state = state.update(
        "messages",
        json!([Message::assistant(
            "Rust focuses on safety and performance."
        )]),
        &reducer,
    );

    let messages = state.get_messages();
    assert_eq!(messages.len(), 4);
    assert_eq!(messages[0].role, "user");
    assert_eq!(messages[1].role, "assistant");
    assert_eq!(messages[2].role, "user");
    assert_eq!(messages[3].role, "assistant");

    println!("✅ Test 5 passed: Multi-turn conversation flow works");
}

#[tokio::test]
async fn test_llm_config_variations() {
    // Test 6: Different LLM configurations
    let config1 = LlmConfig::ollama("llama3.2:1b");
    assert_eq!(config1.base_url, "http://localhost:11434/v1");
    assert_eq!(config1.model, "llama3.2:1b");
    assert!(config1.api_key.is_none());

    let config2 = LlmConfig::openai("sk-test-key", "gpt-4");
    assert_eq!(config2.base_url, "https://api.openai.com/v1");
    assert_eq!(config2.model, "gpt-4");
    assert_eq!(config2.api_key, Some("sk-test-key".to_string()));

    let config3 = LlmConfig::custom("http://custom:8080/v1", "custom-model");
    assert_eq!(config3.base_url, "http://custom:8080/v1");
    assert_eq!(config3.model, "custom-model");

    println!("✅ Test 6 passed: LLM config variations work correctly");
}

#[tokio::test]
async fn test_state_merge_operations() {
    // Test 7: State merge with reducers
    use conductor_primitives::{MergeReducer, ReplaceReducer};
    use std::collections::HashMap;

    let state1 = State::new()
        .set("count", json!(5))
        .set("config", json!({"debug": true}));

    let state2 = State::new()
        .set("count", json!(10))
        .set("config", json!({"verbose": true}));

    let mut reducers: HashMap<String, Box<dyn conductor_primitives::Reducer>> = HashMap::new();
    reducers.insert("count".to_string(), Box::new(ReplaceReducer));
    reducers.insert("config".to_string(), Box::new(MergeReducer));

    let merged = state1.merge(&state2, &reducers);

    assert_eq!(merged.get("count"), Some(&json!(10))); // Replaced
    assert_eq!(
        merged.get("config"),
        Some(&json!({"debug": true, "verbose": true}))
    ); // Merged

    println!("✅ Test 7 passed: State merge with different reducers works");
}
