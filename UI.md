# Conductor Web UI Documentation

WASM-based frontend for Agent Conductor built with Leptos 0.7

## Overview

The Conductor UI is a reactive, single-page application (SPA) built entirely in Rust and compiled to WebAssembly. It provides a user-friendly interface for managing workflow templates, sessions, and executions.

## Architecture

### Technology Stack

- **Framework**: Leptos 0.7 (Client-Side Rendering)
- **Language**: Rust (100% - no JavaScript)
- **Build Tool**: Trunk (WASM bundler)
- **HTTP Client**: gloo-net
- **Routing**: leptos_router
- **Target**: WebAssembly (wasm32-unknown-unknown)

### Crate Structure

```
crates/conductor-ui/
├── Cargo.toml           # Dependencies and crate config
├── Trunk.toml           # Trunk build configuration
├── index.html           # HTML template with embedded CSS
└── src/
    ├── main.rs          # WASM entry point
    ├── lib.rs           # App component and routing
    ├── api.rs           # REST API client
    └── components.rs    # UI components
```

## Installation

### Prerequisites

```bash
# Install Trunk (WASM bundler)
cargo install trunk

# Install wasm32 target
rustup target add wasm32-unknown-unknown
```

### Development Setup

```bash
# Clone the repository
git clone https://github.com/automataIA/agent-conductor-rs
cd agent-conductor-rs/crates/conductor-ui

# Run development server
trunk serve

# UI will be available at http://127.0.0.1:8080
```

### Building for Production

```bash
# Build optimized WASM bundle
trunk build --release

# Output will be in dist/
# Deploy dist/ directory to your web server
```

## Pages

### 1. Templates Page (`/`)

**Purpose**: Browse and select workflow templates

**Features**:
- Displays all available workflow templates
- Shows template name and description
- "Create Session" button for each template
- Automatically navigates to new session after creation

**API Calls**:
- `GET /templates` - Fetch available templates
- `POST /sessions` - Create new session from template

**Example Templates**:
- **Counter**: Simple counter incrementer
- **Echo**: Echoes input state
- **Math**: Basic arithmetic operations

### 2. Sessions Page (`/sessions`)

**Purpose**: View and manage active workflow sessions

**Features**:
- Lists all active sessions
- Shows session ID, template name, and execution count
- Click to view session details
- Refresh button to reload sessions
- Empty state when no sessions exist

**API Calls**:
- `GET /sessions` - Fetch all sessions

**Session Card Display**:
```
┌─────────────────────────────┐
│ Session ID          [Badge] │
│ Executions: N               │
└─────────────────────────────┘
```

### 3. Session Detail Page (`/sessions/:id`)

**Purpose**: Execute workflows and view results

**Features**:
- Session information header
- JSON state input (textarea)
- Execute button with loading state
- Formatted JSON result display
- Delete session functionality
- Error handling and display

**API Calls**:
- `GET /sessions/:id` - Fetch session details
- `POST /sessions/:id/execute` - Execute workflow
- `DELETE /sessions/:id` - Delete session

**Workflow**:
1. Enter initial state as JSON (optional)
2. Click "Execute" button
3. View formatted result
4. Repeat as needed

**Example Input**:
```json
{
  "count": 5,
  "message": "hello"
}
```

## Components

### App Component (`lib.rs`)

Main application component with routing and layout.

```rust
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <nav class="navbar">...</nav>
            <main class="container">
                <Routes>
                    <Route path="/" view=TemplatesPage/>
                    <Route path="/sessions" view=SessionsPage/>
                    <Route path="/sessions/:id" view=SessionDetailPage/>
                </Routes>
            </main>
            <footer>...</footer>
        </Router>
    }
}
```

### API Client (`api.rs`)

HTTP client for backend communication.

**Functions**:
```rust
pub async fn list_templates() -> Result<Vec<TemplateInfo>, String>
pub async fn create_session(template_name: String) -> Result<CreateSessionResponse, String>
pub async fn list_sessions() -> Result<Vec<SessionInfo>, String>
pub async fn get_session(session_id: &str) -> Result<SessionInfo, String>
pub async fn execute_workflow(session_id: &str, state: HashMap<String, Value>) -> Result<ExecuteWorkflowResponse, String>
pub async fn delete_session(session_id: &str) -> Result<(), String>
```

**Configuration**:
```rust
const API_BASE: &str = "http://127.0.0.1:3000";
```

To use a different API endpoint, update this constant before building.

### UI Components (`components.rs`)

Three main page components:

1. **TemplatesPage**: Template browser
2. **SessionsPage**: Session manager
3. **SessionDetailPage**: Workflow executor

## Styling

The UI uses embedded CSS in `index.html` with a modern, clean design:

### Design System

**Colors**:
- Primary: `#3b82f6` (blue)
- Secondary: `#64748b` (slate)
- Success: `#10b981` (green)
- Danger: `#ef4444` (red)
- Background: `#f8fafc` (light gray)
- Surface: `#ffffff` (white)

**Typography**:
- System font stack
- Font sizes: 0.75rem - 2rem
- Line height: 1.6

**Layout**:
- Max width: 1200px
- Responsive grid (auto-fill, minmax 300px)
- Mobile-first design

**Components**:
- Cards with hover effects
- Buttons with 3 variants (primary, secondary, danger)
- Badges for status indicators
- Form inputs with focus states

### Responsive Design

```css
@media (max-width: 768px) {
    /* Single column layout */
    /* Stacked navigation */
    /* Full-width cards */
}
```

## State Management

Leptos provides reactive primitives for state management:

### Signals

```rust
let (input_value, set_input_value) = create_signal(String::new());
let (result, set_result) = create_signal(Option::None);
let (executing, set_executing) = create_signal(false);
```

### Resources

```rust
let templates = create_resource(
    || (),
    |_| async { api::list_templates().await }
);
```

### Actions

```rust
let create_session = create_action(|template_name: &String| {
    let template_name = template_name.clone();
    async move {
        api::create_session(template_name).await
    }
});
```

## Error Handling

All API errors are caught and displayed to the user:

```rust
match api::execute_workflow(&id, state).await {
    Ok(response) => {
        set_result.set(Some(response.result));
        set_error.set(None);
    }
    Err(e) => {
        set_error.set(Some(e));
    }
}
```

**Error Display**:
- Red error boxes for API errors
- Inline validation errors
- User-friendly error messages

## Development

### Running the Dev Server

```bash
cd crates/conductor-ui
trunk serve

# With auto-reload on changes
trunk serve --open
```

### Hot Reload

Trunk automatically reloads the browser when source files change.

### Debugging

```rust
// Use leptos logging
use leptos::logging;

logging::log!("Debug message");
logging::error!("Error message");
```

Browser console will show Rust panic messages and logs.

### Building

```bash
# Debug build (faster, larger)
trunk build

# Release build (optimized)
trunk build --release

# With specific features
trunk build --release --features "feature_name"
```

## API Integration

### Backend Requirements

The UI expects the API server to be running on `http://127.0.0.1:3000`.

**Required Endpoints**:
- `GET /templates`
- `GET /sessions`
- `POST /sessions`
- `GET /sessions/:id`
- `POST /sessions/:id/execute`
- `DELETE /sessions/:id`

**CORS Configuration**:

The backend must allow CORS requests from `http://127.0.0.1:8080`:

```rust
use tower_http::cors::{CorsLayer, Any};

let app = Router::new()
    .layer(CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any))
    // ... routes
```

### Request/Response Types

**CreateSessionRequest**:
```json
{
  "template_name": "counter"
}
```

**ExecuteWorkflowRequest**:
```json
{
  "state": {
    "key": "value"
  }
}
```

**ExecuteWorkflowResponse**:
```json
{
  "result": {
    "key": "new_value"
  }
}
```

## Deployment

### Static Hosting

The UI is a static site and can be hosted anywhere:

1. **Build for production**:
   ```bash
   trunk build --release
   ```

2. **Deploy `dist/` directory** to:
   - Netlify
   - Vercel
   - GitHub Pages
   - AWS S3 + CloudFront
   - Any static web server

### Environment Configuration

To use a different API endpoint in production:

1. **Option 1**: Update `API_BASE` in `src/api.rs` before building

2. **Option 2**: Use environment variables with trunk:
   ```rust
   const API_BASE: &str = env!("API_URL");
   ```

   Then build with:
   ```bash
   API_URL=https://api.example.com trunk build --release
   ```

### Nginx Configuration

```nginx
server {
    listen 80;
    server_name conductor.example.com;
    root /var/www/conductor-ui/dist;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    # WASM MIME type
    types {
        application/wasm wasm;
    }
}
```

## Browser Support

- **Chrome/Edge**: 90+ ✅
- **Firefox**: 88+ ✅
- **Safari**: 14+ ✅
- **Mobile**: iOS Safari 14+, Chrome Android 90+ ✅

**Requirements**:
- WebAssembly support
- ES6 modules
- Async/await

## Performance

### Bundle Size

- **WASM (release)**: ~200-300 KB (gzipped)
- **JS Glue**: ~20-30 KB
- **HTML/CSS**: ~5 KB

### Optimization

Trunk automatically applies optimizations:
- wasm-opt (size optimization)
- Asset minification
- Tree shaking

### Load Time

- **First Load**: 500ms - 1s
- **Subsequent Loads**: <100ms (cached)

## Troubleshooting

### UI Not Loading

1. Check if API server is running on port 3000
2. Check browser console for errors
3. Verify CORS is enabled on backend

### API Errors

1. Open browser DevTools Network tab
2. Check request/response details
3. Verify API endpoint URLs
4. Check CORS headers

### Build Errors

```bash
# Clean and rebuild
rm -rf dist/
trunk clean
trunk build
```

### WASM Not Loading

1. Check MIME type configuration
2. Verify wasm32 target is installed:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

## Future Enhancements

- [ ] Real-time workflow execution updates (SSE integration)
- [ ] Dark mode toggle
- [ ] Workflow visualization (graph rendering)
- [ ] Session history and logs
- [ ] Export/import workflows
- [ ] User authentication
- [ ] Collaborative features
- [ ] Workflow templates editor

## Examples

### Creating a Session and Executing

1. Navigate to http://127.0.0.1:8080
2. Click "Create Session" on the "Counter" template
3. Enter initial state:
   ```json
   {"count": 0}
   ```
4. Click "Execute"
5. View result:
   ```json
   {"count": 1}
   ```

### Working with Math Template

1. Create session from "Math" template
2. Enter state:
   ```json
   {
     "a": 10,
     "b": 5
   }
   ```
3. Execute to get:
   ```json
   {
     "a": 10,
     "b": 5,
     "sum": 15,
     "product": 50
   }
   ```

## Contributing

When contributing to the UI:

1. Follow Leptos conventions
2. Use reactive signals for state
3. Keep components focused and reusable
4. Add error handling for all API calls
5. Test on multiple browsers
6. Update this documentation

## Resources

- [Leptos Book](https://leptos-rs.github.io/leptos/)
- [Trunk Documentation](https://trunkrs.dev/)
- [WebAssembly](https://webassembly.org/)
- [API Documentation](API.md)
