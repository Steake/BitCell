//! Configuration manager

use std::sync::RwLock;

use crate::api::config::*;

pub struct ConfigManager {
    config: RwLock<Config>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            config: RwLock::new(Self::default_config()),
        }
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
        *config = new_config;
        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}
