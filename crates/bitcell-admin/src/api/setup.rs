//! Setup wizard API endpoints

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;
use crate::setup::NodeEndpoint;

#[derive(Debug, Serialize)]
pub struct SetupStatusResponse {
    pub initialized: bool,
    pub config_path: Option<String>,
    pub data_dir: Option<String>,
    pub nodes: Vec<NodeEndpoint>,
}

#[derive(Debug, Deserialize)]
pub struct AddNodeRequest {
    pub id: String,
    pub node_type: String,
    pub metrics_endpoint: String,
    pub rpc_endpoint: String,
}

#[derive(Debug, Deserialize)]
pub struct SetConfigPathRequest {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct SetDataDirRequest {
    pub path: String,
}

/// Get setup status
pub async fn get_setup_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SetupStatusResponse>, (StatusCode, Json<String>)> {
    let setup_state = state.setup.get_state();

    let response = SetupStatusResponse {
        initialized: setup_state.initialized,
        config_path: setup_state.config_path.map(|p| p.to_string_lossy().to_string()),
        data_dir: setup_state.data_dir.map(|p| p.to_string_lossy().to_string()),
        nodes: setup_state.nodes,
    };

    Ok(Json(response))
}

/// Add a node endpoint
pub async fn add_node(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddNodeRequest>,
) -> Result<Json<NodeEndpoint>, (StatusCode, Json<String>)> {
    let node = NodeEndpoint {
        id: req.id,
        node_type: req.node_type,
        metrics_endpoint: req.metrics_endpoint,
        rpc_endpoint: req.rpc_endpoint,
    };

    state.setup.add_node(node.clone());

    // Save setup state
    let setup_path = std::path::PathBuf::from(".bitcell/admin/setup.json");
    state.setup.save_to_file(&setup_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e)))?;

    tracing::info!("Added node: {}", node.id);

    Ok(Json(node))
}

/// Set config path
pub async fn set_config_path(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetConfigPathRequest>,
) -> Result<Json<String>, (StatusCode, Json<String>)> {
    let path = std::path::PathBuf::from(&req.path);

    state.setup.set_config_path(path.clone());

    // Save setup state
    let setup_path = std::path::PathBuf::from(".bitcell/admin/setup.json");
    state.setup.save_to_file(&setup_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e)))?;

    Ok(Json(req.path))
}

/// Set data directory
pub async fn set_data_dir(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetDataDirRequest>,
) -> Result<Json<String>, (StatusCode, Json<String>)> {
    let path = std::path::PathBuf::from(&req.path);

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&path)
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(format!("Failed to create data directory: {}", e))
        ))?;

    state.setup.set_data_dir(path);

    // Save setup state
    let setup_path = std::path::PathBuf::from(".bitcell/admin/setup.json");
    state.setup.save_to_file(&setup_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e)))?;

    Ok(Json(req.path))
}

/// Mark setup as complete
pub async fn complete_setup(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SetupStatusResponse>, (StatusCode, Json<String>)> {
    state.setup.mark_initialized();

    // Save setup state
    let setup_path = std::path::PathBuf::from(".bitcell/admin/setup.json");
    state.setup.save_to_file(&setup_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e)))?;

    tracing::info!("Setup completed");

    let setup_state = state.setup.get_state();

    let response = SetupStatusResponse {
        initialized: setup_state.initialized,
        config_path: setup_state.config_path.map(|p| p.to_string_lossy().to_string()),
        data_dir: setup_state.data_dir.map(|p| p.to_string_lossy().to_string()),
        nodes: setup_state.nodes,
    };

    Ok(Json(response))
}
