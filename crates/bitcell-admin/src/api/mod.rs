//! API module for admin console

pub mod nodes;
pub mod metrics;
pub mod deployment;
pub mod config;
pub mod test;
pub mod setup;
pub mod blocks;
pub mod wallet;

use std::collections::HashMap;
use std::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub node_type: NodeType,
    pub status: NodeStatus,
    pub address: String,
    pub port: u16,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub enable_dht: bool,
    pub dht_peer_count: usize,
    pub bootstrap_nodes: Vec<String>,
    pub key_seed: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Validator,
    Miner,
    FullNode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Running,
    Stopped,
    Starting,
    Stopping,
    Error,
}

/// Administrative API handler
pub struct AdminApi {
    nodes: RwLock<HashMap<String, NodeInfo>>,
}

impl AdminApi {
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_node(&self, node: NodeInfo) {
        let mut nodes = self.nodes.write().unwrap();
        nodes.insert(node.id.clone(), node);
    }

    pub fn get_node(&self, id: &str) -> Option<NodeInfo> {
        let nodes = self.nodes.read().unwrap();
        nodes.get(id).cloned()
    }

    pub fn list_nodes(&self) -> Vec<NodeInfo> {
        let nodes = self.nodes.read().unwrap();
        nodes.values().cloned().collect()
    }

    pub fn update_node_status(&self, id: &str, status: NodeStatus) -> bool {
        let mut nodes = self.nodes.write().unwrap();
        if let Some(node) = nodes.get_mut(id) {
            node.status = status;
            true
        } else {
            false
        }
    }
}

impl Default for AdminApi {
    fn default() -> Self {
        Self::new()
    }
}
