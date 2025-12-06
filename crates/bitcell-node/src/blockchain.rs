///! Blockchain manager for block production and validation
///!
///! Provides functionality for:
///! - Block production with VRF-based proposer selection
///! - Block validation including signature, VRF, and transaction verification
///! - Transaction indexing for efficient lookups
///! - State management with Merkle tree root computation

use crate::{Result, MetricsRegistry};
use bitcell_consensus::{Block, BlockHeader, Transaction, BattleProof};
use bitcell_crypto::{Hash256, PublicKey, SecretKey};
use bitcell_economics::{COIN, INITIAL_BLOCK_REWARD, HALVING_INTERVAL, MAX_HALVINGS};
use bitcell_state::StateManager;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

/// Genesis block height
pub const GENESIS_HEIGHT: u64 = 0;

/// Transaction location in blockchain (block height and index within block)
#[derive(Clone, Debug)]
pub struct TxLocation {
    pub block_height: u64,
    pub tx_index: usize,
}

/// Blockchain manager
/// 
/// Maintains the blockchain state including blocks, transactions, and state root.
/// Provides O(1) transaction lookup via hash index.
#[derive(Clone)]
pub struct Blockchain {
    /// Current chain height
    height: Arc<RwLock<u64>>,
    
    /// Latest block hash
    latest_hash: Arc<RwLock<Hash256>>,
    
    /// Block storage (height -> block)
    blocks: Arc<RwLock<HashMap<u64, Block>>>,
    
    /// Transaction hash index for O(1) lookups (tx_hash -> location)
    tx_index: Arc<RwLock<HashMap<Hash256, TxLocation>>>,
    
    /// State manager
    state: Arc<RwLock<StateManager>>,
    
    /// Metrics registry
    metrics: MetricsRegistry,
    
    /// Node secret key for signing
    secret_key: Arc<SecretKey>,
}

impl Blockchain {
    /// Create new blockchain with genesis block
    pub fn new(secret_key: Arc<SecretKey>, metrics: MetricsRegistry) -> Self {
        let genesis = Self::create_genesis_block(&secret_key);
        let genesis_hash = genesis.hash();
        
        let mut blocks = HashMap::new();
        blocks.insert(GENESIS_HEIGHT, genesis);
        
        let blockchain = Self {
            height: Arc::new(RwLock::new(GENESIS_HEIGHT)),
            latest_hash: Arc::new(RwLock::new(genesis_hash)),
            blocks: Arc::new(RwLock::new(blocks)),
            tx_index: Arc::new(RwLock::new(HashMap::new())),
            state: Arc::new(RwLock::new(StateManager::new())),
            metrics,
            secret_key,
        };
        
        // Initialize metrics
        blockchain.metrics.set_chain_height(GENESIS_HEIGHT);
        blockchain.metrics.set_sync_progress(100);
        
        blockchain
    }
    
    /// Create genesis block
    fn create_genesis_block(secret_key: &SecretKey) -> Block {
        let header = BlockHeader {
            height: GENESIS_HEIGHT,
            prev_hash: Hash256::zero(),
            tx_root: Hash256::zero(),
            state_root: Hash256::zero(),
            timestamp: 0,
            proposer: secret_key.public_key(),
            vrf_output: [0u8; 32],
            vrf_proof: vec![],
            work: 0,
        };
        
        Block {
            header,
            transactions: vec![],
            battle_proofs: vec![],
            signature: secret_key.sign(&[0u8; 32]),
        }
    }
    
    /// Get current chain height
    /// 
    /// Returns the current blockchain height. If the lock is poisoned (indicating
    /// a prior panic while holding the lock), logs an error and recovers the guard.
    pub fn height(&self) -> u64 {
        *self.height.read().unwrap_or_else(|e| {
            tracing::error!("Lock poisoned in height() - prior panic detected: {}", e);
            e.into_inner()
        })
    }
    
    /// Get latest block hash
    ///
    /// Returns the hash of the latest block. If the lock is poisoned (indicating
    /// a prior panic while holding the lock), logs an error and recovers the guard.
    pub fn latest_hash(&self) -> Hash256 {
        *self.latest_hash.read().unwrap_or_else(|e| {
            tracing::error!("Lock poisoned in latest_hash() - prior panic detected: {}", e);
            e.into_inner()
        })
    }
    
    /// Get block by height
    ///
    /// Returns the block at the specified height, or None if not found.
    /// If the lock is poisoned, logs an error and recovers the guard.
    pub fn get_block(&self, height: u64) -> Option<Block> {
        self.blocks.read().unwrap_or_else(|e| {
            tracing::error!("Lock poisoned in get_block() - prior panic detected: {}", e);
            e.into_inner()
        }).get(&height).cloned()
    }

    /// Get transaction by hash using the O(1) hash index
    ///
    /// Returns the transaction and its location (block height, index) if found.
    /// This is significantly more efficient than linear scan for large blockchains.
    pub fn get_transaction_by_hash(&self, tx_hash: &Hash256) -> Option<(Transaction, TxLocation)> {
        // First, look up the location in the index
        let location = {
            let index = self.tx_index.read().unwrap_or_else(|e| {
                tracing::error!("Lock poisoned in get_transaction_by_hash() - prior panic detected: {}", e);
                e.into_inner()
            });
            index.get(tx_hash).cloned()
        };

        // Then retrieve the actual transaction from the block
        if let Some(loc) = location {
            if let Some(block) = self.get_block(loc.block_height) {
                if loc.tx_index < block.transactions.len() {
                    return Some((block.transactions[loc.tx_index].clone(), loc));
                }
            }
        }

        None
    }

    /// Get state manager (read-only access)
    pub fn state(&self) -> Arc<RwLock<StateManager>> {
        Arc::clone(&self.state)
    }
    
    /// Calculate block reward based on height (halves every HALVING_INTERVAL blocks)
    pub fn calculate_block_reward(height: u64) -> u64 {
        let halvings = height / HALVING_INTERVAL;
        if halvings >= MAX_HALVINGS {
            // After MAX_HALVINGS halvings, reward is effectively 0
            return 0;
        }
        INITIAL_BLOCK_REWARD >> halvings
    }
    
    /// Produce a new block
    pub fn produce_block(
        &self,
        transactions: Vec<Transaction>,
        battle_proofs: Vec<BattleProof>,
        winner: PublicKey,
    ) -> Result<Block> {
        let current_height = self.height();
        let new_height = current_height + 1;
        let prev_hash = self.latest_hash();
        
        // Calculate transaction root
        let tx_root = self.calculate_tx_root(&transactions);
        
        // Get current state root
        let state_root = {
            let state = self.state.read().unwrap_or_else(|e| {
                tracing::error!("Lock poisoned in produce_block() while reading state - prior panic detected: {}", e);
                e.into_inner()
            });
            state.state_root
        };

        // Generate VRF output and proof using proper VRF chaining
        // For genesis block (height 1), use previous hash as input
        // For all other blocks, use the previous block's VRF output for chaining
        //
        // NOTE: We generate VRF proof while holding the blocks lock to prevent race conditions
        // where the blockchain state could change between reading the VRF input and using it.
        let (vrf_output, vrf_proof_bytes) = if new_height == 1 {
            // First block after genesis uses genesis hash as VRF input
            let vrf_input = prev_hash.as_bytes().to_vec();
            let (vrf_output, vrf_proof) = self.secret_key.vrf_prove(&vrf_input);
            (vrf_output, bincode::serialize(&vrf_proof).unwrap_or_default())
        } else {
            // Use previous block's VRF output for proper VRF chaining
            // This ensures verifiable randomness chain where each output
            // deterministically derives from the previous output
            let blocks = self.blocks.read().unwrap_or_else(|e| {
                tracing::error!("Lock poisoned in produce_block() - prior panic detected: {}", e);
                e.into_inner()
            });

            let vrf_input = if let Some(prev_block) = blocks.get(&current_height) {
                prev_block.header.vrf_output.to_vec()
            } else {
                // Fallback if previous block not found (shouldn't happen in normal operation)
                tracing::warn!("Previous block {} not found for VRF chaining, using hash fallback", current_height);
                prev_hash.as_bytes().to_vec()
            };

            // Generate VRF proof while still holding the read lock to prevent race conditions
            let (vrf_output, vrf_proof) = self.secret_key.vrf_prove(&vrf_input);
            (vrf_output, bincode::serialize(&vrf_proof).unwrap_or_default())
        };

        // Create block header
        let header = BlockHeader {
            height: new_height,
            prev_hash,
            tx_root,
            state_root,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            proposer: winner,
            vrf_output: *vrf_output.as_bytes(),
            vrf_proof: vrf_proof_bytes,
            work: battle_proofs.len() as u64 * 1000, // Simplified work calculation
        };
        
        // Sign the block
        let header_hash = header.hash();
        let signature = self.secret_key.sign(header_hash.as_bytes());
        
        let block = Block {
            header,
            transactions,
            battle_proofs,
            signature,
        };
        
        Ok(block)
    }
    
    /// Validate a block
    pub fn validate_block(&self, block: &Block) -> Result<()> {
        // Check height is sequential
        let current_height = self.height();
        if block.header.height != current_height + 1 {
            return Err(crate::Error::Node(format!(
                "Invalid block height: expected {}, got {}",
                current_height + 1,
                block.header.height
            )));
        }
        
        // Check previous hash matches
        if block.header.prev_hash != self.latest_hash() {
            return Err(crate::Error::Node("Previous hash mismatch".to_string()));
        }
        
        // Verify signature
        let header_hash = block.header.hash();
        if block.signature.verify(&block.header.proposer, header_hash.as_bytes()).is_err() {
            return Err(crate::Error::Node("Invalid block signature".to_string()));
        }

        // Verify VRF proof using proper VRF chaining
        let vrf_proof: bitcell_crypto::VrfProof = bincode::deserialize(&block.header.vrf_proof)
            .map_err(|_| crate::Error::Node("Invalid VRF proof format".to_string()))?;

        // Reconstruct VRF input using the same chaining logic as produce_block
        let vrf_input = if block.header.height == 1 {
            // First block after genesis uses genesis hash as VRF input
            block.header.prev_hash.as_bytes().to_vec()
        } else {
            // Use previous block's VRF output for proper VRF chaining
            let blocks = self.blocks.read().unwrap_or_else(|e| {
                tracing::error!("Lock poisoned in validate_block() - prior panic detected: {}", e);
                e.into_inner()
            });
            if let Some(prev_block) = blocks.get(&(block.header.height - 1)) {
                prev_block.header.vrf_output.to_vec()
            } else {
                return Err(crate::Error::Node(
                    format!("Previous block {} not found for VRF verification", block.header.height - 1)
                ));
            }
        };
        
        let vrf_output = vrf_proof.verify(&block.header.proposer, &vrf_input)
            .map_err(|_| crate::Error::Node("VRF verification failed".to_string()))?;
            
        if vrf_output.as_bytes() != &block.header.vrf_output {
            return Err(crate::Error::Node("VRF output mismatch".to_string()));
        }
        
        // Verify transaction root
        let calculated_tx_root = self.calculate_tx_root(&block.transactions);
        if block.header.tx_root != calculated_tx_root {
            return Err(crate::Error::Node("Transaction root mismatch".to_string()));
        }
        
        // Validate individual transactions
        for tx in &block.transactions {
            self.validate_transaction(tx)?;
        }
        
        Ok(())
    }
    
    /// Add a validated block to the chain
    pub fn add_block(&self, block: Block) -> Result<()> {
        // Validate first
        self.validate_block(&block)?;
        
        let block_height = block.header.height;
        let block_hash = block.hash();
        
        // Apply transactions to state
        {
            let mut state = self.state.write().unwrap_or_else(|e| {
                tracing::error!("Lock poisoned in add_block() while writing state - prior panic detected: {}", e);
                e.into_inner()
            });
            
            // Apply block reward to proposer
            let reward = Self::calculate_block_reward(block_height);
            if reward > 0 {
                match state.credit_account(*block.header.proposer.as_bytes(), reward) {
                    Ok(_) => {
                        tracing::info!("Block reward credited: {} units to proposer", reward);
                    }
                    Err(e) => {
                        tracing::error!("Failed to credit block reward: {:?}", e);
                        return Err(crate::Error::Node("Failed to credit block reward".to_string()));
                    }
                }
            }
            
            for tx in &block.transactions {
                // Apply transaction and update state
                match state.apply_transaction(
                    *tx.from.as_bytes(),
                    *tx.to.as_bytes(),
                    tx.amount,
                    tx.nonce,
                ) {
                    Ok(new_state_root) => {
                        // State updated successfully
                        tracing::debug!("Transaction applied, new state root: {:?}", new_state_root);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to apply transaction: {:?}", e);
                        // In production, this should rollback the entire block
                        // For now, we just skip the transaction
                    }
                }
            }
        }
        
        // Index transactions for O(1) lookup
        {
            let mut tx_index = self.tx_index.write().unwrap_or_else(|e| {
                tracing::error!("Lock poisoned in add_block() while indexing transactions - prior panic detected: {}", e);
                e.into_inner()
            });
            for (idx, tx) in block.transactions.iter().enumerate() {
                tx_index.insert(tx.hash(), TxLocation {
                    block_height,
                    tx_index: idx,
                });
            }
            tracing::debug!("Indexed {} transactions in block {}", block.transactions.len(), block_height);
        }
        
        // Store block
        {
            let mut blocks = self.blocks.write().unwrap_or_else(|e| {
                tracing::error!("Lock poisoned in add_block() while storing block - prior panic detected: {}", e);
                e.into_inner()
            });
            blocks.insert(block_height, block);
        }
        
        // Update chain tip
        {
            let mut height = self.height.write().unwrap_or_else(|e| {
                tracing::error!("Lock poisoned in add_block() while updating height - prior panic detected: {}", e);
                e.into_inner()
            });
            *height = block_height;
        }
        {
            let mut latest_hash = self.latest_hash.write().unwrap_or_else(|e| {
                tracing::error!("Lock poisoned in add_block() while updating latest hash - prior panic detected: {}", e);
                e.into_inner()
            });
            *latest_hash = block_hash;
        }
        
        // Update metrics
        self.metrics.set_chain_height(block_height);
        
        Ok(())
    }
    
    /// Calculate Merkle root of transactions
    fn calculate_tx_root(&self, transactions: &[Transaction]) -> Hash256 {
        if transactions.is_empty() {
            return Hash256::zero();
        }
        
        // Simple hash of all transaction hashes concatenated
        let mut combined = Vec::new();
        for tx in transactions {
            combined.extend_from_slice(tx.hash().as_bytes());
        }
        Hash256::hash(&combined)
    }
    
    /// Validate a single transaction
    fn validate_transaction(&self, tx: &Transaction) -> Result<()> {
        // Verify signature
        let tx_hash = tx.hash();
        if tx.signature.verify(&tx.from, tx_hash.as_bytes()).is_err() {
            return Err(crate::Error::Node("Invalid transaction signature".to_string()));
        }
        
        // Check nonce and balance
        let state = self.state.read().unwrap_or_else(|e| {
            tracing::error!("Lock poisoned in validate_transaction() - prior panic detected: {}", e);
            e.into_inner()
        });
        if let Some(account) = state.get_account(tx.from.as_bytes()) {
            if tx.nonce != account.nonce {
                return Err(crate::Error::Node(format!(
                    "Invalid nonce: expected {}, got {}",
                    account.nonce, tx.nonce
                )));
            }
            
            if tx.amount > account.balance {
                return Err(crate::Error::Node("Insufficient balance".to_string()));
            }
        } else {
            return Err(crate::Error::Node("Account not found".to_string()));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_genesis_block_creation() {
        let sk = Arc::new(SecretKey::generate());
        let metrics = MetricsRegistry::new();
        let blockchain = Blockchain::new(sk, metrics);
        
        assert_eq!(blockchain.height(), GENESIS_HEIGHT);
        assert!(blockchain.get_block(GENESIS_HEIGHT).is_some());
    }
    
    #[test]
    fn test_block_production() {
        let sk = Arc::new(SecretKey::generate());
        let metrics = MetricsRegistry::new();
        let blockchain = Blockchain::new(sk.clone(), metrics);
        
        let block = blockchain.produce_block(
            vec![],
            vec![],
            sk.public_key(),
        ).unwrap();
        
        assert_eq!(block.header.height, 1);
        assert_eq!(block.header.prev_hash, blockchain.latest_hash());
    }
    
    #[test]
    fn test_block_reward_halving() {
        // Test initial reward
        assert_eq!(Blockchain::calculate_block_reward(0), INITIAL_BLOCK_REWARD);
        assert_eq!(Blockchain::calculate_block_reward(1), INITIAL_BLOCK_REWARD);
        
        // Test first halving at 210,000
        assert_eq!(Blockchain::calculate_block_reward(HALVING_INTERVAL - 1), INITIAL_BLOCK_REWARD);
        assert_eq!(Blockchain::calculate_block_reward(HALVING_INTERVAL), INITIAL_BLOCK_REWARD / 2);
        
        // Test second halving at 420,000
        assert_eq!(Blockchain::calculate_block_reward(HALVING_INTERVAL * 2), INITIAL_BLOCK_REWARD / 4);
        
        // Test third halving at 630,000
        assert_eq!(Blockchain::calculate_block_reward(HALVING_INTERVAL * 3), INITIAL_BLOCK_REWARD / 8);
        
        // Test reward becomes 0 after 64 halvings
        assert_eq!(Blockchain::calculate_block_reward(HALVING_INTERVAL * 64), 0);
    }
}
