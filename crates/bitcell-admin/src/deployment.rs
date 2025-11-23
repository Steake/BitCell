//! Deployment manager for nodes

use std::collections::HashMap;
use std::sync::RwLock;

use crate::api::NodeType;

pub struct DeploymentManager {
    deployments: RwLock<HashMap<String, Deployment>>,
}

struct Deployment {
    id: String,
    node_type: NodeType,
    node_count: usize,
    status: DeploymentStatus,
}

#[derive(Debug, Clone, Copy)]
enum DeploymentStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl DeploymentManager {
    pub fn new() -> Self {
        Self {
            deployments: RwLock::new(HashMap::new()),
        }
    }

    pub async fn deploy_nodes(&self, deployment_id: &str, node_type: NodeType, count: usize) {
        // Create deployment record
        {
            let mut deployments = self.deployments.write().unwrap();
            deployments.insert(
                deployment_id.to_string(),
                Deployment {
                    id: deployment_id.to_string(),
                    node_type,
                    node_count: count,
                    status: DeploymentStatus::InProgress,
                },
            );
        }

        // Simulate deployment
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Update status
        {
            let mut deployments = self.deployments.write().unwrap();
            if let Some(deployment) = deployments.get_mut(deployment_id) {
                deployment.status = DeploymentStatus::Completed;
            }
        }

        tracing::info!(
            "Deployment {} completed: {} {:?} nodes",
            deployment_id,
            count,
            node_type
        );
    }

    pub fn get_deployment(&self, id: &str) -> Option<String> {
        let deployments = self.deployments.read().unwrap();
        deployments.get(id).map(|d| d.id.clone())
    }
}

impl Default for DeploymentManager {
    fn default() -> Self {
        Self::new()
    }
}
