//! Simple executor example
//!
//! Demonstrates basic usage:
//! - Creating a simple chain
//! - Executing with GraphExecutor
//! - Viewing results

use conductor_executor::{ExecutorConfig, GraphExecutor};
use conductor_graph::{Edge, FunctionNode};
use conductor_primitives::State;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔗 Simple Executor Example\n");

    // === Create 3 simple nodes ===

    let node_a = FunctionNode::new("node_a", |state| {
        Box::pin(async move {
            println!("✓ Node A: Setting value to 10");
            Ok(state.set("value", json!(10)))
        })
    });

    let node_b = FunctionNode::new("node_b", |state| {
        Box::pin(async move {
            let value: i32 = state.get_typed("value").unwrap_or(0);
            let new_value = value * 2;
            println!("✓ Node B: Multiplying by 2 -> {}", new_value);
            Ok(state.set("value", json!(new_value)))
        })
    });

    let node_c = FunctionNode::new("node_c", |state| {
        Box::pin(async move {
            let value: i32 = state.get_typed("value").unwrap_or(0);
            let new_value = value + 5;
            println!("✓ Node C: Adding 5 -> {}", new_value);
            Ok(state.set("value", json!(new_value)))
        })
    });

    // === Create edges: start -> A -> B -> C -> end ===

    let edges = vec![
        Edge::required("start", "node_a"),
        Edge::required("node_a", "node_b"),
        Edge::required("node_b", "node_c"),
        Edge::required("node_c", "end"),
    ];

    // === Setup executor ===

    let config = ExecutorConfig::default();
    let mut executor = GraphExecutor::new(config, ":memory:");

    executor.add_node(Box::new(node_a));
    executor.add_node(Box::new(node_b));
    executor.add_node(Box::new(node_c));
    executor.set_edges(edges);

    // === Execute ===

    println!("Executing: start -> A -> B -> C -> end\n");

    let state = State::new();
    let result = executor.execute("simple-workflow", state).await?;

    // === Display result ===

    let final_value: i32 = result.get_typed("value").unwrap();
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Final value: {}", final_value);
    println!("Expected: (10 * 2) + 5 = 25");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    assert_eq!(final_value, 25);
    println!("\n✅ Test passed!");

    Ok(())
}
