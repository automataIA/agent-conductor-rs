//! Complete workflow example with executor
//!
//! Demonstrates:
//! - GraphExecutor with multiple nodes
//! - Conditional routing
//! - Checkpointing
//! - Event handling
//! - Resume from checkpoint
//!
//! Flow:
//! ```
//! start -> input -> validator -> [valid? yes -> processor -> end]
//!                                [valid? no  -> error_handler -> end]
//! ```

use conductor_executor::{ExecutionEvent, ExecutorConfig, GraphExecutor};
use conductor_graph::{BoolCondition, Edge, FunctionNode};
use conductor_primitives::{AppendReducer, Message, State};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Complete Workflow Example\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // === Create Nodes ===

    // Node 1: Input - Receives user input
    let input_node = FunctionNode::new("input", |state| {
        Box::pin(async move {
            println!("📥 Input Node: Receiving user input");
            let user_msg = Message::user("Process invoice #12345");

            let reducer = AppendReducer;
            let state = state.update("messages", json!([user_msg]), &reducer);
            println!("   ✓ Input received\n");

            Ok(state)
        })
    });

    // Node 2: Validator - Validates input
    let validator = FunctionNode::new("validator", |state| {
        Box::pin(async move {
            println!("🔍 Validator Node: Checking input validity");

            let messages = state.get_messages();
            let last_msg = messages.last().unwrap();

            // Simple validation: check if message contains "invoice"
            let is_valid = last_msg.content.contains("invoice");

            println!(
                "   {} Validation: {}",
                if is_valid { "✓" } else { "✗" },
                if is_valid { "PASSED" } else { "FAILED" }
            );
            println!();

            Ok(state.set("is_valid", json!(is_valid)))
        })
    });

    // Node 3: Processor - Processes valid inputs
    let processor = FunctionNode::new("processor", |state| {
        Box::pin(async move {
            println!("⚙️  Processor Node: Processing valid input");

            let messages = state.get_messages();
            let input = &messages.last().unwrap().content;

            // Extract invoice number (simple regex simulation)
            let invoice_id = input
                .split('#')
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .unwrap_or("unknown");

            let result = format!("Invoice {} processed successfully", invoice_id);
            let response = Message::assistant(&result);

            let reducer = AppendReducer;
            let state = state.update("messages", json!([response]), &reducer);
            let state = state.set("status", json!("success"));

            println!("   ✓ {}\n", result);

            Ok(state)
        })
    });

    // Node 4: Error Handler - Handles invalid inputs
    let error_handler = FunctionNode::new("error_handler", |state| {
        Box::pin(async move {
            println!("❌ Error Handler Node: Handling invalid input");

            let error_msg = Message::assistant("Error: Invalid input format");

            let reducer = AppendReducer;
            let state = state.update("messages", json!([error_msg]), &reducer);
            let state = state.set("status", json!("error"));

            println!("   ✓ Error handled\n");

            Ok(state)
        })
    });

    // === Create Edges ===

    let edges = vec![
        Edge::required("start", "input"),
        Edge::required("input", "validator"),
        Edge::conditional(
            "validator",
            BoolCondition::new("is_valid", "processor", "error_handler"),
        ),
        Edge::required("processor", "end"),
        Edge::required("error_handler", "end"),
    ];

    // === Setup Executor ===

    let config = ExecutorConfig {
        max_steps: 50,
        enable_checkpoints: true,
        start_node: "start".to_string(),
        terminal_nodes: vec!["end".to_string()],
    };

    let mut executor = GraphExecutor::new(config, "workflow.db");

    // Add nodes
    executor.add_node(Box::new(input_node));
    executor.add_node(Box::new(validator));
    executor.add_node(Box::new(processor));
    executor.add_node(Box::new(error_handler));
    executor.set_edges(edges);

    // Add event handler for monitoring
    executor.on_event(|event| match event {
        ExecutionEvent::NodeStarted { node_name, step } => {
            println!("🔄 [Step {}] Starting node: {}", step, node_name);
        }
        ExecutionEvent::CheckpointSaved { step, .. } => {
            println!("💾 [Step {}] Checkpoint saved", step);
        }
        ExecutionEvent::ExecutionCompleted { total_steps, .. } => {
            println!("\n✅ Execution completed in {} steps", total_steps);
        }
        _ => {}
    });

    // === Execute Workflow ===

    println!("📍 Executing workflow: start -> input -> validator -> [processor | error_handler] -> end\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let initial_state = State::new();
    let result = executor.execute("workflow-001", initial_state).await?;

    // === Display Results ===

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("📊 Final State:\n");

    let messages = result.get_messages();
    println!("   Messages: {}", messages.len());
    for (i, msg) in messages.iter().enumerate() {
        println!("   [{}] {}: {}", i + 1, msg.role, msg.content);
    }

    println!("\n   Status: {:?}", result.get_typed::<String>("status"));
    println!("   Valid: {:?}", result.get_typed::<bool>("is_valid"));

    // === Demonstrate Checkpoint Access ===

    if let Some(checkpointer) = executor.checkpointer() {
        println!("\n📦 Checkpoints saved:");
        let checkpoints = checkpointer.list("workflow-001")?;
        for cp in checkpoints {
            println!(
                "   Step {}: {} (timestamp: {})",
                cp.step, cp.node_name, cp.timestamp
            );
        }
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✨ Workflow completed successfully!");

    Ok(())
}
