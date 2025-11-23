//! Miner node implementation

use crate::{NodeConfig, Result};
use bitcell_crypto::SecretKey;
use bitcell_ca::{Glider, GliderPattern};
use bitcell_state::StateManager;

/// Miner node
pub struct MinerNode {
    pub config: NodeConfig,
    pub secret_key: SecretKey,
    pub state: StateManager,
    pub glider_strategy: GliderPattern,
}

impl MinerNode {
    pub fn new(config: NodeConfig, secret_key: SecretKey) -> Self {
        Self {
            config,
            secret_key,
            state: StateManager::new(),
            glider_strategy: GliderPattern::Standard,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        println!("Starting miner node on port {}", self.config.network_port);
        println!("Glider strategy: {:?}", self.glider_strategy);
        Ok(())
    }

    pub fn generate_glider(&self) -> Glider {
        Glider::new(self.glider_strategy, bitcell_ca::Position::new(256, 512))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_miner_creation() {
        let config = NodeConfig::default();
        let sk = SecretKey::generate();
        let miner = MinerNode::new(config, sk);
        assert_eq!(miner.glider_strategy, GliderPattern::Standard);
    }

    #[test]
    fn test_glider_generation() {
        let config = NodeConfig::default();
        let sk = SecretKey::generate();
        let miner = MinerNode::new(config, sk);
        let glider = miner.generate_glider();
        assert_eq!(glider.pattern, GliderPattern::Standard);
    }
}
