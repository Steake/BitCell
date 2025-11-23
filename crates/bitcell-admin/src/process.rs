//! Process manager for spawning and managing node processes

use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::api::{NodeInfo, NodeType, NodeStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub node_type: NodeType,
    pub data_dir: String,
    pub port: u16,
    pub rpc_port: u16,
    pub log_level: String,
    pub network: String,
}

struct ManagedNode {
    info: NodeInfo,
    config: NodeConfig,
    process: Option<Child>,
}

pub struct ProcessManager {
    nodes: Arc<RwLock<HashMap<String, ManagedNode>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new node (without starting it)
    pub fn register_node(&self, id: String, config: NodeConfig) -> NodeInfo {
        let info = NodeInfo {
            id: id.clone(),
            node_type: config.node_type,
            status: NodeStatus::Stopped,
            address: "127.0.0.1".to_string(),
            port: config.port,
            started_at: None,
        };

        let managed = ManagedNode {
            info: info.clone(),
            config,
            process: None,
        };

        let mut nodes = self.nodes.write();
        nodes.insert(id, managed);

        info
    }

    /// Start a node process
    pub fn start_node(&self, id: &str) -> Result<NodeInfo, String> {
        let mut nodes = self.nodes.write();
        let node = nodes.get_mut(id)
            .ok_or_else(|| format!("Node '{}' not found", id))?;

        if node.process.is_some() {
            return Err("Node is already running".to_string());
        }

        // Build command to start node
        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("-p")
            .arg("bitcell-node")
            .arg("--")
            .arg(match node.config.node_type {
                NodeType::Validator => "validator",
                NodeType::Miner => "miner",
                NodeType::FullNode => "full-node",
            })
            .arg("--port")
            .arg(node.config.port.to_string())
            .arg("--rpc-port")
            .arg(node.config.rpc_port.to_string())
            .arg("--data-dir")
            .arg(&node.config.data_dir)
            .env("RUST_LOG", &node.config.log_level)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        tracing::info!("Starting node '{}' with command: {:?}", id, cmd);

        // Spawn the process
        let child = cmd.spawn()
            .map_err(|e| format!("Failed to spawn process: {}", e))?;

        node.process = Some(child);
        node.info.status = NodeStatus::Running;
        node.info.started_at = Some(chrono::Utc::now());

        tracing::info!("Node '{}' started successfully", id);

        Ok(node.info.clone())
    }

    /// Stop a node process
    pub fn stop_node(&self, id: &str) -> Result<NodeInfo, String> {
        let mut nodes = self.nodes.write();
        let node = nodes.get_mut(id)
            .ok_or_else(|| format!("Node '{}' not found", id))?;

        if let Some(mut process) = node.process.take() {
            tracing::info!("Stopping node '{}'", id);

            // Try graceful shutdown first
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                let pid = process.id();
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }

                // Wait up to 5 seconds for graceful shutdown
                let timeout = std::time::Duration::from_secs(5);
                let start = std::time::Instant::now();

                while start.elapsed() < timeout {
                    match process.try_wait() {
                        Ok(Some(_)) => break,
                        Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
                        Err(e) => {
                            tracing::error!("Error waiting for process: {}", e);
                            break;
                        }
                    }
                }
            }

            // Force kill if still running
            if let Err(e) = process.kill() {
                tracing::warn!("Failed to kill process for node '{}': {}", id, e);
            }

            let _ = process.wait();

            node.info.status = NodeStatus::Stopped;
            node.info.started_at = None;

            tracing::info!("Node '{}' stopped", id);

            Ok(node.info.clone())
        } else {
            Err("Node is not running".to_string())
        }
    }

    /// Get node information
    pub fn get_node(&self, id: &str) -> Option<NodeInfo> {
        let nodes = self.nodes.read();
        nodes.get(id).map(|n| n.info.clone())
    }

    /// List all nodes
    pub fn list_nodes(&self) -> Vec<NodeInfo> {
        let nodes = self.nodes.read();
        nodes.values().map(|n| n.info.clone()).collect()
    }

    /// Check if node process is still alive
    pub fn check_node_health(&self, id: &str) -> bool {
        let mut nodes = self.nodes.write();
        if let Some(node) = nodes.get_mut(id) {
            if let Some(ref mut process) = node.process {
                match process.try_wait() {
                    Ok(Some(_)) => {
                        // Process has exited
                        node.process = None;
                        node.info.status = NodeStatus::Error;
                        node.info.started_at = None;
                        false
                    }
                    Ok(None) => {
                        // Still running
                        true
                    }
                    Err(_) => {
                        node.info.status = NodeStatus::Error;
                        false
                    }
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Cleanup all node processes on shutdown
    pub fn shutdown(&self) {
        let mut nodes = self.nodes.write();
        for (id, node) in nodes.iter_mut() {
            if let Some(mut process) = node.process.take() {
                tracing::info!("Shutting down node '{}'", id);
                let _ = process.kill();
                let _ = process.wait();
            }
        }
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}
