//! Graph executor with message passing
//!
//! Implements the core execution engine following LangGraph's message passing model:
//! 1. Start node receives initial state
//! 2. Node executes and produces new state
//! 3. Save checkpoint
//! 4. Router evaluates edges to find next node
//! 5. Repeat until terminal node

use anyhow::{Context, Result};
use conductor_graph::Node;
use conductor_primitives::State;
use std::collections::HashMap;
use std::sync::Arc;

use crate::checkpointer::Checkpointer;
use crate::router::Router;

/// Execution event emitted during graph execution
#[derive(Debug, Clone)]
pub enum ExecutionEvent {
    /// Node started executing
    NodeStarted {
        node_name: String,
        step: usize,
    },

    /// Node completed successfully
    NodeCompleted {
        node_name: String,
        step: usize,
    },

    /// Checkpoint saved
    CheckpointSaved {
        workflow_id: String,
        step: usize,
    },

    /// Execution completed
    ExecutionCompleted {
        workflow_id: String,
        total_steps: usize,
    },

    /// Error occurred
    Error {
        node_name: String,
        error: String,
    },
}

/// Graph executor configuration
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Maximum steps before termination (prevents infinite loops)
    pub max_steps: usize,

    /// Whether to enable checkpointing
    pub enable_checkpoints: bool,

    /// Start node name
    pub start_node: String,

    /// Terminal node names (execution stops when reached)
    pub terminal_nodes: Vec<String>,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_steps: 100,
            enable_checkpoints: true,
            start_node: "start".to_string(),
            terminal_nodes: vec!["end".to_string()],
        }
    }
}

/// Graph executor - the core execution engine
///
/// Executes graphs using message passing:
/// - Nodes receive state, transform it, return new state
/// - Router determines next node based on edges
/// - Checkpointer persists state for resumability
///
/// # Example
///
/// ```rust
/// use conductor_executor::{GraphExecutor, ExecutorConfig};
/// use conductor_graph::{FunctionNode, Edge};
/// use conductor_primitives::State;
/// use serde_json::json;
///
/// # async fn example() -> anyhow::Result<()> {
/// let node = FunctionNode::new("process", |state| {
///     Box::pin(async move {
///         Ok(state.set("processed", json!(true)))
///     })
/// });
///
/// let edges = vec![
///     Edge::required("start", "process"),
///     Edge::required("process", "end"),
/// ];
///
/// let config = ExecutorConfig::default();
/// let mut executor = GraphExecutor::new(config, ":memory:");
///
/// executor.add_node(Box::new(node));
/// executor.set_edges(edges);
///
/// let initial_state = State::new();
/// let result = executor.execute("workflow-1", initial_state).await?;
/// # Ok(())
/// # }
/// ```
pub struct GraphExecutor {
    config: ExecutorConfig,
    nodes: HashMap<String, Box<dyn Node>>,
    router: Option<Router>,
    checkpointer: Option<Checkpointer>,
    event_handlers: Vec<Box<dyn Fn(ExecutionEvent) + Send + Sync>>,
}

impl GraphExecutor {
    /// Create a new graph executor
    ///
    /// # Arguments
    /// * `config` - Executor configuration
    /// * `checkpoint_db` - Path to checkpoint database, or ":memory:"
    pub fn new(config: ExecutorConfig, checkpoint_db: &str) -> Self {
        let checkpointer = if config.enable_checkpoints {
            Checkpointer::new(checkpoint_db).ok()
        } else {
            None
        };

        Self {
            config,
            nodes: HashMap::new(),
            router: None,
            checkpointer,
            event_handlers: Vec::new(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: Box<dyn Node>) {
        let name = node.name().to_string();
        self.nodes.insert(name, node);
    }

    /// Set edges for routing
    pub fn set_edges(&mut self, edges: Vec<conductor_graph::Edge>) {
        self.router = Some(Router::new(edges));
    }

    /// Add an event handler
    pub fn on_event<F>(&mut self, handler: F)
    where
        F: Fn(ExecutionEvent) + Send + Sync + 'static,
    {
        self.event_handlers.push(Box::new(handler));
    }

    /// Emit an event to all handlers
    fn emit_event(&self, event: ExecutionEvent) {
        for handler in &self.event_handlers {
            handler(event.clone());
        }
    }

    /// Execute the graph from start to completion
    ///
    /// # Arguments
    /// * `workflow_id` - Unique identifier for this execution
    /// * `initial_state` - Starting state
    ///
    /// # Returns
    /// Final state after execution completes
    pub async fn execute(&mut self, workflow_id: &str, initial_state: State) -> Result<State> {
        let mut current_node = self.config.start_node.clone();
        let mut state = initial_state;
        let mut step = 0;

        // Validate router is set
        let router = self
            .router
            .as_ref()
            .context("Router not set. Call set_edges() first.")?;

        while step < self.config.max_steps {
            // Check if we've reached a terminal node
            if self.config.terminal_nodes.contains(&current_node) {
                self.emit_event(ExecutionEvent::ExecutionCompleted {
                    workflow_id: workflow_id.to_string(),
                    total_steps: step,
                });
                break;
            }

            // Emit node start event
            self.emit_event(ExecutionEvent::NodeStarted {
                node_name: current_node.clone(),
                step,
            });

            // Get and execute node
            let node = self
                .nodes
                .get(&current_node)
                .with_context(|| format!("Node '{}' not found", current_node))?;

            state = match node.invoke(state).await {
                Ok(new_state) => {
                    self.emit_event(ExecutionEvent::NodeCompleted {
                        node_name: current_node.clone(),
                        step,
                    });
                    new_state
                }
                Err(e) => {
                    self.emit_event(ExecutionEvent::Error {
                        node_name: current_node.clone(),
                        error: e.to_string(),
                    });
                    return Err(e);
                }
            };

            // Save checkpoint
            if let Some(checkpointer) = &self.checkpointer {
                checkpointer.save(workflow_id, step, &current_node, &state)?;
                self.emit_event(ExecutionEvent::CheckpointSaved {
                    workflow_id: workflow_id.to_string(),
                    step,
                });
            }

            // Determine next node
            let next_node_opt = router.next_node(&current_node, &state).await?;

            match next_node_opt {
                Some(next) => current_node = next,
                None => {
                    // Terminal node reached
                    self.emit_event(ExecutionEvent::ExecutionCompleted {
                        workflow_id: workflow_id.to_string(),
                        total_steps: step + 1,
                    });
                    break;
                }
            }

            step += 1;
        }

        if step >= self.config.max_steps {
            anyhow::bail!(
                "Max steps ({}) reached. Possible infinite loop.",
                self.config.max_steps
            );
        }

        Ok(state)
    }

    /// Resume execution from the latest checkpoint
    pub async fn resume(&mut self, workflow_id: &str) -> Result<State> {
        let checkpointer = self
            .checkpointer
            .as_ref()
            .context("Checkpointing not enabled")?;

        let (step, node_name, state) = checkpointer
            .load_latest(workflow_id)
            .context("No checkpoint found for workflow")?;

        println!(
            "Resuming workflow '{}' from step {} (node: {})",
            workflow_id, step, node_name
        );

        // Continue execution from the checkpoint
        self.execute(workflow_id, state).await
    }

    /// Get the checkpointer (if enabled)
    pub fn checkpointer(&self) -> Option<&Checkpointer> {
        self.checkpointer.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conductor_graph::{Edge, FunctionNode};
    use serde_json::json;

    #[tokio::test]
    async fn test_executor_simple_chain() {
        // Create nodes
        let node_a = FunctionNode::new("node_a", |state| {
            Box::pin(async move { Ok(state.set("visited_a", json!(true))) })
        });

        let node_b = FunctionNode::new("node_b", |state| {
            Box::pin(async move { Ok(state.set("visited_b", json!(true))) })
        });

        // Create edges
        let edges = vec![
            Edge::required("start", "node_a"),
            Edge::required("node_a", "node_b"),
            Edge::required("node_b", "end"),
        ];

        // Setup executor
        let config = ExecutorConfig::default();
        let mut executor = GraphExecutor::new(config, ":memory:");

        executor.add_node(Box::new(node_a));
        executor.add_node(Box::new(node_b));
        executor.set_edges(edges);

        // Execute
        let state = State::new();
        let result = executor.execute("test-1", state).await.unwrap();

        assert_eq!(result.get("visited_a"), Some(&json!(true)));
        assert_eq!(result.get("visited_b"), Some(&json!(true)));
    }

    #[tokio::test]
    async fn test_executor_with_checkpoints() {
        let node = FunctionNode::new("counter", |state| {
            Box::pin(async move {
                let count: i32 = state.get_typed("count").unwrap_or(0);
                Ok(state.set("count", json!(count + 1)))
            })
        });

        let edges = vec![
            Edge::required("start", "counter"),
            Edge::required("counter", "end"),
        ];

        let config = ExecutorConfig::default();
        let mut executor = GraphExecutor::new(config, ":memory:");

        executor.add_node(Box::new(node));
        executor.set_edges(edges);

        let state = State::new();
        let result = executor.execute("checkpoint-test", state).await.unwrap();

        assert_eq!(result.get_typed::<i32>("count"), Some(1));

        // Verify checkpoint was saved
        let checkpointer = executor.checkpointer().unwrap();
        assert_eq!(checkpointer.count("checkpoint-test").unwrap(), 1);
    }

    #[tokio::test]
    async fn test_executor_max_steps() {
        // Create a node that loops forever
        let node = FunctionNode::new("loop", |state| Box::pin(async move { Ok(state) }));

        // Create a loop: start -> loop -> loop (forever)
        let edges = vec![
            Edge::required("start", "loop"),
            Edge::required("loop", "loop"), // Infinite loop!
        ];

        let mut config = ExecutorConfig::default();
        config.max_steps = 5; // Limit to 5 steps

        let mut executor = GraphExecutor::new(config, ":memory:");
        executor.add_node(Box::new(node));
        executor.set_edges(edges);

        let state = State::new();
        let result = executor.execute("loop-test", state).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Max steps"));
    }

    #[tokio::test]
    async fn test_executor_events() {
        use std::sync::{Arc, Mutex};

        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let node = FunctionNode::new("test", |state| Box::pin(async move { Ok(state) }));

        let edges = vec![
            Edge::required("start", "test"),
            Edge::required("test", "end"),
        ];

        let config = ExecutorConfig::default();
        let mut executor = GraphExecutor::new(config, ":memory:");

        executor.add_node(Box::new(node));
        executor.set_edges(edges);
        executor.on_event(move |event| {
            events_clone.lock().unwrap().push(format!("{:?}", event));
        });

        let state = State::new();
        executor.execute("event-test", state).await.unwrap();

        let captured = events.lock().unwrap();
        assert!(captured.len() > 0);
        assert!(captured.iter().any(|e| e.contains("NodeStarted")));
        assert!(captured.iter().any(|e| e.contains("NodeCompleted")));
    }
}
