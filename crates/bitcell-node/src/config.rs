//! Node configuration

use serde::{Deserialize, Serialize};

/// Node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub mode: NodeMode,
    pub network_port: u16,
    pub rpc_port: u16,
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
        }
    }
}
