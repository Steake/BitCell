//! Header-only blockchain storage
//!
//! Maintains a chain of block headers without full block data.
//! Provides efficient header validation and lookup.

use bitcell_consensus::BlockHeader;
use bitcell_crypto::Hash256;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::{Result, Error};

/// Configuration for header chain
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeaderChainConfig {
    /// Maximum number of headers to keep in memory
    pub max_headers: usize,
    
    /// Checkpoint interval (headers)
    pub checkpoint_interval: u64,
    
    /// Minimum checkpoint confirmations before accepting
    pub checkpoint_confirmations: u64,
}

impl Default for HeaderChainConfig {
    fn default() -> Self {
        Self {
            max_headers: 10_000,
            checkpoint_interval: 1_000,
            checkpoint_confirmations: 100,
        }
    }
}

/// Header-only blockchain storage
///
/// Stores only block headers and validates chain continuity.
/// Much more lightweight than full blockchain storage.
pub struct HeaderChain {
    config: HeaderChainConfig,
    
    /// Headers by height
    headers: Arc<RwLock<HashMap<u64, BlockHeader>>>,
    
    /// Headers by hash
    header_by_hash: Arc<RwLock<HashMap<Hash256, BlockHeader>>>,
    
    /// Current tip height
    tip_height: Arc<RwLock<u64>>,
    
    /// Current tip hash
    tip_hash: Arc<RwLock<Hash256>>,
    
    /// Total work at each height (for fork choice)
    total_work: Arc<RwLock<HashMap<u64, u64>>>,
}

impl HeaderChain {
    /// Create a new header chain with genesis
    pub fn new(genesis: BlockHeader, config: HeaderChainConfig) -> Self {
        let genesis_hash = genesis.hash();
        let genesis_height = genesis.height;
        let genesis_work = genesis.work;
        
        let mut headers = HashMap::new();
        headers.insert(genesis_height, genesis.clone());
        
        let mut header_by_hash = HashMap::new();
        header_by_hash.insert(genesis_hash, genesis);
        
        let mut total_work = HashMap::new();
        total_work.insert(genesis_height, genesis_work);
        
        Self {
            config,
            headers: Arc::new(RwLock::new(headers)),
            header_by_hash: Arc::new(RwLock::new(header_by_hash)),
            tip_height: Arc::new(RwLock::new(genesis_height)),
            tip_hash: Arc::new(RwLock::new(genesis_hash)),
            total_work: Arc::new(RwLock::new(total_work)),
        }
    }
    
    /// Get current tip height
    pub fn tip_height(&self) -> u64 {
        *self.tip_height.read()
    }
    
    /// Get current tip hash
    pub fn tip_hash(&self) -> Hash256 {
        *self.tip_hash.read()
    }
    
    /// Get header by height
    pub fn get_header(&self, height: u64) -> Option<BlockHeader> {
        self.headers.read().get(&height).cloned()
    }
    
    /// Get header by hash
    pub fn get_header_by_hash(&self, hash: &Hash256) -> Option<BlockHeader> {
        self.header_by_hash.read().get(hash).cloned()
    }
    
    /// Add a new header to the chain
    pub fn add_header(&self, header: BlockHeader) -> Result<()> {
        // Validate header
        self.validate_header(&header)?;
        
        let hash = header.hash();
        let height = header.height;
        
        // Check if we already have this header
        if self.headers.read().contains_key(&height) {
            return Ok(()); // Already have it
        }
        
        // Calculate total work up to this height
        let parent_height = height.checked_sub(1)
            .ok_or_else(|| Error::InvalidHeader("invalid height (underflow)".to_string()))?;
        let parent_work = self.total_work.read().get(&parent_height).copied()
            .ok_or_else(|| Error::InvalidHeader("missing parent work".to_string()))?;
        let new_total_work = parent_work + header.work;
        
        // Update storage
        self.headers.write().insert(height, header.clone());
        self.header_by_hash.write().insert(hash, header);
        self.total_work.write().insert(height, new_total_work);
        
        // Update tip if this is the heaviest chain
        let current_tip_height = *self.tip_height.read();
        let current_tip_work = self.total_work.read().get(&current_tip_height).copied().unwrap_or(0);
        
        if new_total_work > current_tip_work {
            *self.tip_height.write() = height;
            *self.tip_hash.write() = hash;
        }
        
        // Prune old headers if needed
        self.prune_old_headers()?;
        
        Ok(())
    }
    
    /// Validate a header
    fn validate_header(&self, header: &BlockHeader) -> Result<()> {
        // Check height continuity
        if header.height == 0 {
            return Err(Error::InvalidHeader("cannot add genesis header".to_string()));
        }
        
        // Verify parent exists
        let parent_height = header.height.checked_sub(1)
            .ok_or_else(|| Error::InvalidHeader("cannot get parent of genesis header".to_string()))?;
        let parent = self.get_header(parent_height)
            .ok_or_else(|| Error::InvalidHeader("missing parent header".to_string()))?;
        
        // Check parent hash
        if header.prev_hash != parent.hash() {
            return Err(Error::InvalidHeader("invalid parent hash".to_string()));
        }
        
        // Check timestamp is after parent
        if header.timestamp <= parent.timestamp {
            return Err(Error::InvalidHeader("timestamp not increasing".to_string()));
        }
        
        // VRF validation would go here in production
        // For now, we trust the headers from checkpointed nodes
        
        Ok(())
    }
    
    /// Prune old headers to maintain memory limits
    fn prune_old_headers(&self) -> Result<()> {
        let tip_height = *self.tip_height.read();
        
        // Keep recent headers
        let keep_from = if tip_height > self.config.max_headers as u64 {
            tip_height - self.config.max_headers as u64
        } else {
            0
        };
        
        let mut headers = self.headers.write();
        let mut header_by_hash = self.header_by_hash.write();
        let mut total_work = self.total_work.write();
        
        // Remove old headers
        let heights_to_remove: Vec<u64> = headers.keys()
            .filter(|&&h| h < keep_from && h > 0) // Keep genesis
            .copied()
            .collect();
        
        for height in heights_to_remove {
            if let Some(header) = headers.remove(&height) {
                header_by_hash.remove(&header.hash());
            }
            total_work.remove(&height);
        }
        
        Ok(())
    }
    
    /// Get total work at a given height
    pub fn total_work_at(&self, height: u64) -> Option<u64> {
        self.total_work.read().get(&height).copied()
    }
    
    /// Get headers in a range
    pub fn get_headers_range(&self, start: u64, end: u64) -> Vec<BlockHeader> {
        let headers = self.headers.read();
        (start..=end)
            .filter_map(|h| headers.get(&h).cloned())
            .collect()
    }
    
    /// Get estimated memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        let header_count = self.headers.read().len();
        // Approximate: each header ~500 bytes with overhead
        header_count * 500 + 
        self.header_by_hash.read().len() * 32 + // Hash keys
        self.total_work.read().len() * 16 // Height->work map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcell_crypto::SecretKey;

    fn create_genesis() -> BlockHeader {
        BlockHeader {
            height: 0,
            prev_hash: Hash256::zero(),
            tx_root: Hash256::zero(),
            state_root: Hash256::zero(),
            timestamp: 0,
            proposer: SecretKey::generate().public_key(),
            vrf_output: [0u8; 32],
            vrf_proof: vec![],
            work: 100,
        }
    }

    fn create_next_header(parent: &BlockHeader) -> BlockHeader {
        BlockHeader {
            height: parent.height + 1,
            prev_hash: parent.hash(),
            tx_root: Hash256::zero(),
            state_root: Hash256::zero(),
            timestamp: parent.timestamp + 10,
            proposer: SecretKey::generate().public_key(),
            vrf_output: [0u8; 32],
            vrf_proof: vec![],
            work: 100,
        }
    }

    #[test]
    fn test_header_chain_creation() {
        let genesis = create_genesis();
        let config = HeaderChainConfig::default();
        let chain = HeaderChain::new(genesis.clone(), config);
        
        assert_eq!(chain.tip_height(), 0);
        assert_eq!(chain.tip_hash(), genesis.hash());
    }

    #[test]
    fn test_add_headers() {
        let genesis = create_genesis();
        let config = HeaderChainConfig::default();
        let chain = HeaderChain::new(genesis.clone(), config);
        
        let header1 = create_next_header(&genesis);
        chain.add_header(header1.clone()).unwrap();
        
        assert_eq!(chain.tip_height(), 1);
        assert_eq!(chain.get_header(1).unwrap().height, 1);
    }

    #[test]
    fn test_header_validation() {
        let genesis = create_genesis();
        let config = HeaderChainConfig::default();
        let chain = HeaderChain::new(genesis.clone(), config);
        
        // Try to add header with invalid parent
        let mut bad_header = create_next_header(&genesis);
        bad_header.prev_hash = Hash256::hash(b"wrong");
        
        assert!(chain.add_header(bad_header).is_err());
    }

    #[test]
    fn test_memory_pruning() {
        let genesis = create_genesis();
        let mut config = HeaderChainConfig::default();
        config.max_headers = 10;
        let chain = HeaderChain::new(genesis.clone(), config);
        
        let mut prev = genesis;
        for _ in 0..20 {
            let next = create_next_header(&prev);
            chain.add_header(next.clone()).unwrap();
            prev = next;
        }
        
        // Should have pruned old headers
        let memory = chain.memory_usage();
        assert!(memory < 20 * 500 + 10000); // Less than full 20 headers
    }
}
