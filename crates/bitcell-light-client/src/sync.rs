//! Header synchronization protocol
//!
//! Implements efficient header sync with checkpoint support.

use bitcell_consensus::BlockHeader;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use crate::{
    Result, HeaderChain, CheckpointManager, Checkpoint,
};

/// Sync status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStatus {
    /// Not started
    Idle,
    
    /// Syncing from checkpoint
    SyncingFromCheckpoint,
    
    /// Syncing headers
    SyncingHeaders,
    
    /// Fully synced
    Synced,
    
    /// Error occurred
    Error,
}

/// Header synchronization manager
pub struct HeaderSync {
    /// Header chain being synced
    header_chain: Arc<HeaderChain>,
    
    /// Checkpoint manager
    checkpoint_manager: Arc<RwLock<CheckpointManager>>,
    
    /// Current sync status
    status: Arc<RwLock<SyncStatus>>,
    
    /// Target height to sync to
    target_height: Arc<RwLock<Option<u64>>>,
    
    /// Sync batch size
    batch_size: usize,
}

impl HeaderSync {
    /// Create a new header sync manager
    pub fn new(
        header_chain: Arc<HeaderChain>,
        checkpoint_manager: Arc<RwLock<CheckpointManager>>,
    ) -> Self {
        Self {
            header_chain,
            checkpoint_manager,
            status: Arc::new(RwLock::new(SyncStatus::Idle)),
            target_height: Arc::new(RwLock::new(None)),
            batch_size: 500, // Request 500 headers at a time
        }
    }
    
    /// Get current sync status
    pub fn status(&self) -> SyncStatus {
        *self.status.read()
    }
    
    /// Get sync progress (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        let target = match *self.target_height.read() {
            Some(h) => h,
            None => return 0.0,
        };
        
        let current = self.header_chain.tip_height();
        
        if target == 0 {
            return 1.0;
        }
        
        ((current as f64) / (target as f64)).min(1.0)
    }
    
    /// Start syncing to a target height
    pub async fn sync_to(&self, target_height: u64) -> Result<()> {
        *self.target_height.write() = Some(target_height);
        *self.status.write() = SyncStatus::SyncingFromCheckpoint;
        
        // First, try to jump to a checkpoint
        let checkpoint_result = self.sync_from_checkpoint(target_height).await;
        
        if checkpoint_result.is_ok() {
            tracing::info!("Jumped to checkpoint successfully");
        }
        
        // Then sync remaining headers
        *self.status.write() = SyncStatus::SyncingHeaders;
        self.sync_remaining_headers(target_height).await?;
        
        // Only mark as synced if we actually reached the target
        if self.header_chain.tip_height() >= target_height {
            *self.status.write() = SyncStatus::Synced;
        }
        Ok(())
    }
    
    /// Sync from the best checkpoint
    async fn sync_from_checkpoint(&self, target_height: u64) -> Result<()> {
        let checkpoint_manager = self.checkpoint_manager.read();
        let checkpoint = checkpoint_manager.get_checkpoint_at_or_before(target_height);
        
        if let Some(cp) = checkpoint {
            if cp.height > self.header_chain.tip_height() {
                tracing::info!("Using checkpoint at height {}", cp.height);
                self.header_chain.add_header(cp.header.clone())?;
            }
        }
        
        Ok(())
    }
    
    /// Sync remaining headers from current tip to target
    async fn sync_remaining_headers(&self, target_height: u64) -> Result<()> {
        let mut current_height = self.header_chain.tip_height();
        
        while current_height < target_height {
            let end_height = std::cmp::min(
                current_height + self.batch_size as u64,
                target_height
            );
            
            // In a real implementation, this would request headers from peers
            // For now, this is a placeholder that would be implemented with
            // the actual network protocol
            tracing::debug!(
                "Would request headers from {} to {}",
                current_height + 1,
                end_height
            );
            
            // Simulate some network delay
            sleep(Duration::from_millis(100)).await;
            
            // Break if no progress (would need peer responses in real impl)
            break;
        }
        
        Ok(())
    }
    
    /// Add a batch of headers received from a peer
    pub fn add_header_batch(&self, headers: Vec<BlockHeader>) -> Result<usize> {
        let mut added = 0;
        
        for header in headers {
            match self.header_chain.add_header(header) {
                Ok(()) => added += 1,
                Err(e) => {
                    tracing::warn!("Failed to add header: {}", e);
                    // Continue with remaining headers
                }
            }
        }
        
        Ok(added)
    }
    
    /// Add a checkpoint dynamically
    pub fn add_checkpoint(&self, checkpoint: Checkpoint) -> Result<()> {
        self.checkpoint_manager.write().add_checkpoint(checkpoint)
    }
    
    /// Check if we're synced to a specific height
    pub fn is_synced_to(&self, height: u64) -> bool {
        self.header_chain.tip_height() >= height
    }
    
    /// Get the current tip height
    pub fn tip_height(&self) -> u64 {
        self.header_chain.tip_height()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcell_crypto::{SecretKey, Hash256};
    use crate::HeaderChainConfig;

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
    fn test_header_sync_creation() {
        let genesis = create_genesis();
        let config = HeaderChainConfig::default();
        let chain = Arc::new(HeaderChain::new(genesis, config));
        let checkpoint_manager = Arc::new(RwLock::new(CheckpointManager::new()));
        
        let sync = HeaderSync::new(chain, checkpoint_manager);
        
        assert_eq!(sync.status(), SyncStatus::Idle);
        assert_eq!(sync.progress(), 0.0);
    }

    #[test]
    fn test_add_header_batch() {
        let genesis = create_genesis();
        let config = HeaderChainConfig::default();
        let chain = Arc::new(HeaderChain::new(genesis.clone(), config));
        let checkpoint_manager = Arc::new(RwLock::new(CheckpointManager::new()));
        
        let sync = HeaderSync::new(chain.clone(), checkpoint_manager);
        
        // Create a batch of headers
        let mut headers = vec![];
        let mut prev = genesis;
        for _ in 0..10 {
            let next = create_next_header(&prev);
            headers.push(next.clone());
            prev = next;
        }
        
        let added = sync.add_header_batch(headers).unwrap();
        assert_eq!(added, 10);
        assert_eq!(chain.tip_height(), 10);
    }

    #[test]
    fn test_sync_progress() {
        let genesis = create_genesis();
        let config = HeaderChainConfig::default();
        let chain = Arc::new(HeaderChain::new(genesis, config));
        let checkpoint_manager = Arc::new(RwLock::new(CheckpointManager::new()));
        
        let sync = HeaderSync::new(chain, checkpoint_manager);
        
        *sync.target_height.write() = Some(100);
        
        // Should show 0% progress initially
        let progress = sync.progress();
        assert!(progress < 0.01);
    }
}
