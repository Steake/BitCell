//! Node management API endpoints

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;
use super::{NodeInfo, NodeStatus};

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

/// List all registered nodes
pub async fn list_nodes(
    State(state): State<Arc<AppState>>,
) -> Result<Json<NodesResponse>, (StatusCode, Json<ErrorResponse>)> {
    let nodes = state.api.list_nodes();
    let total = nodes.len();

    Ok(Json(NodesResponse { nodes, total }))
}

/// Get information about a specific node
pub async fn get_node(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<NodeResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.api.get_node(&id) {
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
    Json(_req): Json<StartNodeRequest>,
) -> Result<Json<NodeResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Update status to starting
    if !state.api.update_node_status(&id, NodeStatus::Starting) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Node '{}' not found", id),
            }),
        ));
    }

    // TODO: Actually start the node process
    // For now, simulate starting
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Update to running
    state.api.update_node_status(&id, NodeStatus::Running);

    match state.api.get_node(&id) {
        Some(node) => Ok(Json(NodeResponse { node })),
        None => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to retrieve node after starting".to_string(),
            }),
        )),
    }
}

/// Stop a node
pub async fn stop_node(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<NodeResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Update status to stopping
    if !state.api.update_node_status(&id, NodeStatus::Stopping) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Node '{}' not found", id),
            }),
        ));
    }

    // TODO: Actually stop the node process
    // For now, simulate stopping
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Update to stopped
    state.api.update_node_status(&id, NodeStatus::Stopped);

    match state.api.get_node(&id) {
        Some(node) => Ok(Json(NodeResponse { node })),
        None => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to retrieve node after stopping".to_string(),
            }),
        )),
    }
}
