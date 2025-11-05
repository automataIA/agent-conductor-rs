# Testing Guide

This document explains how to test the Agent Conductor framework.

## Test Types

### 1. Unit Tests (No External Dependencies)
Run all unit tests:
```bash
cargo test
```

Run tests for specific crate:
```bash
cargo test --package conductor-primitives
cargo test --package conductor-graph
```

### 2. Integration Tests (Mock LLM)
We provide comprehensive integration tests that work without Ollama:

```bash
# Run all integration tests
cargo test --test integration_test

# Run specific integration test
cargo test --test integration_test test_function_node_chain
```

These tests verify:
- ✅ State immutability and reducers
- ✅ Message serialization (OpenAI compatibility)
- ✅ Node execution chains
- ✅ Conditional routing logic
- ✅ Multi-agent workflow simulation
- ✅ Complex state operations

### 3. Examples with Real LLM (Requires Ollama)
To run examples with actual LLM calls:

#### Step 1: Install Ollama

**Linux:**
```bash
curl -fsSL https://ollama.com/install.sh | sh
```

**macOS:**
```bash
brew install ollama
```

**Windows:**
Download from https://ollama.com/download

#### Step 2: Start Ollama Server
```bash
ollama serve
```

The server will run on `http://localhost:11434`

#### Step 3: Download Required Model
```bash
# Download llama3.2:1b (recommended for testing - smallest model)
ollama pull llama3.2:1b

# Verify installation
ollama list
```

#### Step 4: Run Examples

**Primitives Example:**
```bash
cargo run --example simple_chat --package conductor-primitives
```

**Graph Examples:**
```bash
# Simple chain (Translator -> Summarizer -> Counter)
cargo run --example simple_chain --package conductor-graph

# Multi-agent workflow (Researcher -> Quality Check -> [Reviewer | Writer])
cargo run --example multi_agent_workflow --package conductor-graph
```

### 4. Ignored Tests (Requires Ollama)
Some unit tests are marked with `#[ignore]` because they require Ollama:

```bash
# Run only ignored tests (requires Ollama running)
cargo test -- --ignored

# Run ALL tests including ignored ones
cargo test -- --include-ignored
```

## Test Environment

### Without Ollama
- ✅ All unit tests pass
- ✅ All integration tests pass
- ✅ Test coverage: ~95% (all logic except actual LLM calls)

### With Ollama
- ✅ Everything above
- ✅ Real LLM integration tests
- ✅ Runnable examples with actual AI responses

## Test Structure

```
conductor/
├── crates/
│   ├── conductor-primitives/
│   │   ├── src/
│   │   │   ├── lib.rs           # Doc tests
│   │   │   ├── message.rs       # Unit tests
│   │   │   ├── state.rs         # Unit tests
│   │   │   └── llm_client.rs    # Unit tests + 1 ignored
│   │   ├── tests/
│   │   │   └── integration_test.rs  # 7 integration tests
│   │   └── examples/
│   │       └── simple_chat.rs       # Requires Ollama
│   │
│   └── conductor-graph/
│       ├── src/
│       │   ├── node.rs          # Unit tests + 1 ignored
│       │   └── edge.rs          # Unit tests
│       ├── tests/
│       │   └── integration_test.rs  # 7 integration tests
│       └── examples/
│           ├── simple_chain.rs          # Requires Ollama
│           └── multi_agent_workflow.rs  # Requires Ollama
```

## Quick Test Commands

```bash
# Fast: Run all tests without Ollama
cargo test

# Thorough: Run all tests (mock + integration)
cargo test --verbose

# Full: With Ollama running
ollama serve &
ollama pull llama3.2:1b
cargo test -- --include-ignored

# Examples: Test real workflows
cargo run --example multi_agent_workflow --package conductor-graph
```

## Expected Output

### Mock Tests
```
running 14 tests
test tests::test_append_reducer ... ok
test tests::test_function_condition ... ok
test tests::test_multi_agent_simulation ... ok
...
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured
```

### With Ollama
```
🚀 Agent Conductor - Multi-Agent Workflow Example

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📝 User Query: What is Rust programming language?
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🔄 Iteration 1 - Current Node: researcher
─────────────────────────────────────────────────

🔬 Researcher: Gathering information...
📄 Response: Rust is a systems programming language...
...
```

## Continuous Integration

For CI/CD pipelines without Ollama:

```yaml
# .github/workflows/test.yml
- name: Run tests
  run: cargo test --all-features
```

For local development with Ollama:

```bash
# Start Ollama in background
ollama serve > /dev/null 2>&1 &

# Run full test suite
cargo test -- --include-ignored
```

## Troubleshooting

### "connection refused" errors
- Ensure Ollama is running: `ollama serve`
- Check port 11434 is available: `lsof -i :11434`

### "model not found" errors
- Pull the model: `ollama pull llama3.2:1b`
- Verify: `ollama list`

### Network issues in CI
- Use mock tests only: `cargo test` (excludes `--ignored`)
- Or setup Ollama in CI with Docker: `docker run -d ollama/ollama`

## Performance

- **Unit tests**: ~0.5s
- **Integration tests (mock)**: ~1s
- **Examples with Ollama**: ~10-30s (depends on model speed)

## Coverage

Run with coverage tool:
```bash
cargo tarpaulin --out Html --output-dir coverage
```

Current coverage: **~95%** (excluding real LLM calls)
