//! Router for graph edge evaluation
//!
//! Determines the next node to execute based on edges and state.
//! Handles both required and conditional edges.

use anyhow::{Context, Result};
use conductor_graph::Edge;
use conductor_primitives::State;
use std::collections::HashMap;

/// Router manages edge traversal in graphs
///
/// Given a current node and available edges, determines the next node.
///
/// # Example
///
/// ```rust
/// use conductor_executor::Router;
/// use conductor_graph::Edge;
/// use conductor_primitives::State;
///
/// # async fn example() -> anyhow::Result<()> {
/// let edges = vec![
///     Edge::required("start", "process"),
///     Edge::required("process", "end"),
/// ];
///
/// let router = Router::new(edges);
/// let state = State::new();
///
/// let next = router.next_node("start", &state).await?;
/// assert_eq!(next, Some("process".to_string()));
/// # Ok(())
/// # }
/// ```
pub struct Router {
    /// Map from node name to outgoing edges
    edges_map: HashMap<String, Vec<Edge>>,
}

impl Router {
    /// Create a new router from edges
    pub fn new(edges: Vec<Edge>) -> Self {
        let mut edges_map: HashMap<String, Vec<Edge>> = HashMap::new();

        for edge in edges {
            let from = edge.from().to_string();
            edges_map.entry(from).or_insert_with(Vec::new).push(edge);
        }

        Self { edges_map }
    }

    /// Get the next node to execute
    ///
    /// # Arguments
    /// * `current_node` - Current node name
    /// * `state` - Current state (used for conditional edges)
    ///
    /// # Returns
    /// * `Some(node_name)` if next node exists
    /// * `None` if current node is terminal (no outgoing edges)
    pub async fn next_node(&self, current_node: &str, state: &State) -> Result<Option<String>> {
        // Get outgoing edges for current node
        let edges = match self.edges_map.get(current_node) {
            Some(edges) => edges,
            None => return Ok(None), // Terminal node
        };

        // If no edges, terminal node
        if edges.is_empty() {
            return Ok(None);
        }

        // Evaluate first edge (in production, could support multiple edges with priority)
        let next = edges[0].next_node(state).await.with_context(|| {
            format!(
                "Failed to evaluate edge from node '{}'",
                current_node
            )
        })?;

        Ok(Some(next))
    }

    /// Check if a node is terminal (has no outgoing edges)
    pub fn is_terminal(&self, node: &str) -> bool {
        !self.edges_map.contains_key(node) || self.edges_map.get(node).unwrap().is_empty()
    }

    /// Get all outgoing edges for a node
    pub fn outgoing_edges(&self, node: &str) -> Vec<&Edge> {
        self.edges_map
            .get(node)
            .map(|edges| edges.iter().collect())
            .unwrap_or_default()
    }

    /// Get all node names that have outgoing edges
    pub fn nodes_with_edges(&self) -> Vec<&str> {
        self.edges_map.keys().map(|s| s.as_str()).collect()
    }

    /// Validate graph structure
    ///
    /// Checks for:
    /// - Unreachable nodes (referenced but never defined)
    /// - Cycles (optional check)
    pub fn validate(&self, node_names: &[&str]) -> Result<()> {
        // Collect all referenced destination nodes
        let mut referenced_nodes = std::collections::HashSet::new();

        for edges in self.edges_map.values() {
            for edge in edges {
                if edge.is_required() {
                    if let Some(to) = edge.to() {
                        referenced_nodes.insert(to);
                    }
                }
            }
        }

        // Check if all referenced nodes exist
        for referenced in referenced_nodes {
            if !node_names.contains(&referenced) && referenced != "end" {
                anyhow::bail!("Referenced node '{}' does not exist", referenced);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conductor_graph::BoolCondition;

    #[tokio::test]
    async fn test_router_required_edge() {
        let edges = vec![
            Edge::required("start", "middle"),
            Edge::required("middle", "end"),
        ];

        let router = Router::new(edges);
        let state = State::new();

        let next = router.next_node("start", &state).await.unwrap();
        assert_eq!(next, Some("middle".to_string()));

        let next = router.next_node("middle", &state).await.unwrap();
        assert_eq!(next, Some("end".to_string()));

        let next = router.next_node("end", &state).await.unwrap();
        assert_eq!(next, None); // Terminal
    }

    #[tokio::test]
    async fn test_router_conditional_edge() {
        use serde_json::json;

        let condition = BoolCondition::new("go_left", "left", "right");
        let edges = vec![Edge::conditional("fork", condition)];

        let router = Router::new(edges);

        // Test left path
        let state = State::new().set("go_left", json!(true));
        let next = router.next_node("fork", &state).await.unwrap();
        assert_eq!(next, Some("left".to_string()));

        // Test right path
        let state = State::new().set("go_left", json!(false));
        let next = router.next_node("fork", &state).await.unwrap();
        assert_eq!(next, Some("right".to_string()));
    }

    #[tokio::test]
    async fn test_router_terminal_node() {
        let edges = vec![Edge::required("start", "end")];
        let router = Router::new(edges);

        assert!(!router.is_terminal("start"));
        assert!(router.is_terminal("end"));
        assert!(router.is_terminal("nonexistent"));
    }

    #[tokio::test]
    async fn test_router_outgoing_edges() {
        let edges = vec![
            Edge::required("start", "a"),
            Edge::required("start", "b"),
            Edge::required("middle", "end"),
        ];

        let router = Router::new(edges);

        let outgoing = router.outgoing_edges("start");
        assert_eq!(outgoing.len(), 2);

        let outgoing = router.outgoing_edges("middle");
        assert_eq!(outgoing.len(), 1);

        let outgoing = router.outgoing_edges("nonexistent");
        assert_eq!(outgoing.len(), 0);
    }

    #[tokio::test]
    async fn test_router_nodes_with_edges() {
        let edges = vec![
            Edge::required("a", "b"),
            Edge::required("b", "c"),
            Edge::required("c", "d"),
        ];

        let router = Router::new(edges);
        let mut nodes = router.nodes_with_edges();
        nodes.sort();

        assert_eq!(nodes, vec!["a", "b", "c"]);
    }

    #[tokio::test]
    async fn test_router_validate() {
        let edges = vec![
            Edge::required("start", "middle"),
            Edge::required("middle", "end"),
        ];

        let router = Router::new(edges);

        // Valid: all referenced nodes exist
        let result = router.validate(&["start", "middle", "end"]);
        assert!(result.is_ok());

        // Invalid: "middle" is referenced but not in node list
        let result = router.validate(&["start", "end"]);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_router_empty() {
        let router = Router::new(vec![]);
        let state = State::new();

        let next = router.next_node("any", &state).await.unwrap();
        assert_eq!(next, None);
    }
}
