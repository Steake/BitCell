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
    let nodes = state.process.list_nodes();
    let total = nodes.len();

    Ok(Json(NodesResponse { nodes, total }))
}

/// Get information about a specific node
pub async fn get_node(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<NodeResponse>, (StatusCode, Json<ErrorResponse>)> {
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
    Json(_req): Json<StartNodeRequest>,
) -> Result<Json<NodeResponse>, (StatusCode, Json<ErrorResponse>)> {
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
