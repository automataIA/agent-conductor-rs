//! Edge types for graph connectivity
//!
//! Edges define how nodes are connected in the graph.
//! Two types:
//! - **Required**: Always follow this path (A -> B)
//! - **Conditional**: Choose path based on state evaluation
//!
//! This follows LangGraph's edge concept with explicit routing logic.

use anyhow::Result;
use async_trait::async_trait;
use conductor_primitives::State;

/// Condition trait for conditional edges
///
/// Evaluates state and returns the name of the next node to execute.
/// This enables dynamic routing in graphs.
///
/// # Example
///
/// ```rust
/// use conductor_graph::Condition;
/// use conductor_primitives::State;
/// use async_trait::async_trait;
///
/// struct QualityCheck;
///
/// #[async_trait]
/// impl Condition for QualityCheck {
///     async fn evaluate(&self, state: &State) -> anyhow::Result<String> {
///         let score: i32 = state.get_typed("quality_score").unwrap_or(0);
///
///         if score >= 80 {
///             Ok("writer".to_string())
///         } else {
///             Ok("reviewer".to_string())
///         }
///     }
/// }
/// ```
#[async_trait]
pub trait Condition: Send + Sync {
    /// Evaluate the condition and return the next node name
    ///
    /// # Arguments
    /// * `state` - Current state to evaluate
    ///
    /// # Returns
    /// * `Result<String>` - Name of the next node to execute
    async fn evaluate(&self, state: &State) -> Result<String>;
}

/// Edge connecting nodes in the graph
#[derive(Clone)]
pub enum Edge {
    /// Required edge: always follows this path
    ///
    /// Represented as solid lines in LangGraph Studio
    Required {
        /// Source node name
        from: String,
        /// Destination node name
        to: String,
    },

    /// Conditional edge: chooses path based on condition
    ///
    /// Represented as dotted lines in LangGraph Studio
    Conditional {
        /// Source node name
        from: String,
        /// Condition function that determines next node
        /// Note: We use Arc to make Edge cloneable
        condition: std::sync::Arc<dyn Condition>,
    },
}

impl Edge {
    /// Create a required edge
    pub fn required(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self::Required {
            from: from.into(),
            to: to.into(),
        }
    }

    /// Create a conditional edge
    pub fn conditional(from: impl Into<String>, condition: impl Condition + 'static) -> Self {
        Self::Conditional {
            from: from.into(),
            condition: std::sync::Arc::new(condition),
        }
    }

    /// Get the source node name
    pub fn from(&self) -> &str {
        match self {
            Self::Required { from, .. } => from,
            Self::Conditional { from, .. } => from,
        }
    }

    /// Get the destination node name (for required edges)
    pub fn to(&self) -> Option<&str> {
        match self {
            Self::Required { to, .. } => Some(to),
            Self::Conditional { .. } => None,
        }
    }

    /// Evaluate the edge to get the next node name
    ///
    /// For Required edges, returns the fixed destination.
    /// For Conditional edges, evaluates the condition.
    pub async fn next_node(&self, state: &State) -> Result<String> {
        match self {
            Self::Required { to, .. } => Ok(to.clone()),
            Self::Conditional { condition, .. } => condition.evaluate(state).await,
        }
    }

    /// Check if this is a required edge
    pub fn is_required(&self) -> bool {
        matches!(self, Self::Required { .. })
    }

    /// Check if this is a conditional edge
    pub fn is_conditional(&self) -> bool {
        matches!(self, Self::Conditional { .. })
    }
}

impl std::fmt::Debug for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Required { from, to } => {
                write!(f, "Edge::Required {{ from: {}, to: {} }}", from, to)
            }
            Self::Conditional { from, .. } => {
                write!(f, "Edge::Conditional {{ from: {} }}", from)
            }
        }
    }
}

/// Simple condition that routes based on a boolean state value
///
/// # Example
///
/// ```rust
/// use conductor_graph::{BoolCondition, Edge};
/// use conductor_primitives::State;
/// use serde_json::json;
///
/// # async fn example() -> anyhow::Result<()> {
/// let condition = BoolCondition::new("is_valid", "valid_handler", "invalid_handler");
/// let edge = Edge::conditional("validator", condition);
///
/// let state = State::new().set("is_valid", json!(true));
/// let next = edge.next_node(&state).await?;
/// assert_eq!(next, "valid_handler");
/// # Ok(())
/// # }
/// ```
pub struct BoolCondition {
    key: String,
    if_true: String,
    if_false: String,
}

impl BoolCondition {
    /// Create a new boolean condition
    pub fn new(
        key: impl Into<String>,
        if_true: impl Into<String>,
        if_false: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            if_true: if_true.into(),
            if_false: if_false.into(),
        }
    }
}

#[async_trait]
impl Condition for BoolCondition {
    async fn evaluate(&self, state: &State) -> Result<String> {
        let value: bool = state.get_typed(&self.key).unwrap_or(false);
        if value {
            Ok(self.if_true.clone())
        } else {
            Ok(self.if_false.clone())
        }
    }
}

/// Function-based condition for custom routing logic
///
/// # Example
///
/// ```rust
/// use conductor_graph::{FunctionCondition, Edge};
/// use conductor_primitives::State;
/// use serde_json::json;
///
/// # async fn example() -> anyhow::Result<()> {
/// let condition = FunctionCondition::new(|state| {
///     Box::pin(async move {
///         let score: i32 = state.get_typed("score").unwrap_or(0);
///         if score >= 50 {
///             Ok("pass".to_string())
///         } else {
///             Ok("fail".to_string())
///         }
///     })
/// });
///
/// let edge = Edge::conditional("scorer", condition);
/// let state = State::new().set("score", json!(75));
/// let next = edge.next_node(&state).await?;
/// assert_eq!(next, "pass");
/// # Ok(())
/// # }
/// ```
pub struct FunctionCondition<F>
where
    F: Fn(&State) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String>> + Send>>
        + Send
        + Sync,
{
    func: F,
}

impl<F> FunctionCondition<F>
where
    F: Fn(&State) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String>> + Send>>
        + Send
        + Sync,
{
    /// Create a new function-based condition
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

#[async_trait]
impl<F> Condition for FunctionCondition<F>
where
    F: Fn(&State) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String>> + Send>>
        + Send
        + Sync,
{
    async fn evaluate(&self, state: &State) -> Result<String> {
        (self.func)(state).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_required_edge_creation() {
        let edge = Edge::required("a", "b");
        assert!(edge.is_required());
        assert!(!edge.is_conditional());
        assert_eq!(edge.from(), "a");
        assert_eq!(edge.to(), Some("b"));
    }

    #[tokio::test]
    async fn test_required_edge_next_node() {
        let edge = Edge::required("start", "end");
        let state = State::new();
        let next = edge.next_node(&state).await.unwrap();
        assert_eq!(next, "end");
    }

    #[tokio::test]
    async fn test_bool_condition_true() {
        let condition = BoolCondition::new("flag", "yes", "no");
        let state = State::new().set("flag", json!(true));
        let result = condition.evaluate(&state).await.unwrap();
        assert_eq!(result, "yes");
    }

    #[tokio::test]
    async fn test_bool_condition_false() {
        let condition = BoolCondition::new("flag", "yes", "no");
        let state = State::new().set("flag", json!(false));
        let result = condition.evaluate(&state).await.unwrap();
        assert_eq!(result, "no");
    }

    #[tokio::test]
    async fn test_bool_condition_missing() {
        let condition = BoolCondition::new("flag", "yes", "no");
        let state = State::new();
        let result = condition.evaluate(&state).await.unwrap();
        assert_eq!(result, "no"); // Default to false
    }

    #[tokio::test]
    async fn test_function_condition() {
        let condition = FunctionCondition::new(|state| {
            Box::pin(async move {
                let count: i32 = state.get_typed("count").unwrap_or(0);
                if count > 10 {
                    Ok("high".to_string())
                } else {
                    Ok("low".to_string())
                }
            })
        });

        let state = State::new().set("count", json!(15));
        let result = condition.evaluate(&state).await.unwrap();
        assert_eq!(result, "high");

        let state = State::new().set("count", json!(5));
        let result = condition.evaluate(&state).await.unwrap();
        assert_eq!(result, "low");
    }

    #[tokio::test]
    async fn test_conditional_edge() {
        let condition = BoolCondition::new("ready", "process", "wait");
        let edge = Edge::conditional("check", condition);

        assert!(edge.is_conditional());
        assert_eq!(edge.from(), "check");
        assert_eq!(edge.to(), None);

        let state = State::new().set("ready", json!(true));
        let next = edge.next_node(&state).await.unwrap();
        assert_eq!(next, "process");
    }
}
