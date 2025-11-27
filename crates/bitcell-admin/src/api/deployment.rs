//! Deployment API endpoints

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;
use super::NodeType;

#[derive(Debug, Deserialize)]
pub struct DeployNodeRequest {
    pub node_type: NodeType,
    pub count: usize,
    pub config: Option<DeploymentConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeploymentConfig {
    pub network: String,
    pub data_dir: Option<String>,
    pub log_level: Option<String>,
    pub port_start: Option<u16>,
    pub enable_dht: Option<bool>,
    pub bootstrap_nodes: Option<Vec<String>>,
    pub key_seed: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DeploymentResponse {
    pub deployment_id: String,
    pub status: String,
    pub nodes_deployed: usize,
    pub message: String,
    pub nodes: Vec<crate::api::NodeInfo>,
}

#[derive(Debug, Serialize)]
pub struct DeploymentStatusResponse {
    pub active_deployments: usize,
    pub total_nodes: usize,
    pub deployments: Vec<DeploymentInfo>,
}

#[derive(Debug, Serialize)]
pub struct DeploymentInfo {
    pub id: String,
    pub node_type: NodeType,
    pub node_count: usize,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Deploy new nodes
pub async fn deploy_node(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeployNodeRequest>,
) -> Result<Json<DeploymentResponse>, (StatusCode, Json<String>)> {
    // Generate deployment ID
    let deployment_id = format!("deploy-{}", chrono::Utc::now().timestamp());

    let deployment = state.deployment.clone();
    let node_type = req.node_type;
    let count = req.count;
    let config = req.config;

    // Perform deployment synchronously to return node info
    let nodes = deployment.deploy_nodes(&deployment_id, node_type, count, config).await;

    Ok(Json(DeploymentResponse {
        deployment_id,
        status: "completed".to_string(),
        nodes_deployed: req.count,
        message: format!(
            "Deployed {} {:?} node(s)",
            req.count, req.node_type
        ),
        nodes,
    }))
}

/// Get deployment status
pub async fn deployment_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<DeploymentStatusResponse>, (StatusCode, Json<String>)> {
    // Get actual node status from process manager
    let nodes = state.process.list_nodes();

    // Group nodes by type and count
    let mut validator_count = 0;
    let mut miner_count = 0;
    let mut fullnode_count = 0;

    for node in &nodes {
        match node.node_type {
            super::NodeType::Validator => validator_count += 1,
            super::NodeType::Miner => miner_count += 1,
            super::NodeType::FullNode => fullnode_count += 1,
        }
    }

    let mut deployments = Vec::new();

    if validator_count > 0 {
        deployments.push(DeploymentInfo {
            id: "validators".to_string(),
            node_type: NodeType::Validator,
            node_count: validator_count,
            status: "running".to_string(),
            created_at: chrono::Utc::now(), // TODO: Track actual creation time
        });
    }

    if miner_count > 0 {
        deployments.push(DeploymentInfo {
            id: "miners".to_string(),
            node_type: NodeType::Miner,
            node_count: miner_count,
            status: "running".to_string(),
            created_at: chrono::Utc::now(),
        });
    }

    if fullnode_count > 0 {
        deployments.push(DeploymentInfo {
            id: "fullnodes".to_string(),
            node_type: NodeType::FullNode,
            node_count: fullnode_count,
            status: "running".to_string(),
            created_at: chrono::Utc::now(),
        });
    }

    let response = DeploymentStatusResponse {
        active_deployments: deployments.len(),
        total_nodes: nodes.len(),
        deployments,
    };

    Ok(Json(response))
}
