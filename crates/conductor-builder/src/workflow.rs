//! Workflow management and templates
//!
//! Provides higher-level abstractions for managing workflows:
//! - Workflow templates
//! - Session management
//! - Batch execution

use anyhow::{Context, Result};
use conductor_executor::GraphExecutor;
use conductor_primitives::State;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Workflow template - a reusable graph configuration
///
/// Templates can be instantiated multiple times with different sessions
pub struct WorkflowTemplate {
    name: String,
    description: String,
    executor_factory: Box<dyn Fn() -> Result<GraphExecutor> + Send + Sync>,
}

impl WorkflowTemplate {
    /// Create a new workflow template
    ///
    /// # Arguments
    /// * `name` - Template name
    /// * `description` - Human-readable description
    /// * `factory` - Function that creates a new executor instance
    pub fn new<F>(name: impl Into<String>, description: impl Into<String>, factory: F) -> Self
    where
        F: Fn() -> Result<GraphExecutor> + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            description: description.into(),
            executor_factory: Box::new(factory),
        }
    }

    /// Get template name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get template description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Create a new executor instance from this template
    pub fn instantiate(&self) -> Result<GraphExecutor> {
        (self.executor_factory)()
    }
}

/// Workflow session - a running instance of a workflow
pub struct WorkflowSession {
    session_id: String,
    template_name: String,
    executor: GraphExecutor,
    execution_count: usize,
}

impl WorkflowSession {
    /// Create a new workflow session
    pub fn new(
        session_id: impl Into<String>,
        template_name: impl Into<String>,
        executor: GraphExecutor,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            template_name: template_name.into(),
            executor,
            execution_count: 0,
        }
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get template name
    pub fn template_name(&self) -> &str {
        &self.template_name
    }

    /// Get execution count
    pub fn execution_count(&self) -> usize {
        self.execution_count
    }

    /// Execute the workflow with given state
    pub async fn execute(&mut self, state: State) -> Result<State> {
        let workflow_id = format!("{}-{}", self.session_id, self.execution_count);
        let result = self.executor.execute(&workflow_id, state).await?;
        self.execution_count += 1;
        Ok(result)
    }

    /// Resume from last checkpoint
    pub async fn resume(&mut self) -> Result<State> {
        let workflow_id = format!("{}-{}", self.session_id, self.execution_count - 1);
        self.executor.resume(&workflow_id).await
    }
}

/// Workflow manager - manages templates and sessions
///
/// # Example
///
/// ```rust
/// use conductor_builder::{WorkflowManager, GraphBuilder};
/// use conductor_graph::FunctionNode;
/// use serde_json::json;
///
/// # async fn example() -> anyhow::Result<()> {
/// let mut manager = WorkflowManager::new();
///
/// // Register template
/// manager.register_template("simple", "A simple workflow", || {
///     let node = FunctionNode::new("process", |state| {
///         Box::pin(async move { Ok(state.set("done", json!(true))) })
///     });
///
///     GraphBuilder::new()
///         .add_node(Box::new(node))
///         .add_edge("start", "process")
///         .add_edge("process", "end")
///         .build(":memory:")
/// });
///
/// // Create session
/// let session = manager.create_session("session-1", "simple")?;
/// # Ok(())
/// # }
/// ```
pub struct WorkflowManager {
    templates: HashMap<String, WorkflowTemplate>,
    sessions: Arc<Mutex<HashMap<String, WorkflowSession>>>,
}

impl WorkflowManager {
    /// Create a new workflow manager
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a workflow template
    ///
    /// # Arguments
    /// * `name` - Template identifier
    /// * `description` - Human-readable description
    /// * `factory` - Function that creates executor instances
    pub fn register_template<F>(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        factory: F,
    ) where
        F: Fn() -> Result<GraphExecutor> + Send + Sync + 'static,
    {
        let name_str = name.into();
        let template = WorkflowTemplate::new(name_str.clone(), description, factory);
        self.templates.insert(name_str, template);
    }

    /// Create a new session from a template
    ///
    /// # Arguments
    /// * `session_id` - Unique session identifier
    /// * `template_name` - Name of registered template
    pub fn create_session(
        &self,
        session_id: impl Into<String>,
        template_name: &str,
    ) -> Result<String> {
        let template = self
            .templates
            .get(template_name)
            .with_context(|| format!("Template '{}' not found", template_name))?;

        let executor = template.instantiate()?;
        let session_id_str = session_id.into();
        let session = WorkflowSession::new(&session_id_str, template_name, executor);

        self.sessions
            .lock()
            .unwrap()
            .insert(session_id_str.clone(), session);

        Ok(session_id_str)
    }

    /// Execute workflow in a session
    pub async fn execute(&self, session_id: &str, state: State) -> Result<State> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions
            .get_mut(session_id)
            .with_context(|| format!("Session '{}' not found", session_id))?;

        session.execute(state).await
    }

    /// Get session info
    pub fn get_session(&self, session_id: &str) -> Option<(String, usize)> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(session_id).map(|s| {
            (
                s.template_name().to_string(),
                s.execution_count(),
            )
        })
    }

    /// List all active sessions
    pub fn list_sessions(&self) -> Vec<String> {
        let sessions = self.sessions.lock().unwrap();
        sessions.keys().cloned().collect()
    }

    /// List all registered templates
    pub fn list_templates(&self) -> Vec<(String, String)> {
        self.templates
            .values()
            .map(|t| (t.name().to_string(), t.description().to_string()))
            .collect()
    }

    /// Delete a session
    pub fn delete_session(&self, session_id: &str) -> bool {
        self.sessions.lock().unwrap().remove(session_id).is_some()
    }

    /// Get number of active sessions
    pub fn session_count(&self) -> usize {
        self.sessions.lock().unwrap().len()
    }

    /// Get number of registered templates
    pub fn template_count(&self) -> usize {
        self.templates.len()
    }
}

impl Default for WorkflowManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::GraphBuilder;
    use conductor_graph::FunctionNode;
    use serde_json::json;

    #[test]
    fn test_workflow_template_creation() {
        let template = WorkflowTemplate::new("test", "A test template", || {
            GraphBuilder::new()
                .add_edge("start", "end")
                .build(":memory:")
        });

        assert_eq!(template.name(), "test");
        assert_eq!(template.description(), "A test template");
    }

    #[test]
    fn test_workflow_manager_creation() {
        let manager = WorkflowManager::new();
        assert_eq!(manager.template_count(), 0);
        assert_eq!(manager.session_count(), 0);
    }

    #[test]
    fn test_register_template() {
        let mut manager = WorkflowManager::new();

        manager.register_template("simple", "Simple workflow", || {
            GraphBuilder::new()
                .add_edge("start", "end")
                .build(":memory:")
        });

        assert_eq!(manager.template_count(), 1);

        let templates = manager.list_templates();
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].0, "simple");
    }

    #[test]
    fn test_create_session() {
        let mut manager = WorkflowManager::new();

        manager.register_template("test", "Test template", || {
            GraphBuilder::new()
                .add_edge("start", "end")
                .build(":memory:")
        });

        let session_id = manager.create_session("session-1", "test").unwrap();
        assert_eq!(session_id, "session-1");
        assert_eq!(manager.session_count(), 1);
    }

    #[test]
    fn test_create_session_invalid_template() {
        let manager = WorkflowManager::new();

        let result = manager.create_session("session-1", "nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_execute_session() {
        let mut manager = WorkflowManager::new();

        manager.register_template("counter", "Counter workflow", || {
            let node = FunctionNode::new("counter", |state| {
                Box::pin(async move {
                    let count: i32 = state.get_typed("count").unwrap_or(0);
                    Ok(state.set("count", json!(count + 1)))
                })
            });

            GraphBuilder::new()
                .add_node(Box::new(node))
                .add_edge("start", "counter")
                .add_edge("counter", "end")
                .build(":memory:")
        });

        manager.create_session("s1", "counter").unwrap();

        let state = State::new();
        let result = manager.execute("s1", state).await.unwrap();

        assert_eq!(result.get_typed::<i32>("count"), Some(1));
    }

    #[test]
    fn test_get_session() {
        let mut manager = WorkflowManager::new();

        manager.register_template("test", "Test", || {
            GraphBuilder::new()
                .add_edge("start", "end")
                .build(":memory:")
        });

        manager.create_session("s1", "test").unwrap();

        let info = manager.get_session("s1");
        assert!(info.is_some());

        let (template_name, exec_count) = info.unwrap();
        assert_eq!(template_name, "test");
        assert_eq!(exec_count, 0);
    }

    #[test]
    fn test_list_sessions() {
        let mut manager = WorkflowManager::new();

        manager.register_template("test", "Test", || {
            GraphBuilder::new()
                .add_edge("start", "end")
                .build(":memory:")
        });

        manager.create_session("s1", "test").unwrap();
        manager.create_session("s2", "test").unwrap();

        let sessions = manager.list_sessions();
        assert_eq!(sessions.len(), 2);
        assert!(sessions.contains(&"s1".to_string()));
        assert!(sessions.contains(&"s2".to_string()));
    }

    #[test]
    fn test_delete_session() {
        let mut manager = WorkflowManager::new();

        manager.register_template("test", "Test", || {
            GraphBuilder::new()
                .add_edge("start", "end")
                .build(":memory:")
        });

        manager.create_session("s1", "test").unwrap();
        assert_eq!(manager.session_count(), 1);

        let deleted = manager.delete_session("s1");
        assert!(deleted);
        assert_eq!(manager.session_count(), 0);

        let deleted_again = manager.delete_session("s1");
        assert!(!deleted_again);
    }

    #[tokio::test]
    async fn test_multi_execution_session() {
        let mut manager = WorkflowManager::new();

        manager.register_template("counter", "Counter", || {
            let node = FunctionNode::new("inc", |state| {
                Box::pin(async move {
                    let count: i32 = state.get_typed("count").unwrap_or(0);
                    Ok(state.set("count", json!(count + 10)))
                })
            });

            GraphBuilder::new()
                .add_node(Box::new(node))
                .add_edge("start", "inc")
                .add_edge("inc", "end")
                .build(":memory:")
        });

        manager.create_session("s1", "counter").unwrap();

        // Execute 3 times
        let state1 = State::new();
        let result1 = manager.execute("s1", state1).await.unwrap();
        assert_eq!(result1.get_typed::<i32>("count"), Some(10));

        let state2 = State::new();
        let result2 = manager.execute("s1", state2).await.unwrap();
        assert_eq!(result2.get_typed::<i32>("count"), Some(10));

        // Check execution count increased
        let (_, exec_count) = manager.get_session("s1").unwrap();
        assert_eq!(exec_count, 2);
    }
}
