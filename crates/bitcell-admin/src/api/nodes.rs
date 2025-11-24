//! Node management API endpoints

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;
use super::NodeInfo;

#[derive(Debug, Serialize)]
pub struct NodesResponse {
    pub nodes: Vec<NodeInfo>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct NodeResponse {
    pub node: NodeInfo,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Deserialize)]
pub struct StartNodeRequest {
    pub config: Option<serde_json::Value>,
}

/// Validate node ID format (alphanumeric, hyphens, and underscores only)
fn validate_node_id(id: &str) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if id.is_empty() || !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid node ID format".to_string(),
            }),
        ));
    }
    Ok(())
}

/// List all registered nodes
pub async fn list_nodes(
    State(state): State<Arc<AppState>>,
) -> Result<Json<NodesResponse>, (StatusCode, Json<ErrorResponse>)> {
    let nodes = state.process.list_nodes();
    let total = nodes.len();

    Ok(Json(NodesResponse { nodes, total }))
}

/// Get information about a specific node
pub async fn get_node(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<NodeResponse>, (StatusCode, Json<ErrorResponse>)> {
    validate_node_id(&id)?;

    match state.process.get_node(&id) {
        Some(node) => Ok(Json(NodeResponse { node })),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Node '{}' not found", id),
            }),
        )),
    }
}

/// Start a node
pub async fn start_node(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<StartNodeRequest>,
) -> Result<Json<NodeResponse>, (StatusCode, Json<ErrorResponse>)> {
    validate_node_id(&id)?;

    // Config is not supported yet
    if req.config.is_some() {
        tracing::warn!("Node '{}': Rejected start request with unsupported config", id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Custom config is not supported yet".to_string(),
            }),
        ));
    }

    match state.process.start_node(&id) {
        Ok(node) => {
            tracing::info!("Started node '{}' successfully", id);
            Ok(Json(NodeResponse { node }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to start node '{}': {}", id, e),
            }),
        )),
    }
}

/// Stop a node
pub async fn stop_node(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<NodeResponse>, (StatusCode, Json<ErrorResponse>)> {
    validate_node_id(&id)?;

    match state.process.stop_node(&id) {
        Ok(node) => {
            tracing::info!("Stopped node '{}' successfully", id);
            Ok(Json(NodeResponse { node }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to stop node '{}': {}", id, e),
            }),
        )),
    }
}
