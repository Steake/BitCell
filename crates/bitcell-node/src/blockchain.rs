///! Blockchain manager for block production and validation

use crate::{Result, MetricsRegistry};
use bitcell_consensus::{Block, BlockHeader, Transaction, BattleProof};
use bitcell_crypto::{Hash256, PublicKey, SecretKey};
use bitcell_state::StateManager;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

/// Genesis block height
pub const GENESIS_HEIGHT: u64 = 0;

/// Blockchain manager
#[derive(Clone)]
pub struct Blockchain {
    /// Current chain height
    height: Arc<RwLock<u64>>,
    
    /// Latest block hash
    latest_hash: Arc<RwLock<Hash256>>,
    
    /// Block storage (height -> block)
    blocks: Arc<RwLock<HashMap<u64, Block>>>,
    
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
    pub fn height(&self) -> u64 {
        *self.height.read().unwrap()
    }
    
    /// Get latest block hash
    pub fn latest_hash(&self) -> Hash256 {
        *self.latest_hash.read().unwrap()
    }
    
    /// Get block by height
    pub fn get_block(&self, height: u64) -> Option<Block> {
        self.blocks.read().unwrap().get(&height).cloned()
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
            let state = self.state.read().unwrap();
            state.state_root
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
            vrf_output: [0u8; 32], // TODO: Implement VRF
            vrf_proof: vec![],
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
            let mut state = self.state.write().unwrap();
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
                        println!("Transaction applied, new state root: {:?}", new_state_root);
                    }
                    Err(e) => {
                        println!("Failed to apply transaction: {:?}", e);
                        // In production, this should rollback the entire block
                        // For now, we just skip the transaction
                    }
                }
            }
        }
        
        // Store block
        {
            let mut blocks = self.blocks.write().unwrap();
            blocks.insert(block_height, block);
        }
        
        // Update chain tip
        {
            let mut height = self.height.write().unwrap();
            *height = block_height;
        }
        {
            let mut latest_hash = self.latest_hash.write().unwrap();
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
        let state = self.state.read().unwrap();
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
        let sk = SecretKey::generate();
        let metrics = MetricsRegistry::new();
        let blockchain = Blockchain::new(sk, metrics);
        
        assert_eq!(blockchain.height(), GENESIS_HEIGHT);
        assert!(blockchain.get_block(GENESIS_HEIGHT).is_some());
    }
    
    #[test]
    fn test_block_production() {
        let sk = SecretKey::generate();
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
}
