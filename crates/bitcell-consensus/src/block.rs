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
    
    /// Aggregated proof commitment (32 bytes)
    /// SHA-256 hash of all battle and state proofs in block
    pub aggregation_commitment: [u8; 32],
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
    
    /// State transition proofs
    pub state_proofs: Vec<StateProof>,
    
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
    /// Compute transaction hash
    pub fn hash(&self) -> Hash256 {
        // Note: bincode serialization to Vec cannot fail for this structure
        let serialized = bincode::serialize(self).expect("transaction serialization should never fail");
        Hash256::hash(&serialized)
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

/// State transition proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateProof {
    /// Old state root
    pub old_root: [u8; 32],
    
    /// New state root
    pub new_root: [u8; 32],
    
    /// Nullifier
    ///
    /// The nullifier is a unique identifier for this state transition, typically computed as a hash
    /// of secret data and/or the input state. Its primary purpose is to prevent double-spending or
    /// replay of the same state transition by ensuring that each transition can only be used once.
    /// Even if the `old_root` and `new_root` are known, without a nullifier, an adversary could
    /// potentially reuse a valid proof. The nullifier allows the system to track which transitions
    /// have already been applied, ensuring uniqueness and preventing double-use.
    pub nullifier: [u8; 32],
    
    /// Proof data (Groth16 proof)
    pub proof: Vec<u8>,
    
    /// Public inputs
    pub public_inputs: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcell_crypto::SecretKey;

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
            aggregation_commitment: [0u8; 32],
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
}
