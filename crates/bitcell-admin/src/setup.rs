//! Setup wizard for initial BitCell deployment

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupState {
    pub initialized: bool,
    pub config_path: Option<PathBuf>,
    pub data_dir: Option<PathBuf>,
    pub nodes: Vec<NodeEndpoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEndpoint {
    pub id: String,
    pub node_type: String,
    pub metrics_endpoint: String,
    pub rpc_endpoint: String,
}

pub struct SetupManager {
    state: RwLock<SetupState>,
}

impl SetupManager {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(SetupState {
                initialized: false,
                config_path: None,
                data_dir: None,
                nodes: Vec::new(),
            }),
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.state.read().unwrap().initialized
    }

    pub fn get_state(&self) -> SetupState {
        self.state.read().unwrap().clone()
    }

    pub fn set_config_path(&self, path: PathBuf) {
        let mut state = self.state.write().unwrap();
        state.config_path = Some(path);
    }

    pub fn set_data_dir(&self, path: PathBuf) {
        let mut state = self.state.write().unwrap();
        state.data_dir = Some(path);
    }

    pub fn add_node(&self, node: NodeEndpoint) {
        let mut state = self.state.write().unwrap();
        state.nodes.push(node);
    }

    pub fn get_nodes(&self) -> Vec<NodeEndpoint> {
        self.state.read().unwrap().nodes.clone()
    }

    pub fn mark_initialized(&self) {
        let mut state = self.state.write().unwrap();
        state.initialized = true;
    }

    /// Load setup state from file
    pub fn load_from_file(&self, path: &PathBuf) -> Result<(), String> {
        if !path.exists() {
            return Ok(()); // Not an error, just not initialized
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read setup file: {}", e))?;

        let loaded_state: SetupState = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse setup file: {}", e))?;

        let mut state = self.state.write().unwrap();
        *state = loaded_state;

        Ok(())
    }

    /// Save setup state to file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), String> {
        let state = self.state.read().unwrap();

        let content = serde_json::to_string_pretty(&*state)
            .map_err(|e| format!("Failed to serialize setup state: {}", e))?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create setup directory: {}", e))?;
        }

        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write setup file: {}", e))?;

        Ok(())
    }
}

impl Default for SetupManager {
    fn default() -> Self {
        Self::new()
    }
}
