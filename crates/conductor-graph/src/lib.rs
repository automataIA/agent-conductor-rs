//! # Conductor Graph
//!
//! Graph components for building agent workflows.
//!
//! This crate provides:
//! - **Node**: Trait for graph nodes (LlmNode, FunctionNode)
//! - **Edge**: Connection between nodes (Required, Conditional)
//! - **Condition**: Trait for conditional routing logic
//!
//! ## Example
//!
//! ```rust
//! use conductor_graph::{LlmNode, Edge, BoolCondition};
//! use conductor_primitives::{LlmClient, LlmConfig, State, Message};
//! use serde_json::json;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create LLM node
//! let config = LlmConfig::ollama("llama3.2:1b");
//! let client = LlmClient::new(config)?;
//! let node = LlmNode::new("assistant", client);
//!
//! // Create edges
//! let edge1 = Edge::required("start", "assistant");
//! let condition = BoolCondition::new("needs_review", "reviewer", "end");
//! let edge2 = Edge::conditional("assistant", condition);
//!
//! // Execute node
//! let state = State::new().set("messages", json!([Message::user("Hi")]));
//! let new_state = node.invoke(state).await?;
//! # Ok(())
//! # }
//! ```

pub mod edge;
pub mod node;

// Re-export main types
pub use edge::{BoolCondition, Condition, Edge, FunctionCondition};
pub use node::{FunctionNode, LlmNode, Node};
