//! Validator node implementation

use crate::{NodeConfig, Result};
use bitcell_consensus::{Block, TournamentOrchestrator};
use bitcell_state::StateManager;
use bitcell_network::PeerManager;
use bitcell_crypto::Hash256;

/// Validator node
pub struct ValidatorNode {
    pub config: NodeConfig,
    pub state: StateManager,
    pub peers: PeerManager,
}

impl ValidatorNode {
    pub fn new(config: NodeConfig) -> Self {
        Self {
            config,
            state: StateManager::new(),
            peers: PeerManager::new(),
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        println!("Starting validator node on port {}", self.config.network_port);
        // Would start network listener here
        Ok(())
    }

    pub fn validate_block(&self, block: &Block) -> bool {
        // Simplified validation
        block.header.height > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let config = NodeConfig::default();
        let node = ValidatorNode::new(config);
        assert_eq!(node.state.accounts.len(), 0);
    }
}
