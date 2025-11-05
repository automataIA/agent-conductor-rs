//! Server-Sent Events for real-time workflow updates

use axum::{
    extract::Path,
    response::{
        sse::{Event, KeepAlive},
        Sse,
    },
};
use futures::stream::{self, Stream};
use std::convert::Infallible;
use std::time::Duration;

/// Stream workflow execution events
///
/// GET /workflows/:id/stream
pub async fn stream_workflow(
    Path(workflow_id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // In a real implementation, this would:
    // 1. Subscribe to executor events
    // 2. Stream events as they occur
    // 3. Complete when workflow finishes
    //
    // For now, we demonstrate the structure with a mock stream

    let stream = stream::iter(vec![
        Event::default()
            .event("workflow_started")
            .data(format!("{{\"workflow_id\":\"{}\"}}", workflow_id)),
        Event::default()
            .event("node_started")
            .data("{\"node\":\"start\",\"step\":0}"),
        Event::default()
            .event("node_completed")
            .data("{\"node\":\"start\",\"step\":0}"),
        Event::default()
            .event("node_started")
            .data("{\"node\":\"process\",\"step\":1}"),
        Event::default()
            .event("node_completed")
            .data("{\"node\":\"process\",\"step\":1}"),
        Event::default()
            .event("workflow_completed")
            .data(format!("{{\"workflow_id\":\"{}\",\"total_steps\":2}}", workflow_id)),
    ])
    .map(Ok);

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Stream session events
///
/// GET /sessions/:id/stream
pub async fn stream_session(
    Path(session_id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Mock stream for session events
    let stream = stream::iter(vec![
        Event::default()
            .event("session_active")
            .data(format!("{{\"session_id\":\"{}\"}}", session_id)),
        Event::default()
            .event("execution_started")
            .data("{\"execution_number\":1}"),
    ])
    .map(Ok);

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive"),
    )
}

/// Heartbeat stream for connection testing
///
/// GET /heartbeat
pub async fn heartbeat() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::iter((0..10).map(|i| {
        Event::default()
            .event("ping")
            .data(format!("{{\"count\":{}}}", i))
    }))
    .map(Ok);

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(1)))
}
