//! API client for interacting with the Conductor backend

use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const API_BASE: &str = "http://127.0.0.1:3000";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub template_name: String,
    pub execution_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub template_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionResponse {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteWorkflowRequest {
    pub state: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteWorkflowResponse {
    pub result: HashMap<String, serde_json::Value>,
}

/// Fetch list of workflow templates
pub async fn list_templates() -> Result<Vec<TemplateInfo>, String> {
    let url = format!("{}/templates", API_BASE);

    Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

/// Create a new session from a template
pub async fn create_session(template_name: String) -> Result<CreateSessionResponse, String> {
    let url = format!("{}/sessions", API_BASE);
    let body = CreateSessionRequest { template_name };

    Request::post(&url)
        .json(&body)
        .map_err(|e| format!("Serialization error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

/// List all active sessions
pub async fn list_sessions() -> Result<Vec<SessionInfo>, String> {
    let url = format!("{}/sessions", API_BASE);

    Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

/// Get session details
pub async fn get_session(session_id: &str) -> Result<SessionInfo, String> {
    let url = format!("{}/sessions/{}", API_BASE, session_id);

    Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

/// Execute workflow in a session
pub async fn execute_workflow(
    session_id: &str,
    state: HashMap<String, serde_json::Value>,
) -> Result<ExecuteWorkflowResponse, String> {
    let url = format!("{}/sessions/{}/execute", API_BASE, session_id);
    let body = ExecuteWorkflowRequest { state };

    Request::post(&url)
        .json(&body)
        .map_err(|e| format!("Serialization error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

/// Delete a session
pub async fn delete_session(session_id: &str) -> Result<(), String> {
    let url = format!("{}/sessions/{}", API_BASE, session_id);

    Request::delete(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    Ok(())
}
