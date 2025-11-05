//! REST API handlers
//!
//! HTTP endpoints for workflow management

use axum::{
    extract::{Path, State as AxumState},
    http::StatusCode,
    Json,
};
use conductor_builder::{GraphBuilder, WorkflowManager};
use conductor_primitives::State;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Shared application state
pub type AppState = Arc<Mutex<WorkflowManager>>;

/// Request to create a workflow session
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub session_id: String,
    pub template_name: String,
}

/// Response with session ID
#[derive(Debug, Serialize)]
pub struct CreateSessionResponse {
    pub session_id: String,
    pub template_name: String,
}

/// Request to execute a workflow
#[derive(Debug, Deserialize)]
pub struct ExecuteWorkflowRequest {
    pub initial_state: serde_json::Value,
}

/// Response from workflow execution
#[derive(Debug, Serialize)]
pub struct ExecuteWorkflowResponse {
    pub final_state: serde_json::Value,
    pub success: bool,
}

/// Template info
#[derive(Debug, Serialize)]
pub struct TemplateInfo {
    pub name: String,
    pub description: String,
}

/// Session info
#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub template_name: String,
    pub execution_count: usize,
}

/// List all available templates
///
/// GET /templates
pub async fn list_templates(
    AxumState(manager): AxumState<AppState>,
) -> Result<Json<Vec<TemplateInfo>>, StatusCode> {
    let manager = manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let templates: Vec<TemplateInfo> = manager
        .list_templates()
        .into_iter()
        .map(|(name, description)| TemplateInfo { name, description })
        .collect();

    Ok(Json(templates))
}

/// Create a new workflow session
///
/// POST /sessions
pub async fn create_session(
    AxumState(manager): AxumState<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<CreateSessionResponse>, (StatusCode, String)> {
    let manager = manager.lock().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lock error: {}", e),
        )
    })?;

    let session_id = manager
        .create_session(&req.session_id, &req.template_name)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok(Json(CreateSessionResponse {
        session_id,
        template_name: req.template_name,
    }))
}

/// List all active sessions
///
/// GET /sessions
pub async fn list_sessions(
    AxumState(manager): AxumState<AppState>,
) -> Result<Json<Vec<SessionInfo>>, StatusCode> {
    let manager = manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let sessions: Vec<SessionInfo> = manager
        .list_sessions()
        .into_iter()
        .filter_map(|id| {
            manager.get_session(&id).map(|(template, count)| SessionInfo {
                session_id: id,
                template_name: template,
                execution_count: count,
            })
        })
        .collect();

    Ok(Json(sessions))
}

/// Get session info
///
/// GET /sessions/:id
pub async fn get_session(
    AxumState(manager): AxumState<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionInfo>, (StatusCode, String)> {
    let manager = manager.lock().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lock error: {}", e),
        )
    })?;

    let (template_name, execution_count) = manager
        .get_session(&session_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Session not found".to_string()))?;

    Ok(Json(SessionInfo {
        session_id,
        template_name,
        execution_count,
    }))
}

/// Execute a workflow in a session
///
/// POST /sessions/:id/execute
pub async fn execute_workflow(
    AxumState(manager): AxumState<AppState>,
    Path(session_id): Path<String>,
    Json(req): Json<ExecuteWorkflowRequest>,
) -> Result<Json<ExecuteWorkflowResponse>, (StatusCode, String)> {
    // Convert JSON to State
    let state: State = serde_json::from_value(req.initial_state)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid state: {}", e)))?;

    // Execute workflow
    let manager = manager.lock().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lock error: {}", e),
        )
    })?;

    let result = manager
        .execute(&session_id, state)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Convert State to JSON
    let final_state = serde_json::to_value(&result)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Serialization error: {}", e)))?;

    Ok(Json(ExecuteWorkflowResponse {
        final_state,
        success: true,
    }))
}

/// Delete a session
///
/// DELETE /sessions/:id
pub async fn delete_session(
    AxumState(manager): AxumState<AppState>,
    Path(session_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let manager = manager.lock().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lock error: {}", e),
        )
    })?;

    let deleted = manager.delete_session(&session_id);

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "Session not found".to_string()))
    }
}

/// Health check endpoint
///
/// GET /health
pub async fn health_check() -> &'static str {
    "OK"
}
