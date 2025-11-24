//! Deployment manager for nodes

use std::sync::Arc;

use crate::api::NodeType;
use crate::process::{ProcessManager, NodeConfig};

pub struct DeploymentManager {
    process: Arc<ProcessManager>,
}

impl DeploymentManager {
    pub fn new(process: Arc<ProcessManager>) -> Self {
        Self { process }
    }

    pub async fn deploy_nodes(&self, deployment_id: &str, node_type: NodeType, count: usize) {
        tracing::info!(
            "Starting deployment {}: deploying {} {:?} nodes",
            deployment_id,
            count,
            node_type
        );

        let base_port = match node_type {
            NodeType::Validator => 9000,
            NodeType::Miner => 9100,
            NodeType::FullNode => 9200,
        };

        let base_rpc_port = base_port + 1000;

        for i in 0..count {
            let node_id = format!("{:?}-{}-{}", node_type, deployment_id, i);
            let config = NodeConfig {
                node_type,
                data_dir: format!("/tmp/bitcell/{}", node_id),
                port: base_port + i as u16,
                rpc_port: base_rpc_port + i as u16,
                log_level: "info".to_string(),
                network: "testnet".to_string(),
            };

            // Register the node (but don't start it automatically)
            self.process.register_node(node_id.clone(), config);

            tracing::info!("Registered node '{}' in deployment {}", node_id, deployment_id);
        }

        tracing::info!(
            "Deployment {} completed: registered {} {:?} nodes",
            deployment_id,
            count,
            node_type
        );
    }
}
