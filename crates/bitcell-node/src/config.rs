//! Node configuration

use serde::{Deserialize, Serialize};

/// Node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub mode: NodeMode,
    pub network_port: u16,
    pub rpc_port: u16,
    pub enable_dht: bool,
    pub bootstrap_nodes: Vec<String>,
    pub key_seed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeMode {
    Validator,
    Miner,
    LightClient,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            mode: NodeMode::Validator,
            network_port: 30333,
            rpc_port: 9933,
            enable_dht: false, // Disabled by default for backwards compatibility
            bootstrap_nodes: vec![],
            key_seed: None,
        }
    }
}
