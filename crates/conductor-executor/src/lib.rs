//! # Conductor Executor
//!
//! Graph execution engine with message passing and checkpointing.
//!
//! This crate provides:
//! - **GraphExecutor**: Core execution engine with message passing
//! - **Checkpointer**: SQLite-based state persistence
//! - **Router**: Edge evaluation and next-node determination
//!
//! ## Example
//!
//! ```rust
//! use conductor_executor::{GraphExecutor, ExecutorConfig};
//! use conductor_graph::{FunctionNode, Edge};
//! use conductor_primitives::State;
//! use serde_json::json;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create nodes
//! let node = FunctionNode::new("process", |state| {
//!     Box::pin(async move {
//!         Ok(state.set("processed", json!(true)))
//!     })
//! });
//!
//! // Create edges
//! let edges = vec![
//!     Edge::required("start", "process"),
//!     Edge::required("process", "end"),
//! ];
//!
//! // Setup executor
//! let config = ExecutorConfig::default();
//! let mut executor = GraphExecutor::new(config, ":memory:");
//!
//! executor.add_node(Box::new(node));
//! executor.set_edges(edges);
//!
//! // Execute
//! let state = State::new();
//! let result = executor.execute("workflow-1", state).await?;
//!
//! assert_eq!(result.get("processed"), Some(&json!(true)));
//! # Ok(())
//! # }
//! ```

pub mod checkpointer;
pub mod executor;
pub mod router;

// Re-export main types
pub use checkpointer::{Checkpoint, Checkpointer};
pub use executor::{ExecutionEvent, ExecutorConfig, GraphExecutor};
pub use router::Router;
