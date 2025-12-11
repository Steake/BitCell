//! Light wallet mode for balance queries and transaction submission
//!
//! Provides wallet functionality using only header chain and state proofs.

use bitcell_consensus::Transaction;
use bitcell_crypto::{Hash256, PublicKey, SecretKey, Signature};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    Result, Error, HeaderChain, StateProofRequest, StateProof,
    LightClientProtocol, LightClientMessage,
};

/// Wallet operating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalletMode {
    /// Read-only mode (balance queries only)
    ReadOnly,
    
    /// Full mode (can sign and submit transactions)
    Full,
}

/// Account information cached from proofs
#[derive(Debug, Clone)]
pub struct AccountInfo {
    /// Account balance
    pub balance: u64,
    
    /// Account nonce
    pub nonce: u64,
    
    /// Last updated at block height
    pub last_updated: u64,
}

/// Light wallet implementation
///
/// Provides wallet functionality without requiring full blockchain state.
/// Uses Merkle proofs to verify account balances and state.
pub struct LightWallet {
    /// Wallet mode
    mode: WalletMode,
    
    /// Public key (address)
    public_key: PublicKey,
    
    /// Secret key (if in Full mode)
    secret_key: Option<Arc<SecretKey>>,
    
    /// Header chain for state root verification
    header_chain: Arc<HeaderChain>,
    
    /// Protocol handler for network communication (reserved for future use)
    #[allow(dead_code)]
    protocol: Arc<LightClientProtocol>,
    
    /// Cached account info
    account_cache: Arc<RwLock<HashMap<PublicKey, AccountInfo>>>,
    
    /// Pending transactions
    pending_txs: Arc<RwLock<Vec<Hash256>>>,
}

impl LightWallet {
    /// Create a new read-only wallet
    pub fn read_only(
        public_key: PublicKey,
        header_chain: Arc<HeaderChain>,
        protocol: Arc<LightClientProtocol>,
    ) -> Self {
        Self {
            mode: WalletMode::ReadOnly,
            public_key,
            secret_key: None,
            header_chain,
            protocol,
            account_cache: Arc::new(RwLock::new(HashMap::new())),
            pending_txs: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Create a new full wallet with signing capability
    pub fn full(
        secret_key: Arc<SecretKey>,
        header_chain: Arc<HeaderChain>,
        protocol: Arc<LightClientProtocol>,
    ) -> Self {
        let public_key = secret_key.public_key();
        
        Self {
            mode: WalletMode::Full,
            public_key,
            secret_key: Some(secret_key),
            header_chain,
            protocol,
            account_cache: Arc::new(RwLock::new(HashMap::new())),
            pending_txs: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Get wallet address
    pub fn address(&self) -> &PublicKey {
        &self.public_key
    }
    
    /// Get wallet mode
    pub fn mode(&self) -> WalletMode {
        self.mode
    }
    
    /// Query balance via state proof
    pub async fn get_balance(&self) -> Result<u64> {
        self.get_balance_for(&self.public_key).await
    }
    
    /// Query balance for any account via state proof
    pub async fn get_balance_for(&self, account: &PublicKey) -> Result<u64> {
        // Check cache first
        if let Some(info) = self.account_cache.read().get(account) {
            // Cache is valid if it's from a recent block
            let tip = self.header_chain.tip_height();
            if tip.saturating_sub(info.last_updated) < 10 {
                return Ok(info.balance);
            }
        }
        
        // Need to request proof from full node
        let tip_height = self.header_chain.tip_height();
        let _request = StateProofRequest::balance(tip_height, account.as_bytes());
        
        // In a real implementation, this would send the request over the network
        // For now, we return an error indicating network communication is needed
        Err(Error::NetworkError(
            "network proof request not implemented - requires full node connection".to_string()
        ))
    }
    
    /// Update account info from a state proof
    pub fn update_from_proof(&self, proof: StateProof) -> Result<()> {
        // Verify proof against header chain
        let header = self.header_chain.get_header(proof.request.block_height)
            .ok_or_else(|| Error::InvalidProof("block not in header chain".to_string()))?;
        
        // Verify proof against state root
        proof.verify(&header.state_root)?;
        
        // Extract account info based on proof type
        if let Ok(balance) = proof.extract_balance() {
            let mut key_bytes = [0u8; 33];
            if proof.request.key.len() != 33 {
                return Err(Error::InvalidProof("invalid key length for public key".to_string()));
            }
            key_bytes.copy_from_slice(&proof.request.key);
            let account_key = PublicKey::from_bytes(key_bytes)?;
            
            let mut cache = self.account_cache.write();
            let info = cache.entry(account_key).or_insert(AccountInfo {
                balance: 0,
                nonce: 0,
                last_updated: 0,
            });
            
            info.balance = balance;
            // Try to extract and update nonce if available
            if let Ok(nonce) = proof.extract_nonce() {
                info.nonce = nonce;
            }
            info.last_updated = proof.request.block_height;
        }
        
        Ok(())
    }
    
    /// Get nonce for account
    pub async fn get_nonce(&self) -> Result<u64> {
        // Check cache
        if let Some(info) = self.account_cache.read().get(&self.public_key) {
            let tip = self.header_chain.tip_height();
            if tip.saturating_sub(info.last_updated) < 10 {
                return Ok(info.nonce);
            }
        }
        
        // Would request from network in real implementation
        Err(Error::NetworkError(
            "network nonce request not implemented".to_string()
        ))
    }
    
    /// Create and sign a transaction
    pub fn create_transaction(
        &self,
        to: PublicKey,
        amount: u64,
        nonce: u64,
        gas_limit: u64,
        gas_price: u64,
    ) -> Result<Transaction> {
        if self.mode != WalletMode::Full {
            return Err(Error::WalletError("wallet is read-only".to_string()));
        }
        
        let secret_key = self.secret_key.as_ref()
            .ok_or_else(|| Error::WalletError("no secret key available".to_string()))?;
        
        // Create transaction with placeholder signature
        let tx = Transaction {
            nonce,
            from: self.public_key,
            to,
            amount,
            gas_limit,
            gas_price,
            data: vec![],
            signature: Signature::from_bytes([0u8; 64]),
        };
        
        // Sign transaction
        let tx_hash = Hash256::hash(&bincode::serialize(&tx)?);
        let signature = secret_key.sign(tx_hash.as_bytes());
        
        let signed_tx = Transaction {
            signature,
            ..tx
        };
        
        Ok(signed_tx)
    }
    
    /// Submit a transaction to the network
    pub async fn submit_transaction(&self, tx: Transaction) -> Result<Hash256> {
        if self.mode != WalletMode::Full {
            return Err(Error::WalletError("wallet is read-only".to_string()));
        }
        
        // Serialize transaction
        let tx_data = bincode::serialize(&tx)?;
        let tx_hash = Hash256::hash(&tx_data);
        
        // Create submit message
        let _message = LightClientMessage::SubmitTransaction(tx_data);
        
        // In real implementation, would send over network
        // For now, add to pending list
        self.pending_txs.write().push(tx_hash);
        
        Ok(tx_hash)
    }
    
    /// Get pending transactions
    pub fn pending_transactions(&self) -> Vec<Hash256> {
        self.pending_txs.read().clone()
    }
    
    /// Clear pending transactions
    pub fn clear_pending(&self) {
        self.pending_txs.write().clear();
    }
    
    /// Get cached account info
    pub fn cached_account_info(&self, account: &PublicKey) -> Option<AccountInfo> {
        self.account_cache.read().get(account).cloned()
    }
    
    /// Estimate memory usage
    pub fn memory_usage(&self) -> usize {
        // Account cache: ~100 bytes per entry
        let cache_size = self.account_cache.read().len() * 100;
        
        // Pending txs: 32 bytes per hash
        let pending_size = self.pending_txs.read().len() * 32;
        
        cache_size + pending_size + 1000 // Base overhead
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HeaderChainConfig};
    use bitcell_consensus::BlockHeader;
    use bitcell_crypto::{Hash256, SecretKey};

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

    #[test]
    fn test_wallet_creation() {
        let sk = Arc::new(SecretKey::generate());
        let genesis = create_genesis();
        let chain = Arc::new(HeaderChain::new(genesis, HeaderChainConfig::default()));
        let protocol = Arc::new(LightClientProtocol::new());
        
        let wallet = LightWallet::full(sk.clone(), chain, protocol);
        
        assert_eq!(wallet.mode(), WalletMode::Full);
        assert_eq!(wallet.address(), &sk.public_key());
    }

    #[test]
    fn test_readonly_wallet() {
        let pk = SecretKey::generate().public_key();
        let genesis = create_genesis();
        let chain = Arc::new(HeaderChain::new(genesis, HeaderChainConfig::default()));
        let protocol = Arc::new(LightClientProtocol::new());
        
        let wallet = LightWallet::read_only(pk, chain, protocol);
        
        assert_eq!(wallet.mode(), WalletMode::ReadOnly);
        assert_eq!(wallet.address(), &pk);
    }

    #[test]
    fn test_transaction_creation() {
        let sk = Arc::new(SecretKey::generate());
        let genesis = create_genesis();
        let chain = Arc::new(HeaderChain::new(genesis, HeaderChainConfig::default()));
        let protocol = Arc::new(LightClientProtocol::new());
        
        let wallet = LightWallet::full(sk, chain, protocol);
        
        let to = SecretKey::generate().public_key();
        let tx = wallet.create_transaction(to, 1000, 0, 21000, 1).unwrap();
        
        assert_eq!(tx.amount, 1000);
        assert_eq!(tx.nonce, 0);
    }

    #[test]
    fn test_readonly_cannot_sign() {
        let pk = SecretKey::generate().public_key();
        let genesis = create_genesis();
        let chain = Arc::new(HeaderChain::new(genesis, HeaderChainConfig::default()));
        let protocol = Arc::new(LightClientProtocol::new());
        
        let wallet = LightWallet::read_only(pk, chain, protocol);
        
        let to = SecretKey::generate().public_key();
        let result = wallet.create_transaction(to, 1000, 0, 21000, 1);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_usage() {
        let sk = Arc::new(SecretKey::generate());
        let genesis = create_genesis();
        let chain = Arc::new(HeaderChain::new(genesis, HeaderChainConfig::default()));
        let protocol = Arc::new(LightClientProtocol::new());
        
        let wallet = LightWallet::full(sk, chain, protocol);
        
        let usage = wallet.memory_usage();
        assert!(usage > 0);
        assert!(usage < 10_000); // Should be very small initially
    }
}
