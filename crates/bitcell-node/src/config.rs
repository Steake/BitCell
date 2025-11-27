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
    /// Block production interval in seconds.
    /// Defaults to 10 seconds for testing. Use 600 (10 minutes) for production.
    pub block_time_secs: u64,
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
            block_time_secs: 10, // Default to 10 seconds for testing
        }
    }
}
