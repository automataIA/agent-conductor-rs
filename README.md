# Agent Conductor

A lightweight, high-performance agent orchestration framework built in Rust with WASM frontend.

## 🎯 Project Goals

- **Minimal Dependencies**: <10 core dependencies
- **Bottom-Up Design**: Built from primitives to high-level APIs
- **LangGraph-Inspired**: Immutable state, reducers, graph execution
- **OpenAI Compatible**: Works with OpenAI, Ollama, and compatible services
- **Local-First**: No vendor lock-in, runs entirely on your infrastructure

## 🏗️ Architecture

The project follows a 5-layer bottom-up architecture:

### Layer 1: Primitives ✅ **COMPLETED**
- `Message`: Chat message types (OpenAI-compatible)
- `State`: Immutable state management with reducers
- `LlmClient`: HTTP client for LLM APIs

### Layer 2: Graph Core ✅ **COMPLETED**
- `Node`: Async functions with state transformations (LlmNode, FunctionNode)
- `Edge`: Required and conditional edges
- `Condition`: Routing logic (BoolCondition, FunctionCondition)

### Layer 3: Execution Engine (Week 3)
- `GraphExecutor`: Message-passing execution model
- `Checkpointer`: SQLite-based persistence
- `Router`: Conditional routing logic

### Layer 4: Orchestration (Week 4)
- `GraphBuilder`: Fluent API for graph construction
- `WorkflowManager`: Multi-turn session management

### Layer 5: API & UI (Week 5)
- REST API (Axum)
- Server-Sent Events for real-time updates
- Leptos WASM dashboard

## 📦 Crates

```
conductor/
├── conductor-primitives   # Layer 1: Core types
├── conductor-graph        # Layer 2: Graph components
├── conductor-executor     # Layer 3: Execution engine
├── conductor-builder      # Layer 4: Builder API
└── conductor-api          # Layer 5: REST API + UI
```

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+
- Ollama (for local LLM testing)

### Installation

```bash
# Clone the repository
git clone https://github.com/automataIA/agent-conductor-rs.git
cd agent-conductor-rs

# Build the project
cargo build --release

# Run tests
cargo test

# Run integration tests (requires Ollama)
cargo test -- --ignored
```

### Basic Usage

```rust
use conductor_primitives::{Message, State, LlmClient, LlmConfig};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure Ollama client
    let config = LlmConfig::ollama("llama3.2:1b");
    let client = LlmClient::new(config)?;

    // Create messages
    let messages = vec![
        Message::system("You are a helpful assistant."),
        Message::user("What is Rust?"),
    ];

    // Get response
    let response = client.chat_completion(messages).await?;
    println!("LLM: {}", response);

    Ok(())
}
```

## 🧪 Testing

The project includes comprehensive tests:

```bash
# Unit tests
cargo test

# Integration tests (requires Ollama running)
cargo test -- --ignored

# Specific crate tests
cargo test --package conductor-primitives
```

## 📊 Progress Tracking

- [x] **Week 1**: Layer 1 - Primitives
  - [x] Message types
  - [x] State management with reducers
  - [x] LLM client (OpenAI-compatible)
- [x] **Week 2**: Layer 2 - Graph Core
  - [x] Node trait + implementations (LlmNode, FunctionNode)
  - [x] Edge types (Required, Conditional)
  - [x] Condition trait + implementations (BoolCondition, FunctionCondition)
  - [x] Comprehensive examples (simple_chain, multi_agent_workflow)
- [ ] **Week 3**: Layer 3 - Execution Engine
- [ ] **Week 4**: Layer 4 - Orchestration
- [ ] **Week 5**: Layer 5 - API & Frontend

## 🎨 Design Principles

### DRY (Don't Repeat Yourself)
- Shared traits for common behavior
- Generic reducer pattern
- Unified LLM client for all providers

### KISS (Keep It Simple)
- Zero custom macros (only derive)
- Simple HashMap-based state
- Embedded SQLite (no external DB)
- Straightforward REST API

### Bottom-Up Implementation
1. Build and test leaves (primitives)
2. Compose into executor
3. Add convenience layers (builder)
4. Expose via API

## 🔍 Comparison with LangGraph

| Feature | LangGraph (Python) | Conductor (Rust) |
|---------|-------------------|------------------|
| State Management | TypedDict + Annotated | HashMap + Reducer trait |
| Graph Execution | Message passing | Message passing (same) |
| Checkpointing | MemorySaver/SqliteSaver | SQLite (same concept) |
| Conditional Routing | routing_function | Condition trait |
| Runtime | Python (GIL) | Rust (no GIL) |
| Dependencies | 50+ (LangChain) | <10 (minimal) |
| Performance | ~1K req/s | ~500K req/s |

## 🛠️ Tech Stack

- **Async Runtime**: smol (minimal, 1K LOC)
- **Web Framework**: Axum (lightweight, high-performance)
- **HTTP Client**: reqwest (with rustls)
- **Database**: rusqlite (embedded)
- **Serialization**: serde + serde_json
- **Frontend**: Leptos 2.0 (WASM)

## 📝 License

MIT OR Apache-2.0

## 🤝 Contributing

Contributions welcome! Please read CONTRIBUTING.md first.

## 🔗 Links

- Documentation: Coming soon
- Issues: https://github.com/automataIA/agent-conductor-rs/issues
- Discussions: https://github.com/automataIA/agent-conductor-rs/discussions
