//! # Conductor API Server
//!
//! REST API and SSE endpoints for Agent Conductor
//!
//! ## Endpoints
//!
//! ### Templates
//! - GET /templates - List all workflow templates
//!
//! ### Sessions
//! - POST /sessions - Create a new session
//! - GET /sessions - List all sessions
//! - GET /sessions/:id - Get session info
//! - POST /sessions/:id/execute - Execute workflow in session
//! - DELETE /sessions/:id - Delete a session
//!
//! ### SSE
//! - GET /workflows/:id/stream - Stream workflow events
//! - GET /sessions/:id/stream - Stream session events
//! - GET /heartbeat - Heartbeat for testing
//!
//! ### Health
//! - GET /health - Health check

mod handlers;
mod sse;

use axum::{
    routing::{delete, get, post},
    Router,
};
use conductor_builder::{GraphBuilder, WorkflowManager};
use conductor_graph::FunctionNode;
use serde_json::json;
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "conductor_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🚀 Starting Conductor API Server");

    // Create workflow manager and register demo templates
    let manager = create_demo_workflow_manager();
    let shared_manager = Arc::new(Mutex::new(manager));

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(handlers::health_check))
        // Templates
        .route("/templates", get(handlers::list_templates))
        // Sessions
        .route("/sessions", post(handlers::create_session))
        .route("/sessions", get(handlers::list_sessions))
        .route("/sessions/:id", get(handlers::get_session))
        .route("/sessions/:id/execute", post(handlers::execute_workflow))
        .route("/sessions/:id", delete(handlers::delete_session))
        // SSE
        .route("/workflows/:id/stream", get(sse::stream_workflow))
        .route("/sessions/:id/stream", get(sse::stream_session))
        .route("/heartbeat", get(sse::heartbeat))
        // Add CORS
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        // Shared state
        .with_state(shared_manager);

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::info!("✅ Server listening on http://127.0.0.1:3000");
    tracing::info!("");
    tracing::info!("📋 Available endpoints:");
    tracing::info!("  - GET  /health");
    tracing::info!("  - GET  /templates");
    tracing::info!("  - POST /sessions");
    tracing::info!("  - GET  /sessions");
    tracing::info!("  - GET  /sessions/:id");
    tracing::info!("  - POST /sessions/:id/execute");
    tracing::info!("  - GET  /workflows/:id/stream");
    tracing::info!("  - GET  /heartbeat");
    tracing::info!("");

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

/// Create demo workflow manager with sample templates
fn create_demo_workflow_manager() -> WorkflowManager {
    let mut manager = WorkflowManager::new();

    // Template 1: Simple counter
    manager.register_template("counter", "Increments a counter", || {
        let node = FunctionNode::new("increment", |state| {
            Box::pin(async move {
                let count: i32 = state.get_typed("count").unwrap_or(0);
                Ok(state.set("count", json!(count + 1)))
            })
        });

        GraphBuilder::new()
            .add_node(Box::new(node))
            .add_edge("start", "increment")
            .add_edge("increment", "end")
            .build(":memory:")
    });

    // Template 2: Echo
    manager.register_template("echo", "Echoes the input", || {
        let node = FunctionNode::new("echo", |state| {
            Box::pin(async move {
                let input: String = state.get_typed("input").unwrap_or_default();
                Ok(state.set("output", json!(format!("Echo: {}", input))))
            })
        });

        GraphBuilder::new()
            .add_node(Box::new(node))
            .add_edge("start", "echo")
            .add_edge("echo", "end")
            .build(":memory:")
    });

    // Template 3: Math processor
    manager.register_template("math", "Basic math operations", || {
        let node = FunctionNode::new("calculate", |state| {
            Box::pin(async move {
                let a: i32 = state.get_typed("a").unwrap_or(0);
                let b: i32 = state.get_typed("b").unwrap_or(0);
                let sum = a + b;
                let product = a * b;

                Ok(state
                    .set("sum", json!(sum))
                    .set("product", json!(product)))
            })
        });

        GraphBuilder::new()
            .add_node(Box::new(node))
            .add_edge("start", "calculate")
            .add_edge("calculate", "end")
            .build(":memory:")
    });

    tracing::info!("✅ Registered {} demo templates", manager.template_count());

    manager
}
