//! Multi-agent workflow example
//!
//! This example demonstrates a realistic multi-agent system:
//! 1. Researcher: Gathers information
//! 2. Quality Check: Evaluates if research is good enough
//! 3. Reviewer: Provides feedback if quality is low
//! 4. Writer: Creates final report
//!
//! Flow:
//! ```
//! Start -> Researcher -> Quality Check
//!                            |
//!                  +---------+---------+
//!                  |                   |
//!              (quality < 7)       (quality >= 7)
//!                  |                   |
//!                  v                   v
//!              Reviewer ----------> Writer -> End
//!                  |
//!                  +----(loop back to Researcher)
//! ```
//!
//! Prerequisites:
//! - Ollama running: `ollama serve`
//! - Model: `ollama pull llama3.2:1b`
//!
//! Run with:
//! ```bash
//! cargo run --example multi_agent_workflow --package conductor-graph
//! ```

use conductor_graph::{BoolCondition, Condition, Edge, FunctionNode, LlmNode, Node};
use conductor_primitives::{LlmClient, LlmConfig, Message, State};
use serde_json::json;

/// Quality check condition - routes based on quality score
struct QualityCheckCondition;

#[async_trait::async_trait]
impl Condition for QualityCheckCondition {
    async fn evaluate(&self, state: &State) -> anyhow::Result<String> {
        let quality_score: i32 = state.get_typed("quality_score").unwrap_or(0);

        if quality_score >= 7 {
            println!("✅ Quality check passed (score: {})", quality_score);
            Ok("writer".to_string())
        } else {
            println!("❌ Quality check failed (score: {})", quality_score);
            Ok("reviewer".to_string())
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Agent Conductor - Multi-Agent Workflow Example\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // === Setup LLM Client ===
    let config = LlmConfig::ollama("llama3.2:1b");
    let client = LlmClient::new(config)?;
    println!("✅ LLM Client configured\n");

    // === Create Nodes ===

    // 1. Researcher Node
    let researcher = LlmNode::new("researcher", client.clone())
        .with_system_prompt("You are a researcher. Gather key facts about the topic. Be concise (2-3 sentences).");

    // 2. Quality Checker (simulated with function)
    let quality_checker = FunctionNode::new("quality_checker", |state| {
        Box::pin(async move {
            let messages = state.get_messages();
            let last_response = messages
                .last()
                .map(|m| &m.content)
                .unwrap_or(&String::new());

            // Simple quality check: length and content
            let score = if last_response.len() > 50 && last_response.contains("Rust") {
                8 // Good quality
            } else {
                5 // Needs improvement
            };

            println!("\n🔍 Quality Checker: Score = {}/10", score);
            Ok(state.set("quality_score", json!(score)))
        })
    });

    // 3. Reviewer Node
    let reviewer = LlmNode::new("reviewer", client.clone())
        .with_system_prompt("You are a critical reviewer. Identify what's missing in the research. Be specific.");

    // 4. Writer Node
    let writer = LlmNode::new("writer", client.clone())
        .with_system_prompt("You are a technical writer. Create a polished summary based on the research.");

    // === Create Edges ===
    let edge_start_to_researcher = Edge::required("start", "researcher");
    let edge_researcher_to_checker = Edge::required("researcher", "quality_checker");
    let edge_checker_conditional = Edge::conditional("quality_checker", QualityCheckCondition);
    let edge_reviewer_to_researcher = Edge::required("reviewer", "researcher");
    let edge_writer_to_end = Edge::required("writer", "end");

    println!("✅ Graph structure created:");
    println!("   - Nodes: researcher, quality_checker, reviewer, writer");
    println!("   - Edges: start -> researcher -> quality_checker -> [conditional]");
    println!("   - Conditional: quality >= 7 -> writer, else -> reviewer\n");

    // === Initialize State ===
    let initial_messages = vec![Message::user("What is Rust programming language?")];
    let mut state = State::new().set("messages", json!(initial_messages));

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📝 User Query: What is Rust programming language?");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // === Manual Execution (simulating executor) ===
    let mut current_node = "researcher";
    let mut iteration = 0;
    let max_iterations = 5;

    while current_node != "end" && iteration < max_iterations {
        iteration += 1;
        println!("🔄 Iteration {} - Current Node: {}", iteration, current_node);
        println!("─────────────────────────────────────────────────\n");

        // Execute current node
        state = match current_node {
            "researcher" => {
                println!("🔬 Researcher: Gathering information...\n");
                let new_state = researcher.invoke(state).await?;
                let messages = new_state.get_messages();
                let response = &messages.last().unwrap().content;
                println!("📄 Response: {}\n", response);
                new_state
            }
            "quality_checker" => {
                println!("🔍 Quality Checker: Evaluating research...\n");
                quality_checker.invoke(state).await?
            }
            "reviewer" => {
                println!("👀 Reviewer: Providing feedback...\n");
                let new_state = reviewer.invoke(state).await?;
                let messages = new_state.get_messages();
                let feedback = &messages.last().unwrap().content;
                println!("💬 Feedback: {}\n", feedback);
                new_state
            }
            "writer" => {
                println!("✍️  Writer: Creating final report...\n");
                let new_state = writer.invoke(state).await?;
                let messages = new_state.get_messages();
                let report = &messages.last().unwrap().content;
                println!("📋 Final Report:\n{}\n", report);
                new_state
            }
            _ => {
                println!("❌ Unknown node: {}", current_node);
                break;
            }
        };

        // Determine next node using edges
        current_node = match current_node {
            "researcher" => "quality_checker",
            "quality_checker" => {
                let next = edge_checker_conditional.next_node(&state).await?;
                &next
            }
            "reviewer" => "researcher", // Loop back
            "writer" => "end",
            _ => "end",
        };

        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Workflow completed in {} iterations", iteration);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Display final state
    let final_messages = state.get_messages();
    println!("📊 Final State:");
    println!("   - Total messages: {}", final_messages.len());
    println!("   - Quality score: {:?}", state.get_typed::<i32>("quality_score"));
    println!("\n✨ Example completed successfully!");

    Ok(())
}
