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
}

#[derive(Debug, Serialize)]
pub struct DeploymentResponse {
    pub deployment_id: String,
    pub status: String,
    pub nodes_deployed: usize,
    pub message: String,
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

    // Trigger deployment (async)
    tokio::spawn({
        let deployment = state.deployment.clone();
        let deployment_id = deployment_id.clone();
        let node_type = req.node_type;
        let count = req.count;

        async move {
            deployment.deploy_nodes(&deployment_id, node_type, count).await;
        }
    });

    Ok(Json(DeploymentResponse {
        deployment_id,
        status: "deploying".to_string(),
        nodes_deployed: req.count,
        message: format!(
            "Deploying {} {:?} node(s)",
            req.count, req.node_type
        ),
    }))
}

/// Get deployment status
pub async fn deployment_status(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<DeploymentStatusResponse>, (StatusCode, Json<String>)> {
    // TODO: Get actual deployment status
    let response = DeploymentStatusResponse {
        active_deployments: 2,
        total_nodes: 5,
        deployments: vec![
            DeploymentInfo {
                id: "deploy-1".to_string(),
                node_type: NodeType::Validator,
                node_count: 3,
                status: "running".to_string(),
                created_at: chrono::Utc::now() - chrono::Duration::hours(2),
            },
            DeploymentInfo {
                id: "deploy-2".to_string(),
                node_type: NodeType::Miner,
                node_count: 2,
                status: "running".to_string(),
                created_at: chrono::Utc::now() - chrono::Duration::minutes(30),
            },
        ],
    };

    Ok(Json(response))
}
