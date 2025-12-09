//! Fork choice rule (heaviest chain)

use crate::block::{Block, BlockHeader};
use bitcell_crypto::Hash256;
use std::collections::HashMap;

/// Chain state for fork choice
#[derive(Debug, Clone)]
pub struct ChainState {
    /// Blocks by hash
    pub blocks: HashMap<Hash256, Block>,
    
    /// Headers by hash
    pub headers: HashMap<Hash256, BlockHeader>,
    
    /// Chain tips
    pub tips: Vec<Hash256>,
}

impl ChainState {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            headers: HashMap::new(),
            tips: Vec::new(),
        }
    }

    /// Add a block
    pub fn add_block(&mut self, block: Block) {
        let hash = block.hash();
        self.headers.insert(hash, block.header.clone());
        self.blocks.insert(hash, block);
    }

    /// Compute cumulative work for a chain
    pub fn chain_work(&self, tip: Hash256) -> u64 {
        let mut work = 0u64;
        let mut current = tip;

        loop {
            if let Some(header) = self.headers.get(&current) {
                work += header.work;
                
                // Stop at genesis
                if header.height == 0 {
                    break;
                }
                
                current = header.prev_hash;
            } else {
                break;
            }
        }

        work
    }

    /// Select the heaviest chain tip
    pub fn best_tip(&self) -> Option<Hash256> {
        self.tips
            .iter()
            .max_by_key(|&&tip| self.chain_work(tip))
            .copied()
    }
}

impl Default for ChainState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{Block, BlockHeader, Transaction};
    use bitcell_crypto::{PublicKey, SecretKey, Signature};

    fn create_test_block(height: u64, prev_hash: Hash256, work: u64) -> Block {
        let sk = SecretKey::generate();
        Block {
            header: BlockHeader {
                height,
                prev_hash,
                tx_root: Hash256::zero(),
                state_root: Hash256::zero(),
                timestamp: 0,
                proposer: sk.public_key(),
                vrf_output: [0u8; 32],
                vrf_proof: vec![],
                work,
                aggregation_commitment: [0u8; 32],
            },
            transactions: vec![],
            battle_proofs: vec![],
            state_proofs: vec![],
            signature: sk.sign(b"test"),
        }
    }

    #[test]
    fn test_chain_work() {
        let mut state = ChainState::new();

        // Create a simple chain
        let genesis = create_test_block(0, Hash256::zero(), 100);
        let genesis_hash = genesis.hash();
        state.add_block(genesis);

        let block1 = create_test_block(1, genesis_hash, 100);
        let block1_hash = block1.hash();
        state.add_block(block1);

        let block2 = create_test_block(2, block1_hash, 100);
        let block2_hash = block2.hash();
        state.add_block(block2);

        state.tips.push(block2_hash);

        // Total work should be 300
        assert_eq!(state.chain_work(block2_hash), 300);
    }

    #[test]
    fn test_best_tip_selection() {
        let mut state = ChainState::new();

        let genesis = create_test_block(0, Hash256::zero(), 100);
        let genesis_hash = genesis.hash();
        state.add_block(genesis);

        // Create two competing chains
        let block1a = create_test_block(1, genesis_hash, 100);
        let block1a_hash = block1a.hash();
        state.add_block(block1a);

        let block1b = create_test_block(1, genesis_hash, 150);
        let block1b_hash = block1b.hash();
        state.add_block(block1b);

        state.tips.push(block1a_hash);
        state.tips.push(block1b_hash);

        // block1b has more work, should be selected
        let best = state.best_tip().unwrap();
        assert_eq!(best, block1b_hash);
    }
}
