//! GraphBuilder - Fluent API for graph construction
//!
//! Provides an ergonomic, type-safe way to build complex agent workflows.
//! Inspired by builder patterns in Rust and LangGraph's graph construction API.

use anyhow::{Context, Result};
use conductor_executor::{ExecutorConfig, GraphExecutor};
use conductor_graph::{Condition, Edge, Node};
use std::collections::HashMap;

/// Graph builder with fluent API
///
/// # Example
///
/// ```rust
/// use conductor_builder::GraphBuilder;
/// use conductor_graph::{FunctionNode, BoolCondition};
/// use conductor_primitives::State;
/// use serde_json::json;
///
/// # async fn example() -> anyhow::Result<()> {
/// let node_a = FunctionNode::new("process", |state| {
///     Box::pin(async move { Ok(state.set("done", json!(true))) })
/// });
///
/// let graph = GraphBuilder::new()
///     .add_node(Box::new(node_a))
///     .add_edge("start", "process")
///     .add_edge("process", "end")
///     .build(":memory:")?;
///
/// let state = State::new();
/// let result = graph.execute("wf-1", state).await?;
/// # Ok(())
/// # }
/// ```
pub struct GraphBuilder {
    nodes: Vec<Box<dyn Node>>,
    edges: Vec<Edge>,
    config: ExecutorConfig,
}

impl GraphBuilder {
    /// Create a new graph builder
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            config: ExecutorConfig::default(),
        }
    }

    /// Add a node to the graph
    ///
    /// # Example
    ///
    /// ```rust
    /// use conductor_builder::GraphBuilder;
    /// use conductor_graph::FunctionNode;
    /// use serde_json::json;
    ///
    /// let node = FunctionNode::new("test", |state| {
    ///     Box::pin(async move { Ok(state.set("key", json!("value"))) })
    /// });
    ///
    /// let builder = GraphBuilder::new()
    ///     .add_node(Box::new(node));
    /// ```
    pub fn add_node(mut self, node: Box<dyn Node>) -> Self {
        self.nodes.push(node);
        self
    }

    /// Add a required edge between two nodes
    ///
    /// # Example
    ///
    /// ```rust
    /// use conductor_builder::GraphBuilder;
    ///
    /// let builder = GraphBuilder::new()
    ///     .add_edge("start", "process")
    ///     .add_edge("process", "end");
    /// ```
    pub fn add_edge(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.edges.push(Edge::required(from, to));
        self
    }

    /// Add a conditional edge with a condition
    ///
    /// # Example
    ///
    /// ```rust
    /// use conductor_builder::GraphBuilder;
    /// use conductor_graph::BoolCondition;
    ///
    /// let condition = BoolCondition::new("is_valid", "success", "error");
    ///
    /// let builder = GraphBuilder::new()
    ///     .add_conditional_edge("validator", condition);
    /// ```
    pub fn add_conditional_edge(
        mut self,
        from: impl Into<String>,
        condition: impl Condition + 'static,
    ) -> Self {
        self.edges.push(Edge::conditional(from, condition));
        self
    }

    /// Set the maximum number of execution steps
    ///
    /// Default: 100
    pub fn max_steps(mut self, max_steps: usize) -> Self {
        self.config.max_steps = max_steps;
        self
    }

    /// Set the start node name
    ///
    /// Default: "start"
    pub fn start_node(mut self, node: impl Into<String>) -> Self {
        self.config.start_node = node.into();
        self
    }

    /// Add a terminal node (execution stops when reached)
    ///
    /// Default: ["end"]
    pub fn terminal_node(mut self, node: impl Into<String>) -> Self {
        self.config.terminal_nodes.push(node.into());
        self
    }

    /// Disable checkpointing
    ///
    /// By default, checkpointing is enabled
    pub fn disable_checkpoints(mut self) -> Self {
        self.config.enable_checkpoints = false;
        self
    }

    /// Enable checkpointing (enabled by default)
    pub fn enable_checkpoints(mut self) -> Self {
        self.config.enable_checkpoints = true;
        self
    }

    /// Validate the graph before building
    fn validate(&self) -> Result<()> {
        // Check that all nodes have unique names
        let mut node_names = std::collections::HashSet::new();
        for node in &self.nodes {
            let name = node.name();
            if !node_names.insert(name) {
                anyhow::bail!("Duplicate node name: '{}'", name);
            }
        }

        // Check that all edges reference existing nodes
        let node_name_vec: Vec<&str> = node_names.iter().map(|s| s.as_str()).collect();

        for edge in &self.edges {
            let from = edge.from();
            if from != self.config.start_node && !node_names.contains(from) {
                anyhow::bail!("Edge references non-existent node: '{}'", from);
            }

            if edge.is_required() {
                if let Some(to) = edge.to() {
                    if !node_names.contains(to) && !self.config.terminal_nodes.contains(&to.to_string()) {
                        anyhow::bail!("Edge references non-existent node: '{}'", to);
                    }
                }
            }
        }

        // Check that start node has outgoing edges
        let has_start_edge = self.edges.iter().any(|e| e.from() == self.config.start_node);
        if !has_start_edge {
            anyhow::bail!("Start node '{}' has no outgoing edges", self.config.start_node);
        }

        Ok(())
    }

    /// Build the graph executor
    ///
    /// # Arguments
    /// * `checkpoint_db` - Path to checkpoint database, or ":memory:" for in-memory
    ///
    /// # Returns
    /// A configured GraphExecutor ready to execute workflows
    pub fn build(self, checkpoint_db: &str) -> Result<GraphExecutor> {
        // Validate graph structure
        self.validate().context("Graph validation failed")?;

        // Create executor
        let mut executor = GraphExecutor::new(self.config, checkpoint_db);

        // Add all nodes
        for node in self.nodes {
            executor.add_node(node);
        }

        // Set edges
        executor.set_edges(self.edges);

        Ok(executor)
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conductor_graph::{BoolCondition, FunctionNode};
    use conductor_primitives::State;
    use serde_json::json;

    #[test]
    fn test_builder_creation() {
        let builder = GraphBuilder::new();
        assert_eq!(builder.nodes.len(), 0);
        assert_eq!(builder.edges.len(), 0);
    }

    #[test]
    fn test_builder_add_node() {
        let node = FunctionNode::new("test", |state| Box::pin(async move { Ok(state) }));

        let builder = GraphBuilder::new().add_node(Box::new(node));

        assert_eq!(builder.nodes.len(), 1);
    }

    #[test]
    fn test_builder_add_edge() {
        let builder = GraphBuilder::new()
            .add_edge("a", "b")
            .add_edge("b", "c");

        assert_eq!(builder.edges.len(), 2);
    }

    #[test]
    fn test_builder_add_conditional_edge() {
        let condition = BoolCondition::new("flag", "yes", "no");

        let builder = GraphBuilder::new().add_conditional_edge("check", condition);

        assert_eq!(builder.edges.len(), 1);
        assert!(builder.edges[0].is_conditional());
    }

    #[test]
    fn test_builder_config() {
        let builder = GraphBuilder::new()
            .max_steps(50)
            .start_node("custom_start")
            .terminal_node("custom_end")
            .disable_checkpoints();

        assert_eq!(builder.config.max_steps, 50);
        assert_eq!(builder.config.start_node, "custom_start");
        assert!(builder.config.terminal_nodes.contains(&"custom_end".to_string()));
        assert!(!builder.config.enable_checkpoints);
    }

    #[tokio::test]
    async fn test_builder_build_simple() {
        let node = FunctionNode::new("process", |state| {
            Box::pin(async move { Ok(state.set("done", json!(true))) })
        });

        let mut graph = GraphBuilder::new()
            .add_node(Box::new(node))
            .add_edge("start", "process")
            .add_edge("process", "end")
            .build(":memory:")
            .unwrap();

        let state = State::new();
        let result = graph.execute("test", state).await.unwrap();

        assert_eq!(result.get("done"), Some(&json!(true)));
    }

    #[test]
    fn test_builder_validate_duplicate_nodes() {
        let node1 = FunctionNode::new("dup", |state| Box::pin(async move { Ok(state) }));
        let node2 = FunctionNode::new("dup", |state| Box::pin(async move { Ok(state) }));

        let result = GraphBuilder::new()
            .add_node(Box::new(node1))
            .add_node(Box::new(node2))
            .add_edge("start", "dup")
            .add_edge("dup", "end")
            .build(":memory:");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate node"));
    }

    #[test]
    fn test_builder_validate_missing_node() {
        let node = FunctionNode::new("exists", |state| Box::pin(async move { Ok(state) }));

        let result = GraphBuilder::new()
            .add_node(Box::new(node))
            .add_edge("start", "nonexistent") // References node that doesn't exist
            .build(":memory:");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("non-existent"));
    }

    #[test]
    fn test_builder_validate_no_start_edge() {
        let node = FunctionNode::new("orphan", |state| Box::pin(async move { Ok(state) }));

        let result = GraphBuilder::new()
            .add_node(Box::new(node))
            // No edge from start!
            .add_edge("orphan", "end")
            .build(":memory:");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no outgoing edges"));
    }

    #[tokio::test]
    async fn test_builder_complex_graph() {
        let node_a = FunctionNode::new("node_a", |state| {
            Box::pin(async move { Ok(state.set("visited_a", json!(true))) })
        });

        let node_b = FunctionNode::new("node_b", |state| {
            Box::pin(async move { Ok(state.set("visited_b", json!(true))) })
        });

        let node_c = FunctionNode::new("node_c", |state| {
            Box::pin(async move { Ok(state.set("visited_c", json!(true))) })
        });

        let mut graph = GraphBuilder::new()
            .add_node(Box::new(node_a))
            .add_node(Box::new(node_b))
            .add_node(Box::new(node_c))
            .add_edge("start", "node_a")
            .add_edge("node_a", "node_b")
            .add_edge("node_b", "node_c")
            .add_edge("node_c", "end")
            .max_steps(10)
            .build(":memory:")
            .unwrap();

        let state = State::new();
        let result = graph.execute("complex", state).await.unwrap();

        assert_eq!(result.get("visited_a"), Some(&json!(true)));
        assert_eq!(result.get("visited_b"), Some(&json!(true)));
        assert_eq!(result.get("visited_c"), Some(&json!(true)));
    }
}
