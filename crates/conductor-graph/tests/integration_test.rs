//! Integration tests for conductor-graph
//!
//! Tests nodes, edges, and conditions without requiring Ollama

use conductor_graph::{BoolCondition, Edge, FunctionCondition, FunctionNode, Node};
use conductor_primitives::{Message, State};
use serde_json::json;

#[tokio::test]
async fn test_function_node_chain() {
    // Test 1: Chain of FunctionNodes
    println!("🧪 Test 1: Function node chain");

    // Node 1: Add user input
    let input_node = FunctionNode::new("input", |state| {
        Box::pin(async move { Ok(state.set("input", json!("Hello, world!"))) })
    });

    // Node 2: Process (uppercase)
    let process_node = FunctionNode::new("process", |state| {
        Box::pin(async move {
            let input: String = state.get_typed("input").unwrap_or_default();
            let processed = input.to_uppercase();
            Ok(state.set("processed", json!(processed)))
        })
    });

    // Node 3: Count words
    let count_node = FunctionNode::new("count", |state| {
        Box::pin(async move {
            let processed: String = state.get_typed("processed").unwrap_or_default();
            let count = processed.split_whitespace().count();
            Ok(state.set("word_count", json!(count)))
        })
    });

    // Execute chain
    let state = State::new();
    let state = input_node.invoke(state).await.unwrap();
    let state = process_node.invoke(state).await.unwrap();
    let state = count_node.invoke(state).await.unwrap();

    assert_eq!(state.get_typed::<String>("input"), Some("Hello, world!".to_string()));
    assert_eq!(
        state.get_typed::<String>("processed"),
        Some("HELLO, WORLD!".to_string())
    );
    assert_eq!(state.get_typed::<usize>("word_count"), Some(2));

    println!("✅ Test 1 passed: Function node chain works correctly");
}

#[tokio::test]
async fn test_conditional_routing() {
    // Test 2: Conditional edge routing
    println!("🧪 Test 2: Conditional routing");

    let condition = BoolCondition::new("is_premium", "premium_handler", "free_handler");
    let edge = Edge::conditional("check", condition);

    // Test premium user
    let state1 = State::new().set("is_premium", json!(true));
    let next1 = edge.next_node(&state1).await.unwrap();
    assert_eq!(next1, "premium_handler");

    // Test free user
    let state2 = State::new().set("is_premium", json!(false));
    let next2 = edge.next_node(&state2).await.unwrap();
    assert_eq!(next2, "free_handler");

    println!("✅ Test 2 passed: Conditional routing works");
}

#[tokio::test]
async fn test_function_condition_complex() {
    // Test 3: Complex function-based condition
    println!("🧪 Test 3: Complex function condition");

    let condition = FunctionCondition::new(|state| {
        Box::pin(async move {
            let score: i32 = state.get_typed("score").unwrap_or(0);
            let tier = if score >= 90 {
                "excellent"
            } else if score >= 70 {
                "good"
            } else if score >= 50 {
                "average"
            } else {
                "poor"
            };
            Ok(tier.to_string())
        })
    });

    let edge = Edge::conditional("scorer", condition);

    // Test different scores
    let tests = vec![
        (95, "excellent"),
        (75, "good"),
        (55, "average"),
        (30, "poor"),
    ];

    for (score, expected) in tests {
        let state = State::new().set("score", json!(score));
        let result = edge.next_node(&state).await.unwrap();
        assert_eq!(result, expected, "Score {} should route to {}", score, expected);
    }

    println!("✅ Test 3 passed: Complex function conditions work");
}

#[tokio::test]
async fn test_multi_agent_simulation() {
    // Test 4: Simulated multi-agent workflow
    println!("🧪 Test 4: Multi-agent workflow simulation");

    use conductor_primitives::AppendReducer;

    // Agent 1: Analyzer
    let analyzer = FunctionNode::new("analyzer", |state| {
        Box::pin(async move {
            let messages = state.get_messages();
            let user_msg = messages.last().unwrap().content.clone();

            let analysis = format!("Analysis: '{}' has {} characters", user_msg, user_msg.len());
            let response = Message::assistant(&analysis);

            let reducer = AppendReducer;
            Ok(state.update("messages", json!([response]), &reducer))
        })
    });

    // Agent 2: Quality checker
    let quality_checker = FunctionNode::new("quality_checker", |state| {
        Box::pin(async move {
            let messages = state.get_messages();
            let analysis = &messages.last().unwrap().content;

            let quality_score = if analysis.len() > 20 { 8 } else { 4 };
            Ok(state.set("quality_score", json!(quality_score)))
        })
    });

    // Agent 3: Reporter
    let reporter = FunctionNode::new("reporter", |state| {
        Box::pin(async move {
            let score: i32 = state.get_typed("quality_score").unwrap_or(0);
            let report = format!("Quality Report: Score {}/10", score);
            let response = Message::assistant(&report);

            let reducer = AppendReducer;
            Ok(state.update("messages", json!([response]), &reducer))
        })
    });

    // Execute workflow
    let state = State::new().set("messages", json!([Message::user("Test input message")]));

    let state = analyzer.invoke(state).await.unwrap();
    assert_eq!(state.get_messages().len(), 2); // user + analyzer

    let state = quality_checker.invoke(state).await.unwrap();
    assert!(state.get_typed::<i32>("quality_score").is_some());

    let state = reporter.invoke(state).await.unwrap();
    assert_eq!(state.get_messages().len(), 3); // user + analyzer + reporter

    println!("✅ Test 4 passed: Multi-agent simulation works");
}

#[tokio::test]
async fn test_conditional_loop_simulation() {
    // Test 5: Simulate a loop with conditional exit
    println!("🧪 Test 5: Conditional loop simulation");

    let counter = FunctionNode::new("counter", |state| {
        Box::pin(async move {
            let count: i32 = state.get_typed("count").unwrap_or(0);
            Ok(state.set("count", json!(count + 1)))
        })
    });

    let exit_condition = FunctionCondition::new(|state| {
        Box::pin(async move {
            let count: i32 = state.get_typed("count").unwrap_or(0);
            if count >= 5 {
                Ok("exit".to_string())
            } else {
                Ok("continue".to_string())
            }
        })
    });

    let edge = Edge::conditional("counter", exit_condition);

    // Simulate loop
    let mut state = State::new();
    let mut iteration = 0;
    let max_iterations = 10;

    while iteration < max_iterations {
        state = counter.invoke(state).await.unwrap();
        let next_node = edge.next_node(&state).await.unwrap();

        if next_node == "exit" {
            break;
        }
        iteration += 1;
    }

    let final_count: i32 = state.get_typed("count").unwrap();
    assert_eq!(final_count, 5);
    assert_eq!(iteration, 4); // Ran 5 times (0-4 iterations + initial)

    println!("✅ Test 5 passed: Conditional loop exits correctly");
}

#[tokio::test]
async fn test_required_edge_chain() {
    // Test 6: Required edge chain
    println!("🧪 Test 6: Required edge chain");

    let edges = vec![
        Edge::required("start", "node_a"),
        Edge::required("node_a", "node_b"),
        Edge::required("node_b", "node_c"),
        Edge::required("node_c", "end"),
    ];

    // Verify edge structure
    assert_eq!(edges[0].from(), "start");
    assert_eq!(edges[0].to(), Some("node_a"));
    assert!(edges[0].is_required());

    // Simulate traversal
    let state = State::new();
    let mut current = "start";
    let mut path = vec!["start"];

    for edge in &edges {
        if edge.from() == current {
            current = edge.next_node(&state).await.unwrap().as_str();
            path.push(current);
        }
    }

    assert_eq!(path, vec!["start", "node_a", "node_b", "node_c", "end"]);

    println!("✅ Test 6 passed: Required edge chain traversal works");
}

#[tokio::test]
async fn test_node_name_consistency() {
    // Test 7: Node name consistency
    println!("🧪 Test 7: Node name consistency");

    let node1 = FunctionNode::new("test_node", |state| Box::pin(async move { Ok(state) }));

    assert_eq!(node1.name(), "test_node");

    println!("✅ Test 7 passed: Node names are consistent");
}
