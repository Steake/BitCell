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

    pub async fn deploy_nodes(&self, deployment_id: &str, node_type: NodeType, count: usize, config: Option<crate::api::deployment::DeploymentConfig>) -> Vec<crate::api::NodeInfo> {
        tracing::info!(
            "Starting deployment {}: deploying {} {:?} nodes",
            deployment_id,
            count,
            node_type
        );

        // Extract DHT config or use defaults
        let enable_dht = config.as_ref().and_then(|c| c.enable_dht).unwrap_or(false);
        let bootstrap_nodes = config.as_ref().and_then(|c| c.bootstrap_nodes.clone()).unwrap_or_default();
        let key_seed = config.as_ref().and_then(|c| c.key_seed.clone());

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
        let mut deployed_nodes = Vec::new();

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
                enable_dht,
                bootstrap_nodes: bootstrap_nodes.clone(),
                key_seed: key_seed.clone(),
            };

            // Register the node
            let mut node_info = self.process.register_node(node_id.clone(), config);
            
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
            match self.process.start_node(&node_id) {
                Ok(started_info) => {
                    node_info = started_info;
                },
                Err(e) => {
                    tracing::error!("Failed to auto-start node {}: {}", node_id, e);
                }
            }
            
            deployed_nodes.push(node_info);
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
        
        deployed_nodes
    }
}
