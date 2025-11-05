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

## Web UI Testing

### Manual Testing

#### Prerequisites
1. **Install Trunk** (WASM build tool):
   ```bash
   cargo install trunk
   ```

2. **Install wasm32 target**:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

#### Running the Full Stack

**Terminal 1 - Start API Server:**
```bash
cd crates/conductor-api
cargo run

# API will run on http://127.0.0.1:3000
```

**Terminal 2 - Start Web UI:**
```bash
cd crates/conductor-ui
trunk serve

# UI will run on http://127.0.0.1:8080
```

#### UI Test Checklist

**Templates Page** (`/`):
- [ ] Templates load and display correctly
- [ ] Template descriptions are visible
- [ ] "Create Session" button works
- [ ] Navigates to new session after creation

**Sessions Page** (`/sessions`):
- [ ] Active sessions are listed
- [ ] Session cards show correct information
- [ ] "Refresh" button updates the list
- [ ] Clicking a session navigates to detail page
- [ ] Empty state shows when no sessions exist

**Session Detail Page** (`/sessions/:id`):
- [ ] Session information displays correctly
- [ ] JSON input textarea accepts valid JSON
- [ ] Execute button triggers workflow
- [ ] Loading state shows during execution
- [ ] Results display in formatted JSON
- [ ] Error messages show for invalid input
- [ ] Delete button removes session and redirects

**Navigation**:
- [ ] Navigation links work correctly
- [ ] Back button preserves state
- [ ] Active route is highlighted

**Responsive Design**:
- [ ] Layout works on mobile (< 768px)
- [ ] Layout works on tablet (768px - 1024px)
- [ ] Layout works on desktop (> 1024px)

#### Browser Compatibility

Test on:
- [ ] Chrome/Edge 90+
- [ ] Firefox 88+
- [ ] Safari 14+
- [ ] Mobile Safari (iOS 14+)
- [ ] Chrome Android

### End-to-End Testing

#### Test Scenario 1: Counter Workflow

1. Open http://127.0.0.1:8080
2. Click "Create Session" on the Counter template
3. Enter initial state: `{"count": 0}`
4. Click "Execute"
5. Verify result: `{"count": 1}`
6. Execute again with: `{"count": 10}`
7. Verify result: `{"count": 11}`
8. Click "Delete" and verify redirect to sessions page

#### Test Scenario 2: Math Workflow

1. Navigate to Templates page
2. Create session from "Math" template
3. Enter state:
   ```json
   {
     "a": 5,
     "b": 3
   }
   ```
4. Execute workflow
5. Verify result contains:
   ```json
   {
     "a": 5,
     "b": 3,
     "sum": 8,
     "product": 15
   }
   ```

#### Test Scenario 3: Error Handling

1. Create any session
2. Enter invalid JSON: `{invalid json}`
3. Click "Execute"
4. Verify error message appears
5. Enter valid JSON: `{}`
6. Verify execution succeeds

### Build Testing

#### Development Build
```bash
cd crates/conductor-ui
trunk build

# Check dist/ directory
ls -lh dist/
```

#### Production Build
```bash
cd crates/conductor-ui
trunk build --release

# Verify bundle size
du -h dist/conductor-ui*.wasm
# Should be ~200-300 KB gzipped
```

### Performance Testing

#### Load Time
1. Open browser DevTools
2. Navigate to http://127.0.0.1:8080
3. Check Network tab
4. Verify:
   - WASM loads < 1s
   - First Contentful Paint < 500ms
   - Time to Interactive < 1.5s

#### Runtime Performance
1. Open Performance tab in DevTools
2. Record while executing workflows
3. Verify no long tasks (> 50ms)
4. Check memory usage stays stable

### Console Errors

Open browser console and verify:
- [ ] No JavaScript errors
- [ ] No CORS errors
- [ ] No 404s (missing assets)
- [ ] No WASM loading errors

### Network Testing

#### API Communication
1. Open Network tab
2. Execute workflow
3. Verify requests:
   - Correct HTTP methods (GET, POST, DELETE)
   - Proper request headers (Content-Type: application/json)
   - Response status codes (200, 201, 204)
   - CORS headers present

#### Offline Behavior
1. Open DevTools Network tab
2. Set to "Offline"
3. Try to execute workflow
4. Verify user-friendly error message

### Automated UI Testing (Future)

For automated testing, consider:

**wasm-bindgen-test** for unit tests:
```rust
#[wasm_bindgen_test]
fn test_api_client() {
    // Test API client functions
}
```

**web-sys + wasm-bindgen-test** for integration:
```rust
#[wasm_bindgen_test]
async fn test_template_page() {
    // Test component rendering
}
```

**Playwright/Selenium** for E2E:
```javascript
test('create session and execute', async ({ page }) => {
  await page.goto('http://127.0.0.1:8080');
  await page.click('text=Create Session');
  // ...
});
```

## Testing Summary

| Test Type | Command | Duration | Coverage |
|-----------|---------|----------|----------|
| Unit Tests | `cargo test` | ~0.5s | Core logic |
| Integration (Mock) | `cargo test --test integration_test` | ~1s | Business logic |
| Examples (Ollama) | `cargo run --example multi_agent_workflow` | ~30s | Real LLM |
| API Server | `cargo run --bin conductor-api` | Manual | REST endpoints |
| Web UI | `trunk serve` | Manual | User interface |

**Recommended Testing Workflow:**

1. **Development**: Run `cargo test` frequently
2. **Before commit**: Run `cargo test --verbose`
3. **Before PR**: Test API + UI manually
4. **With Ollama**: Run examples to verify real workflows
5. **CI/CD**: Run `cargo test` (mock tests only)
