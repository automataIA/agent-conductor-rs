//! Workflow Manager demonstration
//!
//! Shows how to use templates and sessions for reusable workflows.

use conductor_builder::{GraphBuilder, WorkflowManager};
use conductor_graph::FunctionNode;
use conductor_primitives::State;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔧 Workflow Manager Demonstration\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut manager = WorkflowManager::new();

    // === Register workflow templates ===

    println!("📝 Registering workflow templates...\n");

    // Template 1: Counter workflow
    manager.register_template(
        "counter",
        "Increments a counter each execution",
        || {
            let node = FunctionNode::new("increment", |state| {
                Box::pin(async move {
                    let count: i32 = state.get_typed("count").unwrap_or(0);
                    let new_count = count + 1;
                    println!("   Counter: {} -> {}", count, new_count);
                    Ok(state.set("count", json!(new_count)))
                })
            });

            GraphBuilder::new()
                .add_node(Box::new(node))
                .add_edge("start", "increment")
                .add_edge("increment", "end")
                .build(":memory:")
        },
    );

    // Template 2: Multiplier workflow
    manager.register_template(
        "multiplier",
        "Multiplies a value by 2",
        || {
            let node = FunctionNode::new("multiply", |state| {
                Box::pin(async move {
                    let value: i32 = state.get_typed("value").unwrap_or(1);
                    let new_value = value * 2;
                    println!("   Multiplier: {} -> {}", value, new_value);
                    Ok(state.set("value", json!(new_value)))
                })
            });

            GraphBuilder::new()
                .add_node(Box::new(node))
                .add_edge("start", "multiply")
                .add_edge("multiply", "end")
                .build(":memory:")
        },
    );

    println!("✅ Registered {} templates\n", manager.template_count());

    // === List templates ===

    println!("📋 Available templates:");
    for (name, desc) in manager.list_templates() {
        println!("   - {}: {}", name, desc);
    }
    println!();

    // === Create sessions ===

    println!("🔄 Creating sessions...\n");

    let session1 = manager.create_session("user-alice-counter", "counter")?;
    let session2 = manager.create_session("user-bob-counter", "counter")?;
    let session3 = manager.create_session("user-alice-multiplier", "multiplier")?;

    println!("✅ Created {} sessions\n", manager.session_count());

    // === Execute workflows in sessions ===

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("⚙️  Executing workflows...\n");

    // Alice's counter: Execute 3 times
    println!("👤 Alice's counter session:");
    for i in 1..=3 {
        let state = State::new().set("count", json!(i * 10));
        let result = manager.execute(&session1, state).await?;
        println!("   Execution {}: count = {}", i, result.get_typed::<i32>("count").unwrap());
    }
    println!();

    // Bob's counter: Execute 2 times
    println!("👤 Bob's counter session:");
    for i in 1..=2 {
        let state = State::new().set("count", json!(i * 5));
        let result = manager.execute(&session2, state).await?;
        println!("   Execution {}: count = {}", i, result.get_typed::<i32>("count").unwrap());
    }
    println!();

    // Alice's multiplier: Execute 2 times
    println!("👤 Alice's multiplier session:");
    for i in 1..=2 {
        let state = State::new().set("value", json!(i));
        let result = manager.execute(&session3, state).await?;
        println!("   Execution {}: value = {}", i, result.get_typed::<i32>("value").unwrap());
    }
    println!();

    // === Show session info ===

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("📊 Session Status:\n");

    for session_id in manager.list_sessions() {
        if let Some((template, exec_count)) = manager.get_session(&session_id) {
            println!("   {}", session_id);
            println!("      Template: {}", template);
            println!("      Executions: {}", exec_count);
            println!();
        }
    }

    // === Cleanup ===

    println!("🧹 Cleaning up...");
    manager.delete_session(&session1);
    println!("   Deleted session: {}", session1);
    println!("   Remaining sessions: {}\n", manager.session_count());

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✨ Workflow Manager Demo completed!");

    Ok(())
}
