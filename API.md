# Conductor API Documentation

REST API and Server-Sent Events for Agent Conductor

## Base URL

```
http://127.0.0.1:3000
```

## Endpoints

### Health Check

**GET /health**

Check if the API server is running.

```bash
curl http://127.0.0.1:3000/health
```

Response: `OK`

---

### Templates

#### List Templates

**GET /templates**

Get all available workflow templates.

```bash
curl http://127.0.0.1:3000/templates
```

Response:
```json
[
  {
    "name": "counter",
    "description": "Increments a counter"
  },
  {
    "name": "echo",
    "description": "Echoes the input"
  },
  {
    "name": "math",
    "description": "Basic math operations"
  }
]
```

---

### Sessions

#### Create Session

**POST /sessions**

Create a new workflow session from a template.

```bash
curl -X POST http://127.0.0.1:3000/sessions \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "user-123",
    "template_name": "counter"
  }'
```

Response:
```json
{
  "session_id": "user-123",
  "template_name": "counter"
}
```

#### List Sessions

**GET /sessions**

Get all active sessions.

```bash
curl http://127.0.0.1:3000/sessions
```

Response:
```json
[
  {
    "session_id": "user-123",
    "template_name": "counter",
    "execution_count": 0
  }
]
```

#### Get Session

**GET /sessions/:id**

Get information about a specific session.

```bash
curl http://127.0.0.1:3000/sessions/user-123
```

Response:
```json
{
  "session_id": "user-123",
  "template_name": "counter",
  "execution_count": 2
}
```

#### Execute Workflow

**POST /sessions/:id/execute**

Execute a workflow in a session.

```bash
curl -X POST http://127.0.0.1:3000/sessions/user-123/execute \
  -H "Content-Type: application/json" \
  -d '{
    "initial_state": {
      "data": {
        "count": 5
      }
    }
  }'
```

Response:
```json
{
  "final_state": {
    "data": {
      "count": 6
    }
  },
  "success": true
}
```

#### Delete Session

**DELETE /sessions/:id**

Delete a session.

```bash
curl -X DELETE http://127.0.0.1:3000/sessions/user-123
```

Response: `204 No Content`

---

### Server-Sent Events (SSE)

#### Stream Workflow Events

**GET /workflows/:id/stream**

Subscribe to real-time workflow execution events.

```bash
curl -N http://127.0.0.1:3000/workflows/wf-123/stream
```

Events:
```
event: workflow_started
data: {"workflow_id":"wf-123"}

event: node_started
data: {"node":"start","step":0}

event: node_completed
data: {"node":"start","step":0}

event: workflow_completed
data: {"workflow_id":"wf-123","total_steps":2}
```

#### Stream Session Events

**GET /sessions/:id/stream**

Subscribe to session events.

```bash
curl -N http://127.0.0.1:3000/sessions/user-123/stream
```

#### Heartbeat

**GET /heartbeat**

Test SSE connection with a heartbeat stream.

```bash
curl -N http://127.0.0.1:3000/heartbeat
```

---

## Examples

### Complete Workflow

```bash
# 1. Check health
curl http://127.0.0.1:3000/health

# 2. List available templates
curl http://127.0.0.1:3000/templates

# 3. Create a session
curl -X POST http://127.0.0.1:3000/sessions \
  -H "Content-Type: application/json" \
  -d '{"session_id":"demo","template_name":"counter"}'

# 4. Execute workflow (first time)
curl -X POST http://127.0.0.1:3000/sessions/demo/execute \
  -H "Content-Type: application/json" \
  -d '{"initial_state":{"data":{"count":0}}}'

# 5. Execute workflow (second time)
curl -X POST http://127.0.0.1:3000/sessions/demo/execute \
  -H "Content-Type: application/json" \
  -d '{"initial_state":{"data":{"count":10}}}'

# 6. Check session info
curl http://127.0.0.1:3000/sessions/demo

# 7. Delete session
curl -X DELETE http://127.0.0.1:3000/sessions/demo
```

### Math Template Example

```bash
# Create session with math template
curl -X POST http://127.0.0.1:3000/sessions \
  -H "Content-Type: application/json" \
  -d '{"session_id":"math-demo","template_name":"math"}'

# Execute calculation
curl -X POST http://127.0.0.1:3000/sessions/math-demo/execute \
  -H "Content-Type: application/json" \
  -d '{
    "initial_state": {
      "data": {
        "a": 5,
        "b": 3
      }
    }
  }'

# Response will include sum=8 and product=15
```

---

## Running the Server

```bash
# From project root
cargo run --bin conductor-api

# With debug logging
RUST_LOG=debug cargo run --bin conductor-api
```

The server will start on `http://127.0.0.1:3000`

---

## Error Responses

All errors return appropriate HTTP status codes:

- `400 Bad Request` - Invalid request body or parameters
- `404 Not Found` - Resource not found (session, template)
- `500 Internal Server Error` - Server error

Example error response:
```json
{
  "error": "Session not found"
}
```

Or plain text error message.

---

## CORS

CORS is enabled for all origins in development. Configure appropriately for production.

---

## Next Steps

- Integrate with Leptos frontend
- Add authentication
- Add rate limiting
- Add request validation middleware
- Implement real-time SSE with executor events
- Add metrics and monitoring
