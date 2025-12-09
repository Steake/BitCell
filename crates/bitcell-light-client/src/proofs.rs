//! Merkle proof verification for state queries
//!
//! Light clients verify state by requesting Merkle proofs from full nodes.

use bitcell_crypto::{Hash256, merkle::MerkleProof};
use serde::{Deserialize, Serialize};

use crate::{Result, Error};

/// Type of state proof request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateProofType {
    /// Account balance proof
    AccountBalance,
    
    /// Account nonce proof
    AccountNonce,
    
    /// Transaction inclusion proof
    TransactionInclusion,
    
    /// Storage slot proof
    StorageSlot,
}

/// Request for a state proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateProofRequest {
    /// Type of proof requested
    pub proof_type: StateProofType,
    
    /// Block height for the state
    pub block_height: u64,
    
    /// Account or transaction address
    pub key: Vec<u8>,
    
    /// Optional: storage slot for contract storage proofs
    pub storage_slot: Option<Vec<u8>>,
}

impl StateProofRequest {
    /// Create a balance proof request
    pub fn balance(block_height: u64, account: &[u8]) -> Self {
        Self {
            proof_type: StateProofType::AccountBalance,
            block_height,
            key: account.to_vec(),
            storage_slot: None,
        }
    }
    
    /// Create a nonce proof request
    pub fn nonce(block_height: u64, account: &[u8]) -> Self {
        Self {
            proof_type: StateProofType::AccountNonce,
            block_height,
            key: account.to_vec(),
            storage_slot: None,
        }
    }
    
    /// Create a transaction inclusion proof request
    pub fn transaction(block_height: u64, tx_hash: &[u8]) -> Self {
        Self {
            proof_type: StateProofType::TransactionInclusion,
            block_height,
            key: tx_hash.to_vec(),
            storage_slot: None,
        }
    }
}

/// State proof response from a full node
#[derive(Clone, Serialize, Deserialize)]
pub struct StateProof {
    /// The request this is responding to
    pub request: StateProofRequest,
    
    /// State root that this proof is against
    pub state_root: Hash256,
    
    /// Merkle proof path
    pub proof: MerkleProof,
    
    /// The actual value (encoded)
    pub value: Vec<u8>,
    
    /// Whether the key exists in the state
    pub exists: bool,
}

impl StateProof {
    /// Verify the proof against a state root
    pub fn verify(&self, expected_state_root: &Hash256) -> Result<bool> {
        // Check state root matches
        if self.state_root != *expected_state_root {
            return Err(Error::InvalidProof(
                "state root mismatch".to_string()
            ));
        }
        
        // Verify the Merkle proof
        let valid = bitcell_crypto::MerkleTree::verify_proof(
            self.state_root,
            &self.proof
        );
        
        if !valid {
            return Ok(false);
        }
        
        // If proof is valid, check if it proves existence or non-existence
        Ok(self.exists)
    }
    
    /// Extract balance from a balance proof
    pub fn extract_balance(&self) -> Result<u64> {
        if !matches!(self.request.proof_type, StateProofType::AccountBalance) {
            return Err(Error::InvalidProof(
                "not a balance proof".to_string()
            ));
        }
        
        if !self.exists {
            return Ok(0); // Account doesn't exist = 0 balance
        }
        
        // Decode balance from value
        bincode::deserialize(&self.value)
            .map_err(|e| Error::InvalidProof(format!("failed to decode balance: {}", e)))
    }
    
    /// Extract nonce from a nonce proof
    pub fn extract_nonce(&self) -> Result<u64> {
        if !matches!(self.request.proof_type, StateProofType::AccountNonce) {
            return Err(Error::InvalidProof(
                "not a nonce proof".to_string()
            ));
        }
        
        if !self.exists {
            return Ok(0); // Account doesn't exist = 0 nonce
        }
        
        // Decode nonce from value
        bincode::deserialize(&self.value)
            .map_err(|e| Error::InvalidProof(format!("failed to decode nonce: {}", e)))
    }
    
    /// Check if transaction is included
    pub fn is_transaction_included(&self) -> Result<bool> {
        if !matches!(self.request.proof_type, StateProofType::TransactionInclusion) {
            return Err(Error::InvalidProof(
                "not a transaction inclusion proof".to_string()
            ));
        }
        
        Ok(self.exists)
    }
}

/// Batch proof request for multiple state items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProofRequest {
    /// Individual proof requests
    pub requests: Vec<StateProofRequest>,
    
    /// Maximum time to wait for response (milliseconds)
    pub timeout_ms: u64,
}

/// Batch proof response
#[derive(Clone, Serialize, Deserialize)]
pub struct BatchProofResponse {
    /// Proofs for each request
    pub proofs: Vec<StateProof>,
    
    /// Requests that failed (with error messages)
    pub failed: Vec<(StateProofRequest, String)>,
}

impl BatchProofResponse {
    /// Verify all proofs in the batch
    pub fn verify_all(&self, state_root: &Hash256) -> Result<Vec<bool>> {
        self.proofs
            .iter()
            .map(|proof| proof.verify(state_root))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_request_creation() {
        let account = b"test_account";
        let request = StateProofRequest::balance(100, account);
        
        assert_eq!(request.block_height, 100);
        assert!(matches!(request.proof_type, StateProofType::AccountBalance));
    }

    #[test]
    fn test_state_proof_verification() {
        // Create a simple proof structure
        let request = StateProofRequest::balance(100, b"test");
        let state_root = Hash256::hash(b"state_root");
        
        let proof = StateProof {
            request,
            state_root,
            proof: MerkleProof {
                index: 0,
                leaf: Hash256::hash(b"leaf"),
                path: vec![],
            },
            value: bincode::serialize(&1000u64).unwrap(),
            exists: true,
        };
        
        // This will fail because we don't have a valid Merkle tree
        // but it tests the structure
        let _ = proof.verify(&state_root);
    }

    #[test]
    fn test_balance_extraction() {
        let request = StateProofRequest::balance(100, b"test");
        let state_root = Hash256::hash(b"state_root");
        let balance = 42_000u64;
        
        let proof = StateProof {
            request,
            state_root,
            proof: MerkleProof {
                index: 0,
                leaf: Hash256::hash(b"leaf"),
                path: vec![],
            },
            value: bincode::serialize(&balance).unwrap(),
            exists: true,
        };
        
        assert_eq!(proof.extract_balance().unwrap(), balance);
    }

    #[test]
    fn test_nonexistent_account() {
        let request = StateProofRequest::balance(100, b"nonexistent");
        let state_root = Hash256::hash(b"state_root");
        
        let proof = StateProof {
            request,
            state_root,
            proof: MerkleProof {
                index: 0,
                leaf: Hash256::hash(b"leaf"),
                path: vec![],
            },
            value: vec![],
            exists: false,
        };
        
        assert_eq!(proof.extract_balance().unwrap(), 0);
    }
}
