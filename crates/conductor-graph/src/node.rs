//! Node trait and implementations
//!
//! Nodes are the core building blocks of agent graphs. Each node:
//! - Receives an immutable state
//! - Performs computation (potentially async, like calling an LLM)
//! - Returns a new state with updates
//!
//! This follows LangGraph's node concept but with Rust's type safety.

use anyhow::Result;
use async_trait::async_trait;
use conductor_primitives::{AppendReducer, LlmClient, Message, Reducer, State};
use serde_json::json;

/// Node trait - the core abstraction for graph components
///
/// Nodes are pure functions: State -> Result<State>
/// They should not mutate state, but return a new state with changes.
#[async_trait]
pub trait Node: Send + Sync {
    /// Execute the node's logic with the given state
    ///
    /// # Arguments
    /// * `state` - Current immutable state
    ///
    /// # Returns
    /// * `Result<State>` - New state with updates, or error
    async fn invoke(&self, state: State) -> Result<State>;

    /// Get the node's name (used for graph visualization and routing)
    fn name(&self) -> &str;

    /// Optional: Get the reducer to use for this node's outputs
    ///
    /// Default is ReplaceReducer. Override to use AppendReducer for messages.
    fn reducer(&self) -> Box<dyn Reducer> {
        Box::new(conductor_primitives::ReplaceReducer)
    }
}

/// LLM Node - invokes an LLM with messages from state
///
/// This is one of the most common node types. It:
/// 1. Extracts messages from state
/// 2. Optionally adds a system prompt
/// 3. Calls the LLM
/// 4. Appends the response to messages
///
/// # Example
///
/// ```rust
/// use conductor_graph::LlmNode;
/// use conductor_primitives::{LlmClient, LlmConfig, State, Message};
/// use serde_json::json;
///
/// # async fn example() -> anyhow::Result<()> {
/// let config = LlmConfig::ollama("llama3.2:1b");
/// let client = LlmClient::new(config)?;
///
/// let node = LlmNode::new("assistant", client)
///     .with_system_prompt("You are a helpful assistant.");
///
/// let state = State::new().set("messages", json!([
///     Message::user("Hello!")
/// ]));
///
/// let new_state = node.invoke(state).await?;
/// # Ok(())
/// # }
/// ```
pub struct LlmNode {
    name: String,
    client: LlmClient,
    system_prompt: Option<String>,
}

impl LlmNode {
    /// Create a new LLM node
    pub fn new(name: impl Into<String>, client: LlmClient) -> Self {
        Self {
            name: name.into(),
            client,
            system_prompt: None,
        }
    }

    /// Add a system prompt that will be prepended to messages
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Set the system prompt
    pub fn set_system_prompt(&mut self, prompt: impl Into<String>) {
        self.system_prompt = Some(prompt.into());
    }
}

#[async_trait]
impl Node for LlmNode {
    async fn invoke(&self, state: State) -> Result<State> {
        // 1. Get messages from state
        let mut messages = state.get_messages();

        // 2. Prepend system prompt if configured
        if let Some(sys_prompt) = &self.system_prompt {
            // Only add if no system message exists
            if !messages.iter().any(|m| m.role == "system") {
                messages.insert(0, Message::system(sys_prompt));
            }
        }

        // 3. Call LLM
        let response = self.client.chat_completion(messages).await?;

        // 4. Create assistant message
        let assistant_msg = Message::assistant(response);

        // 5. Update state with new message (using AppendReducer)
        let reducer = AppendReducer;
        let new_state = state.update("messages", json!([assistant_msg]), &reducer);

        Ok(new_state)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn reducer(&self) -> Box<dyn Reducer> {
        Box::new(AppendReducer)
    }
}

/// Function Node - wraps a closure as a node
///
/// Useful for simple transformations or custom logic.
///
/// # Example
///
/// ```rust
/// use conductor_graph::FunctionNode;
/// use conductor_primitives::State;
/// use serde_json::json;
///
/// # async fn example() -> anyhow::Result<()> {
/// let counter = FunctionNode::new("counter", |state| {
///     Box::pin(async move {
///         let count: i32 = state.get_typed("count").unwrap_or(0);
///         Ok(state.set("count", json!(count + 1)))
///     })
/// });
///
/// let state = State::new();
/// let new_state = counter.invoke(state).await?;
/// assert_eq!(new_state.get_typed::<i32>("count"), Some(1));
/// # Ok(())
/// # }
/// ```
pub struct FunctionNode<F>
where
    F: Fn(State) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<State>> + Send>>
        + Send
        + Sync,
{
    name: String,
    func: F,
}

impl<F> FunctionNode<F>
where
    F: Fn(State) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<State>> + Send>>
        + Send
        + Sync,
{
    /// Create a new function node
    pub fn new(name: impl Into<String>, func: F) -> Self {
        Self {
            name: name.into(),
            func,
        }
    }
}

#[async_trait]
impl<F> Node for FunctionNode<F>
where
    F: Fn(State) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<State>> + Send>>
        + Send
        + Sync,
{
    async fn invoke(&self, state: State) -> Result<State> {
        (self.func)(state).await
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conductor_primitives::LlmConfig;

    #[test]
    fn test_llm_node_creation() {
        let config = LlmConfig::ollama("llama3.2:1b");
        let client = LlmClient::new(config).unwrap();
        let node = LlmNode::new("test", client);

        assert_eq!(node.name(), "test");
    }

    #[test]
    fn test_llm_node_with_system_prompt() {
        let config = LlmConfig::ollama("llama3.2:1b");
        let client = LlmClient::new(config).unwrap();
        let node = LlmNode::new("test", client).with_system_prompt("Test prompt");

        assert!(node.system_prompt.is_some());
        assert_eq!(node.system_prompt.unwrap(), "Test prompt");
    }

    #[tokio::test]
    async fn test_function_node() {
        let node = FunctionNode::new("adder", |state| {
            Box::pin(async move {
                let count: i32 = state.get_typed("count").unwrap_or(0);
                Ok(state.set("count", json!(count + 1)))
            })
        });

        let state = State::new();
        let state = node.invoke(state).await.unwrap();
        assert_eq!(state.get_typed::<i32>("count"), Some(1));

        let state = node.invoke(state).await.unwrap();
        assert_eq!(state.get_typed::<i32>("count"), Some(2));
    }

    #[tokio::test]
    #[ignore] // Requires Ollama running
    async fn test_llm_node_invoke() {
        let config = LlmConfig::ollama("llama3.2:1b");
        let client = LlmClient::new(config).unwrap();
        let node = LlmNode::new("assistant", client)
            .with_system_prompt("You are helpful. Be concise.");

        let state = State::new().set(
            "messages",
            json!([Message::user("Say 'hello' and nothing else")]),
        );

        let new_state = node.invoke(state).await.unwrap();
        let messages = new_state.get_messages();

        assert_eq!(messages.len(), 3); // system + user + assistant
        assert_eq!(messages[2].role, "assistant");
        assert!(!messages[2].content.is_empty());

        println!("LLM Response: {}", messages[2].content);
    }
}
