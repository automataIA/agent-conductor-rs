//! # Conductor Builder
//!
//! Fluent API and workflow management for Agent Conductor.
//!
//! This crate provides:
//! - **GraphBuilder**: Fluent API for constructing graphs
//! - **WorkflowManager**: Template and session management
//! - **WorkflowTemplate**: Reusable workflow configurations
//!
//! ## Example
//!
//! ```rust
//! use conductor_builder::GraphBuilder;
//! use conductor_graph::{FunctionNode, BoolCondition};
//! use conductor_primitives::State;
//! use serde_json::json;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create nodes
//! let validator = FunctionNode::new("validator", |state| {
//!     Box::pin(async move {
//!         let is_valid = true; // validation logic
//!         Ok(state.set("is_valid", json!(is_valid)))
//!     })
//! });
//!
//! let processor = FunctionNode::new("processor", |state| {
//!     Box::pin(async move {
//!         Ok(state.set("processed", json!(true)))
//!     })
//! });
//!
//! // Build graph with fluent API
//! let mut graph = GraphBuilder::new()
//!     .add_node(Box::new(validator))
//!     .add_node(Box::new(processor))
//!     .add_edge("start", "validator")
//!     .add_conditional_edge(
//!         "validator",
//!         BoolCondition::new("is_valid", "processor", "end")
//!     )
//!     .add_edge("processor", "end")
//!     .max_steps(50)
//!     .build(":memory:")?;
//!
//! // Execute
//! let state = State::new();
//! let result = graph.execute("workflow-1", state).await?;
//! # Ok(())
//! # }
//! ```

pub mod builder;
pub mod workflow;

// Re-export main types
pub use builder::GraphBuilder;
pub use workflow::{WorkflowManager, WorkflowSession, WorkflowTemplate};
