//! Deployment manager for nodes

use std::sync::Arc;

use crate::api::NodeType;
use crate::process::{ProcessManager, NodeConfig};
use crate::setup::{SetupManager, NodeEndpoint};

pub struct DeploymentManager {
    process: Arc<ProcessManager>,
    setup: Arc<SetupManager>,
}

impl DeploymentManager {
    pub fn new(process: Arc<ProcessManager>, setup: Arc<SetupManager>) -> Self {
        Self { process, setup }
    }

    pub async fn deploy_nodes(&self, deployment_id: &str, node_type: NodeType, count: usize) {
        tracing::info!(
            "Starting deployment {}: deploying {} {:?} nodes",
            deployment_id,
            count,
            node_type
        );

        // Find the highest used port to avoid conflicts
        // Using higher ports (19000+) to avoid conflicts with system services
        let mut base_port = match node_type {
            NodeType::Validator => 19000,
            NodeType::Miner => 19100,
            NodeType::FullNode => 19200,
        };
        
        // Check existing nodes in process manager
        let existing_nodes = self.process.list_nodes();
        for node in &existing_nodes {
            if node.port >= base_port {
                // We use port (P2P) and port+1 (Metrics), so next available is port+2
                base_port = std::cmp::max(base_port, node.port + 2);
            }
        }
        
        // Check existing nodes in setup manager
        let setup_nodes = self.setup.get_nodes();
        for node in &setup_nodes {
            // Parse port from metrics endpoint if possible, or just skip
            // This is a heuristic
            if let Some(port_str) = node.metrics_endpoint.split(':').last() {
                if let Some(port_part) = port_str.split('/').next() {
                    if let Ok(metrics_port) = port_part.parse::<u16>() {
                        // metrics_port is port + 1, so P2P port is metrics_port - 1
                        let p2p_port = metrics_port.saturating_sub(1);
                        if p2p_port >= base_port {
                            base_port = std::cmp::max(base_port, p2p_port + 2);
                        }
                    }
                }
            }
        }

        let base_rpc_port = base_port + 1000;

        for i in 0..count {
            let node_id = format!("{:?}-{}-{}", node_type, deployment_id, i);
            // Increment by 2 to allow space for metrics port (port + 1)
            let port = base_port + (i * 2) as u16;
            let rpc_port = base_rpc_port + i as u16;
            
            let config = NodeConfig {
                node_type,
                data_dir: format!("/tmp/bitcell/{}", node_id),
                port,
                rpc_port,
                log_level: "info".to_string(),
                network: "testnet".to_string(),
            };

            // Register the node (but don't start it automatically)
            // Note: The UI calls start_node separately, or we could start it here.
            // But wait, the UI "Deploy" button calls deploy_node, which spawns this task.
            // The UI then refreshes the list. It doesn't automatically start them?
            // The screenshot shows them as "Running".
            // Let's check process.rs start_node again.
            // Ah, register_node returns NodeStatus::Stopped.
            // So the user must have clicked Start.
            // But wait, if I register them in SetupManager, they appear in the list.
            
            self.process.register_node(node_id.clone(), config);
            
            // Register in SetupManager so metrics can be fetched
            let endpoint = NodeEndpoint {
                id: node_id.clone(),
                node_type: format!("{:?}", node_type).to_lowercase(),
                metrics_endpoint: format!("http://127.0.0.1:{}/metrics", port + 1),
                rpc_endpoint: format!("http://127.0.0.1:{}", rpc_port),
            };
            self.setup.add_node(endpoint);

            tracing::info!("Registered node '{}' in deployment {}", node_id, deployment_id);
            
            // Auto-start the node for convenience
            if let Err(e) = self.process.start_node(&node_id) {
                tracing::error!("Failed to auto-start node {}: {}", node_id, e);
            }
        }
        
        // Save setup state
        let setup_path = std::path::PathBuf::from(crate::setup::SETUP_FILE_PATH);
        if let Err(e) = self.setup.save_to_file(&setup_path) {
            tracing::error!("Failed to save setup state: {}", e);
        }

        tracing::info!(
            "Deployment {} completed: registered {} {:?} nodes",
            deployment_id,
            count,
            node_type
        );
    }
}
