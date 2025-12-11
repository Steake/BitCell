//! Checkpoint management for fast sync
//!
//! Checkpoints allow light clients to skip validation of ancient history
//! by trusting specific block headers verified by the community.

use bitcell_consensus::BlockHeader;
use bitcell_crypto::Hash256;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{Result, Error};

/// A checkpoint is a trusted block header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Block height
    pub height: u64,
    
    /// Block hash
    pub hash: Hash256,
    
    /// Block header
    pub header: BlockHeader,
    
    /// Checkpoint name/description
    pub name: String,
    
    /// When this checkpoint was added
    pub added_at: u64,
}

impl Checkpoint {
    /// Create a new checkpoint
    pub fn new(header: BlockHeader, name: String) -> Self {
        Self {
            height: header.height,
            hash: header.hash(),
            header,
            name,
            added_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before Unix epoch")
                .as_secs(),
        }
    }
    
    /// Verify checkpoint integrity
    pub fn verify(&self) -> Result<()> {
        // Verify hash matches header
        if self.hash != self.header.hash() {
            return Err(Error::InvalidCheckpoint("hash mismatch".to_string()));
        }
        
        // Verify height matches
        if self.height != self.header.height {
            return Err(Error::InvalidCheckpoint("height mismatch".to_string()));
        }
        
        Ok(())
    }
}

/// Manages checkpoints for fast sync
pub struct CheckpointManager {
    /// Checkpoints sorted by height
    checkpoints: BTreeMap<u64, Checkpoint>,
}

impl CheckpointManager {
    /// Create a new checkpoint manager with hardcoded checkpoints
    pub fn new() -> Self {
        let mut manager = Self {
            checkpoints: BTreeMap::new(),
        };
        
        // Add hardcoded checkpoints for testnet/mainnet
        // In production, these would be updated with each release
        manager.add_hardcoded_checkpoints();
        
        manager
    }
    
    /// Add hardcoded checkpoints
    fn add_hardcoded_checkpoints(&mut self) {
        // Genesis is always a checkpoint
        // Additional checkpoints would be added here in production
    }
    
    /// Add a checkpoint
    pub fn add_checkpoint(&mut self, checkpoint: Checkpoint) -> Result<()> {
        checkpoint.verify()?;
        self.checkpoints.insert(checkpoint.height, checkpoint);
        Ok(())
    }
    
    /// Get checkpoint at or before a height
    pub fn get_checkpoint_at_or_before(&self, height: u64) -> Option<&Checkpoint> {
        self.checkpoints.range(..=height).next_back().map(|(_, cp)| cp)
    }
    
    /// Get the latest checkpoint
    pub fn latest_checkpoint(&self) -> Option<&Checkpoint> {
        self.checkpoints.values().next_back()
    }
    
    /// Get all checkpoints
    pub fn all_checkpoints(&self) -> Vec<&Checkpoint> {
        self.checkpoints.values().collect()
    }
    
    /// Get checkpoint by height
    pub fn get_checkpoint(&self, height: u64) -> Option<&Checkpoint> {
        self.checkpoints.get(&height)
    }
    
    /// Remove checkpoints older than a height
    pub fn prune_old_checkpoints(&mut self, keep_from_height: u64) {
        self.checkpoints.retain(|&h, _| h >= keep_from_height);
    }
}

impl Default for CheckpointManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcell_crypto::{SecretKey, Hash256};

    fn create_test_header(height: u64) -> BlockHeader {
        BlockHeader {
            height,
            prev_hash: Hash256::zero(),
            tx_root: Hash256::zero(),
            state_root: Hash256::zero(),
            timestamp: height * 10,
            proposer: SecretKey::generate().public_key(),
            vrf_output: [0u8; 32],
            vrf_proof: vec![],
            work: 100,
        }
    }

    #[test]
    fn test_checkpoint_creation() {
        let header = create_test_header(1000);
        let checkpoint = Checkpoint::new(header.clone(), "Test Checkpoint".to_string());
        
        assert_eq!(checkpoint.height, 1000);
        assert_eq!(checkpoint.hash, header.hash());
        checkpoint.verify().unwrap();
    }

    #[test]
    fn test_checkpoint_manager() {
        let mut manager = CheckpointManager::new();
        
        let header1 = create_test_header(1000);
        let cp1 = Checkpoint::new(header1, "CP 1000".to_string());
        manager.add_checkpoint(cp1).unwrap();
        
        let header2 = create_test_header(2000);
        let cp2 = Checkpoint::new(header2, "CP 2000".to_string());
        manager.add_checkpoint(cp2).unwrap();
        
        assert_eq!(manager.latest_checkpoint().unwrap().height, 2000);
        assert_eq!(manager.get_checkpoint_at_or_before(1500).unwrap().height, 1000);
    }

    #[test]
    fn test_checkpoint_pruning() {
        let mut manager = CheckpointManager::new();
        
        for i in 0..10 {
            let header = create_test_header(i * 1000);
            let cp = Checkpoint::new(header, format!("CP {}", i * 1000));
            manager.add_checkpoint(cp).unwrap();
        }
        
        manager.prune_old_checkpoints(5000);
        assert_eq!(manager.all_checkpoints().len(), 5);
        assert!(manager.get_checkpoint(3000).is_none());
    }
}
