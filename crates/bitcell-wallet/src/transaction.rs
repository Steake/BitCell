//! Transaction Handling
//!
//! Provides transaction creation, signing, and management for multiple chains.

use crate::{Address, Chain, Error, Result};
use bitcell_crypto::{Hash256, PublicKey, SecretKey, Signature};
use serde::{Deserialize, Serialize};

/// Transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Transaction is pending
    Pending,
    /// Transaction is confirmed
    Confirmed,
    /// Transaction failed
    Failed,
    /// Transaction was dropped
    Dropped,
}

/// Raw unsigned transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Chain this transaction is for
    pub chain: Chain,
    /// Sender address
    pub from: String,
    /// Recipient address
    pub to: String,
    /// Amount in smallest units
    pub amount: u64,
    /// Fee in smallest units
    pub fee: u64,
    /// Nonce (for replay protection)
    pub nonce: u64,
    /// Transaction data/memo (optional)
    pub data: Vec<u8>,
    /// Timestamp (Unix epoch)
    pub timestamp: u64,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        chain: Chain,
        from: String,
        to: String,
        amount: u64,
        fee: u64,
        nonce: u64,
    ) -> Self {
        Self {
            chain,
            from,
            to,
            amount,
            fee,
            nonce,
            data: Vec::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Add transaction data/memo
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    /// Get the transaction hash (for signing)
    pub fn hash(&self) -> Hash256 {
        let mut data = Vec::new();
        data.extend_from_slice(&(self.chain.chain_id() as u64).to_le_bytes());
        data.extend_from_slice(self.from.as_bytes());
        data.extend_from_slice(self.to.as_bytes());
        data.extend_from_slice(&self.amount.to_le_bytes());
        data.extend_from_slice(&self.fee.to_le_bytes());
        data.extend_from_slice(&self.nonce.to_le_bytes());
        data.extend_from_slice(&self.data);
        data.extend_from_slice(&self.timestamp.to_le_bytes());
        Hash256::hash(&data)
    }

    /// Calculate total cost (amount + fee)
    pub fn total_cost(&self) -> u64 {
        self.amount.saturating_add(self.fee)
    }

    /// Sign the transaction
    pub fn sign(&self, secret_key: &SecretKey) -> SignedTransaction {
        let hash = self.hash();
        let signature = secret_key.sign(hash.as_bytes());
        SignedTransaction {
            transaction: self.clone(),
            signature,
            tx_hash: hash,
        }
    }
}

/// Signed transaction ready for broadcast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    /// The underlying transaction
    pub transaction: Transaction,
    /// Signature
    pub signature: Signature,
    /// Transaction hash
    pub tx_hash: Hash256,
}

impl SignedTransaction {
    /// Verify the signature
    pub fn verify(&self, public_key: &PublicKey) -> Result<()> {
        let hash = self.transaction.hash();
        self.signature.verify(public_key, hash.as_bytes())
            .map_err(|_| Error::SigningError("Invalid signature".into()))
    }

    /// Get the transaction hash
    pub fn hash(&self) -> &Hash256 {
        &self.tx_hash
    }

    /// Get the transaction hash as hex string
    pub fn hash_hex(&self) -> String {
        hex::encode(self.tx_hash.as_bytes())
    }

    /// Serialize for broadcast
    pub fn serialize(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| Error::Serialization(e.to_string()))
    }

    /// Deserialize from bytes
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data)
            .map_err(|e| Error::Serialization(e.to_string()))
    }
}

/// Transaction builder for easier transaction creation
#[derive(Debug, Clone)]
pub struct TransactionBuilder {
    chain: Chain,
    from: Option<String>,
    to: Option<String>,
    amount: u64,
    fee: u64,
    nonce: u64,
    data: Vec<u8>,
}

impl TransactionBuilder {
    /// Create a new transaction builder
    pub fn new(chain: Chain) -> Self {
        Self {
            chain,
            from: None,
            to: None,
            amount: 0,
            fee: 0,
            nonce: 0,
            data: Vec::new(),
        }
    }

    /// Set the sender address
    pub fn from(mut self, address: &Address) -> Self {
        self.from = Some(address.to_string_formatted());
        self
    }

    /// Set the sender address from string
    pub fn from_str(mut self, address: &str) -> Self {
        self.from = Some(address.to_string());
        self
    }

    /// Set the recipient address
    pub fn to(mut self, address: &Address) -> Self {
        self.to = Some(address.to_string_formatted());
        self
    }

    /// Set the recipient address from string
    pub fn to_str(mut self, address: &str) -> Self {
        self.to = Some(address.to_string());
        self
    }

    /// Set the amount
    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = amount;
        self
    }

    /// Set the fee
    pub fn fee(mut self, fee: u64) -> Self {
        self.fee = fee;
        self
    }

    /// Set the nonce
    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = nonce;
        self
    }

    /// Set transaction data
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    /// Build the transaction
    pub fn build(self) -> Result<Transaction> {
        let from = self.from.ok_or(Error::TransactionError("Missing sender address".into()))?;
        let to = self.to.ok_or(Error::TransactionError("Missing recipient address".into()))?;
        
        if self.amount == 0 {
            return Err(Error::TransactionError("Amount must be greater than 0".into()));
        }
        
        Ok(Transaction {
            chain: self.chain,
            from,
            to,
            amount: self.amount,
            fee: self.fee,
            nonce: self.nonce,
            data: self.data,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }
}

/// Fee estimator for transactions
#[derive(Debug, Clone)]
pub struct FeeEstimator {
    /// Base fee per byte
    base_fee_per_byte: u64,
    /// Priority fee multiplier
    priority_multiplier: f64,
}

impl FeeEstimator {
    /// Create a new fee estimator
    pub fn new(chain: Chain) -> Self {
        let base_fee = match chain {
            Chain::BitCell => 1,
            Chain::Bitcoin | Chain::BitcoinTestnet => 10,
            Chain::Ethereum | Chain::EthereumSepolia => 21_000_000_000, // 21 Gwei base
            Chain::Custom(_) => 1,
        };
        
        Self {
            base_fee_per_byte: base_fee,
            priority_multiplier: 1.0,
        }
    }

    /// Set priority multiplier (1.0 = normal, 2.0 = fast, 0.5 = slow)
    pub fn with_priority(mut self, multiplier: f64) -> Self {
        self.priority_multiplier = multiplier.max(0.1).min(10.0);
        self
    }

    /// Estimate fee for a transaction
    pub fn estimate(&self, tx: &Transaction) -> u64 {
        // Estimate transaction size
        let base_size = 100; // Base overhead
        let data_size = tx.data.len();
        let total_size = base_size + data_size;
        
        let base_fee = (total_size as u64) * self.base_fee_per_byte;
        (base_fee as f64 * self.priority_multiplier) as u64
    }

    /// Estimate fee for simple transfer
    pub fn estimate_simple_transfer(&self, chain: Chain) -> u64 {
        let base_fee: u64 = match chain {
            Chain::BitCell => 100,
            Chain::Bitcoin | Chain::BitcoinTestnet => 2000, // ~200 bytes * 10 sat/byte
            Chain::Ethereum | Chain::EthereumSepolia => 21_000_u64 * 20_000_000_000_u64, // 21k gas * 20 Gwei
            Chain::Custom(_) => 100,
        };
        
        (base_fee as f64 * self.priority_multiplier) as u64
    }
}

impl Default for FeeEstimator {
    fn default() -> Self {
        Self::new(Chain::BitCell)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction::new(
            Chain::BitCell,
            "BC1abc123".to_string(),
            "BC1def456".to_string(),
            100_000,
            100,
            1,
        );
        
        assert_eq!(tx.amount, 100_000);
        assert_eq!(tx.fee, 100);
        assert_eq!(tx.nonce, 1);
        assert_eq!(tx.total_cost(), 100_100);
    }

    #[test]
    fn test_transaction_hash() {
        let tx = Transaction::new(
            Chain::BitCell,
            "BC1abc".to_string(),
            "BC1def".to_string(),
            1000,
            10,
            0,
        );
        
        let hash1 = tx.hash();
        let hash2 = tx.hash();
        
        // Same transaction should have same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_transaction_signing() {
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        
        let tx = Transaction::new(
            Chain::BitCell,
            "BC1abc".to_string(),
            "BC1def".to_string(),
            1000,
            10,
            0,
        );
        
        let signed = tx.sign(&sk);
        
        // Verify should succeed
        assert!(signed.verify(&pk).is_ok());
    }

    #[test]
    fn test_signed_transaction_wrong_key() {
        let sk1 = SecretKey::generate();
        let sk2 = SecretKey::generate();
        let pk2 = sk2.public_key();
        
        let tx = Transaction::new(
            Chain::BitCell,
            "BC1abc".to_string(),
            "BC1def".to_string(),
            1000,
            10,
            0,
        );
        
        let signed = tx.sign(&sk1);
        
        // Verify with wrong key should fail
        assert!(signed.verify(&pk2).is_err());
    }

    #[test]
    fn test_transaction_builder() {
        let tx = TransactionBuilder::new(Chain::BitCell)
            .from_str("BC1abc")
            .to_str("BC1def")
            .amount(1000)
            .fee(10)
            .nonce(1)
            .build()
            .unwrap();
        
        assert_eq!(tx.amount, 1000);
        assert_eq!(tx.chain, Chain::BitCell);
    }

    #[test]
    fn test_transaction_builder_missing_fields() {
        let result = TransactionBuilder::new(Chain::BitCell)
            .amount(1000)
            .build();
        
        assert!(result.is_err());
    }

    #[test]
    fn test_transaction_builder_zero_amount() {
        let result = TransactionBuilder::new(Chain::BitCell)
            .from_str("BC1abc")
            .to_str("BC1def")
            .amount(0)
            .build();
        
        assert!(result.is_err());
    }

    #[test]
    fn test_fee_estimator() {
        let estimator = FeeEstimator::new(Chain::BitCell);
        let fee = estimator.estimate_simple_transfer(Chain::BitCell);
        
        assert!(fee > 0);
    }

    #[test]
    fn test_fee_estimator_priority() {
        let normal = FeeEstimator::new(Chain::BitCell);
        let fast = FeeEstimator::new(Chain::BitCell).with_priority(2.0);
        
        let normal_fee = normal.estimate_simple_transfer(Chain::BitCell);
        let fast_fee = fast.estimate_simple_transfer(Chain::BitCell);
        
        assert!(fast_fee > normal_fee);
    }

    #[test]
    fn test_signed_transaction_serialization() {
        let sk = SecretKey::generate();
        let tx = Transaction::new(
            Chain::BitCell,
            "BC1abc".to_string(),
            "BC1def".to_string(),
            1000,
            10,
            0,
        );
        
        let signed = tx.sign(&sk);
        let serialized = signed.serialize().unwrap();
        let deserialized = SignedTransaction::deserialize(&serialized).unwrap();
        
        assert_eq!(signed.hash(), deserialized.hash());
    }

    #[test]
    fn test_transaction_with_data() {
        let tx = Transaction::new(
            Chain::BitCell,
            "BC1abc".to_string(),
            "BC1def".to_string(),
            1000,
            10,
            0,
        ).with_data(b"Hello, World!".to_vec());
        
        assert_eq!(tx.data, b"Hello, World!");
    }
}
