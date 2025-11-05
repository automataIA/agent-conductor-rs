//! Fluent API demonstration
//!
//! Shows how GraphBuilder makes graph construction elegant and type-safe.
//! Compares manual construction vs fluent API.

use conductor_builder::GraphBuilder;
use conductor_graph::{BoolCondition, FunctionNode};
use conductor_primitives::{AppendReducer, Message, State};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🎨 Fluent API Demonstration\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // === Build a complex workflow using fluent API ===

    println!("Building workflow with fluent API...\n");

    let mut graph = GraphBuilder::new()
        // Add nodes
        .add_node(Box::new(create_input_node()))
        .add_node(Box::new(create_analyzer_node()))
        .add_node(Box::new(create_processor_node()))
        .add_node(Box::new(create_fallback_node()))
        // Configure routing
        .add_edge("start", "input")
        .add_edge("input", "analyzer")
        .add_conditional_edge(
            "analyzer",
            BoolCondition::new("analysis_passed", "processor", "fallback"),
        )
        .add_edge("processor", "end")
        .add_edge("fallback", "end")
        // Configure execution
        .max_steps(20)
        .enable_checkpoints()
        .build(":memory:")?;

    println!("✅ Graph built successfully!\n");

    // === Execute the workflow ===

    println!("Executing workflow...\n");
    println!("Flow: start -> input -> analyzer -> [processor | fallback] -> end\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let initial_state = State::new();
    let result = graph.execute("demo-workflow", initial_state).await?;

    // === Display results ===

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("📊 Results:\n");

    let messages = result.get_messages();
    for (i, msg) in messages.iter().enumerate() {
        println!("   [{}] {}: {}", i + 1, msg.role, msg.content);
    }

    println!("\n   Analysis passed: {:?}", result.get("analysis_passed"));
    println!("   Status: {:?}", result.get("status"));

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✨ Fluent API Demo completed!");

    Ok(())
}

// === Helper functions to create nodes ===

fn create_input_node() -> FunctionNode<impl Fn(State) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<State>> + Send>> + Send + Sync> {
    FunctionNode::new("input", |state| {
        Box::pin(async move {
            println!("📥 Input: Receiving user request");

            let user_msg = Message::user("Analyze this data: [1, 2, 3, 4, 5]");
            let reducer = AppendReducer;
            let state = state.update("messages", json!([user_msg]), &reducer);

            println!("   ✓ Input captured\n");
            Ok(state)
        })
    })
}

fn create_analyzer_node() -> FunctionNode<impl Fn(State) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<State>> + Send>> + Send + Sync> {
    FunctionNode::new("analyzer", |state| {
        Box::pin(async move {
            println!("🔍 Analyzer: Checking data quality");

            let messages = state.get_messages();
            let input = &messages.last().unwrap().content;

            // Simple analysis: check if contains numbers
            let has_numbers = input.contains('[') && input.contains(']');
            let passed = has_numbers;

            println!("   Analysis: {}", if passed { "PASSED ✓" } else { "FAILED ✗" });
            println!();

            let analysis_msg = Message::assistant(format!(
                "Analysis: Data format is {}",
                if passed { "valid" } else { "invalid" }
            ));

            let reducer = AppendReducer;
            let state = state.update("messages", json!([analysis_msg]), &reducer);
            let state = state.set("analysis_passed", json!(passed));

            Ok(state)
        })
    })
}

fn create_processor_node() -> FunctionNode<impl Fn(State) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<State>> + Send>> + Send + Sync> {
    FunctionNode::new("processor", |state| {
        Box::pin(async move {
            println!("⚙️  Processor: Processing valid data");

            let result_msg = Message::assistant("Data processed: Sum = 15, Average = 3.0");

            let reducer = AppendReducer;
            let state = state.update("messages", json!([result_msg]), &reducer);
            let state = state.set("status", json!("success"));

            println!("   ✓ Processing complete\n");
            Ok(state)
        })
    })
}

fn create_fallback_node() -> FunctionNode<impl Fn(State) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<State>> + Send>> + Send + Sync> {
    FunctionNode::new("fallback", |state| {
        Box::pin(async move {
            println!("🔄 Fallback: Handling invalid data");

            let fallback_msg = Message::assistant("Error: Invalid data format. Please provide array.");

            let reducer = AppendReducer;
            let state = state.update("messages", json!([fallback_msg]), &reducer);
            let state = state.set("status", json!("fallback"));

            println!("   ✓ Fallback handled\n");
            Ok(state)
        })
    })
}
