//! Configuration manager with file persistence

use std::path::PathBuf;
use std::sync::RwLock;

use crate::api::config::*;

pub struct ConfigManager {
    config: RwLock<Config>,
    config_path: Option<PathBuf>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            config: RwLock::new(Self::default_config()),
            config_path: None,
        }
    }

    pub fn with_path(path: PathBuf) -> Result<Self, String> {
        let config = if path.exists() {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read config file: {}", e))?;

            serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse config file: {}", e))?
        } else {
            Self::default_config()
        };

        Ok(Self {
            config: RwLock::new(config),
            config_path: Some(path),
        })
    }

    fn default_config() -> Config {
        Config {
            network: NetworkConfig {
                listen_addr: "0.0.0.0:9000".to_string(),
                bootstrap_peers: vec![],
                max_peers: 50,
            },
            consensus: ConsensusConfig {
                battle_steps: 1000,
                tournament_rounds: 5,
                block_time: 6,
            },
            ebsl: EbslConfig {
                evidence_threshold: 0.7,
                slash_percentage: 0.1,
                decay_rate: 0.95,
            },
            economics: EconomicsConfig {
                initial_reward: 50_000_000,
                halving_interval: 210_000,
                base_gas_price: 1000,
            },
        }
    }

    pub fn get_config(&self) -> Result<Config, String> {
        let config = self.config.read().unwrap();
        Ok(config.clone())
    }

    pub fn update_config(&self, new_config: Config) -> Result<(), String> {
        let mut config = self.config.write().unwrap();
        *config = new_config.clone();
        drop(config);

        // Persist to file if path is set
        if let Some(ref path) = self.config_path {
            self.save_to_file(path)?;
        }

        Ok(())
    }

    fn save_to_file(&self, path: &PathBuf) -> Result<(), String> {
        let config = self.config.read().unwrap();

        let content = serde_json::to_string_pretty(&*config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        tracing::info!("Configuration saved to {:?}", path);

        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}
