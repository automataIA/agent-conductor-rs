//! Simple chain example - three nodes in sequence
//!
//! This example shows the basics:
//! - Creating nodes
//! - Connecting them with required edges
//! - Executing a simple A -> B -> C chain
//!
//! Flow: Input -> Translator -> Summarizer -> Output
//!
//! Prerequisites:
//! - Ollama running: `ollama serve`
//! - Model: `ollama pull llama3.2:1b`
//!
//! Run with:
//! ```bash
//! cargo run --example simple_chain --package conductor-graph
//! ```

use conductor_graph::{Edge, FunctionNode, LlmNode, Node};
use conductor_primitives::{LlmClient, LlmConfig, Message, State};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔗 Agent Conductor - Simple Chain Example\n");

    // === Setup ===
    let config = LlmConfig::ollama("llama3.2:1b");
    let client = LlmClient::new(config)?;
    println!("✅ LLM Client ready\n");

    // === Create Nodes ===

    // Node 1: Translator (English -> Spanish)
    let translator = LlmNode::new("translator", client.clone())
        .with_system_prompt("You are a translator. Translate the user's message to Spanish. Only output the translation.");

    // Node 2: Summarizer
    let summarizer = LlmNode::new("summarizer", client.clone())
        .with_system_prompt("You are a summarizer. Make the text extremely concise (one short sentence).");

    // Node 3: Counter (adds word count to state)
    let counter = FunctionNode::new("counter", |state| {
        Box::pin(async move {
            let messages = state.get_messages();
            let last_msg = messages.last().unwrap();
            let word_count = last_msg.content.split_whitespace().count();

            println!("📊 Word Counter: {} words\n", word_count);
            Ok(state.set("word_count", json!(word_count)))
        })
    });

    // === Create Edges ===
    let edges = vec![
        Edge::required("start", "translator"),
        Edge::required("translator", "summarizer"),
        Edge::required("summarizer", "counter"),
        Edge::required("counter", "end"),
    ];

    println!("📍 Graph: start -> translator -> summarizer -> counter -> end\n");

    // === Initialize State ===
    let input = "Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.";
    println!("📝 Input: {}\n", input);

    let mut state = State::new().set("messages", json!([Message::user(input)]));

    // === Execute Chain ===
    let nodes = vec!["translator", "summarizer", "counter"];

    for node_name in nodes {
        println!("🔄 Executing: {}", node_name);
        println!("───────────────────────────────────────\n");

        state = match node_name {
            "translator" => {
                let new_state = translator.invoke(state).await?;
                let messages = new_state.get_messages();
                println!("🌐 Translation: {}\n", messages.last().unwrap().content);
                new_state
            }
            "summarizer" => {
                let new_state = summarizer.invoke(state).await?;
                let messages = new_state.get_messages();
                println!("📄 Summary: {}\n", messages.last().unwrap().content);
                new_state
            }
            "counter" => counter.invoke(state).await?,
            _ => state,
        };
    }

    // === Display Results ===
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("✅ Chain completed successfully!\n");
    println!("📊 Final State:");
    println!("   - Messages: {}", state.get_messages().len());
    println!("   - Word count: {:?}", state.get_typed::<usize>("word_count"));

    Ok(())
}
