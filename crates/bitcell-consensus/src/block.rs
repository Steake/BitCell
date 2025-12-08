//! Block structures

use bitcell_crypto::{Hash256, PublicKey, Signature};
use serde::{Deserialize, Serialize};

/// Block header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Block height
    pub height: u64,
    
    /// Previous block hash
    pub prev_hash: Hash256,
    
    /// Merkle root of transactions
    pub tx_root: Hash256,
    
    /// State root
    pub state_root: Hash256,
    
    /// Timestamp (Unix seconds)
    pub timestamp: u64,
    
    /// Tournament winner (block proposer)
    pub proposer: PublicKey,
    
    /// VRF output for this block
    pub vrf_output: [u8; 32],
    
    /// VRF proof
    pub vrf_proof: Vec<u8>, // Serialized VrfProof
    
    /// Block work (deterministic)
    pub work: u64,
}

impl BlockHeader {
    /// Compute hash of header
    pub fn hash(&self) -> Hash256 {
        // Serialize and hash
        // Note: bincode serialization to Vec cannot fail for this structure
        let serialized = bincode::serialize(self).expect("header serialization should never fail");
        Hash256::hash(&serialized)
    }
}

/// Full block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block header
    pub header: BlockHeader,
    
    /// Transactions
    pub transactions: Vec<Transaction>,
    
    /// Battle proofs (one per tournament battle)
    pub battle_proofs: Vec<BattleProof>,
    
    /// Proposer signature
    pub signature: Signature,
}

impl Block {
    /// Get block hash
    pub fn hash(&self) -> Hash256 {
        self.header.hash()
    }
    
    /// Get block height
    pub fn height(&self) -> u64 {
        self.header.height
    }
    
    /// Get block work
    pub fn work(&self) -> u64 {
        self.header.work
    }
}

/// Transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction nonce
    pub nonce: u64,
    
    /// Sender
    pub from: PublicKey,
    
    /// Recipient
    pub to: PublicKey,
    
    /// Amount
    pub amount: u64,
    
    /// Gas limit
    pub gas_limit: u64,
    
    /// Gas price
    pub gas_price: u64,
    
    /// Transaction data
    pub data: Vec<u8>,
    
    /// Signature
    pub signature: Signature,
}

impl Transaction {
    /// Compute transaction hash (includes signature for uniqueness)
    pub fn hash(&self) -> Hash256 {
        // Note: bincode serialization to Vec cannot fail for this structure
        let serialized = bincode::serialize(self).expect("transaction serialization should never fail");
        Hash256::hash(&serialized)
    }
    
    /// Compute hash for signing (excludes signature)
    /// 
    /// This is the message that should be signed when creating a transaction.
    /// The signature field is excluded to avoid circular dependency.
    pub fn signing_hash(&self) -> Hash256 {
        let mut data = Vec::new();
        data.extend_from_slice(&self.nonce.to_le_bytes());
        data.extend_from_slice(self.from.as_bytes());
        data.extend_from_slice(self.to.as_bytes());
        data.extend_from_slice(&self.amount.to_le_bytes());
        data.extend_from_slice(&self.gas_limit.to_le_bytes());
        data.extend_from_slice(&self.gas_price.to_le_bytes());
        data.extend_from_slice(&self.data);
        Hash256::hash(&data)
    }
}

/// Battle proof (placeholder for ZK proof)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleProof {
    /// Battle participants
    pub participant_a: PublicKey,
    pub participant_b: PublicKey,
    
    /// Winner
    pub winner: PublicKey,
    
    /// Proof data (will be actual Groth16 proof)
    pub proof: Vec<u8>,
    
    /// Public inputs
    pub public_inputs: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcell_crypto::SecretKey;

    /// Placeholder signature for tests (before actual signing)
    const PLACEHOLDER_SIGNATURE: [u8; 64] = [0u8; 64];

    #[test]
    fn test_block_header_hash() {
        let sk = SecretKey::generate();
        let header = BlockHeader {
            height: 1,
            prev_hash: Hash256::zero(),
            tx_root: Hash256::zero(),
            state_root: Hash256::zero(),
            timestamp: 1234567890,
            proposer: sk.public_key(),
            vrf_output: [0u8; 32],
            vrf_proof: vec![],
            work: 1000,
        };

        let hash1 = header.hash();
        let hash2 = header.hash();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_transaction_hash() {
        let sk = SecretKey::generate();
        let tx = Transaction {
            nonce: 1,
            from: sk.public_key(),
            to: sk.public_key(),
            amount: 100,
            gas_limit: 21000,
            gas_price: 1,
            data: vec![],
            signature: sk.sign(b"dummy"),
        };

        let hash = tx.hash();
        assert_ne!(hash, Hash256::zero());
    }

    #[test]
    fn test_transaction_signing_hash() {
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        
        // Create transaction with placeholder signature (will be replaced after signing)
        let placeholder_sig = bitcell_crypto::Signature::from_bytes(PLACEHOLDER_SIGNATURE);
        let mut tx = Transaction {
            nonce: 1,
            from: pk.clone(),
            to: pk.clone(),
            amount: 100,
            gas_limit: 21000,
            gas_price: 1000,
            data: vec![],
            signature: placeholder_sig,
        };
        
        // Get signing hash and sign
        let signing_hash = tx.signing_hash();
        let signature = sk.sign(signing_hash.as_bytes());
        tx.signature = signature;
        
        // Verify signature using signing_hash (not full hash)
        assert!(tx.signature.verify(&pk, signing_hash.as_bytes()).is_ok());
        
        // The full hash should be different from signing hash (because it includes signature)
        let full_hash = tx.hash();
        assert_ne!(full_hash, signing_hash);
    }

    #[test]
    fn test_signing_hash_excludes_signature() {
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        
        // Create two identical transactions with different signatures
        let sig1 = sk.sign(b"different1");
        let sig2 = sk.sign(b"different2");
        
        let tx1 = Transaction {
            nonce: 1,
            from: pk.clone(),
            to: pk.clone(),
            amount: 100,
            gas_limit: 21000,
            gas_price: 1000,
            data: vec![],
            signature: sig1,
        };
        
        let tx2 = Transaction {
            nonce: 1,
            from: pk.clone(),
            to: pk.clone(),
            amount: 100,
            gas_limit: 21000,
            gas_price: 1000,
            data: vec![],
            signature: sig2,
        };
        
        // Signing hashes should be identical (signature not included)
        assert_eq!(tx1.signing_hash(), tx2.signing_hash());
        
        // Full hashes should be different (signature included)
        assert_ne!(tx1.hash(), tx2.hash());
    }
}
