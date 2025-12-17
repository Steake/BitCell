//! Guardian multi-sig controls for emergency governance

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use bitcell_crypto::{PublicKey, Signature};
use crate::proposal::ProposalId;

/// Guardian public key and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guardian {
    /// Guardian's public key
    pub pubkey: [u8; 33],
    
    /// Guardian name/identifier
    pub name: String,
    
    /// When guardian was added
    pub added_at: u64,
}

/// Guardian emergency actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuardianAction {
    /// Cancel a proposal immediately
    Cancel,
    
    /// Execute a proposal immediately (bypass timelock)
    ExecuteImmediately,
}

/// Set of guardians with multi-sig capabilities
pub struct GuardianSet {
    /// Active guardians
    guardians: HashSet<[u8; 33]>,
    
    /// Guardian metadata
    guardian_info: Vec<Guardian>,
}

impl GuardianSet {
    /// Create empty guardian set
    pub fn new() -> Self {
        Self {
            guardians: HashSet::new(),
            guardian_info: Vec::new(),
        }
    }
    
    /// Create with initial guardians
    pub fn with_guardians(guardians: Vec<Guardian>) -> Self {
        let guardian_set: HashSet<[u8; 33]> = guardians.iter()
            .map(|g| g.pubkey)
            .collect();
        
        Self {
            guardians: guardian_set,
            guardian_info: guardians,
        }
    }
    
    /// Add a guardian
    pub fn add_guardian(&mut self, guardian: Guardian) -> crate::Result<()> {
        self.guardians.insert(guardian.pubkey);
        self.guardian_info.push(guardian);
        Ok(())
    }
    
    /// Remove a guardian
    pub fn remove_guardian(&mut self, pubkey: &[u8; 33]) -> crate::Result<()> {
        self.guardians.remove(pubkey);
        self.guardian_info.retain(|g| &g.pubkey != pubkey);
        Ok(())
    }
    
    /// Check if an address is a guardian
    pub fn is_guardian(&self, pubkey: &[u8; 33]) -> bool {
        self.guardians.contains(pubkey)
    }
    
    /// Get total number of guardians
    pub fn count(&self) -> usize {
        self.guardians.len()
    }
    
    /// Verify guardian signatures on a proposal action
    /// Returns the number of valid signatures
    pub fn verify_signatures(
        &self,
        proposal_id: &ProposalId,
        signatures: &[[u8; 64]],
    ) -> crate::Result<usize> {
        let mut valid_count = 0;
        let mut signed_guardians = HashSet::new();
        
        // Message to sign is the proposal ID
        let message = &proposal_id.0;
        
        for sig_bytes in signatures {
            // Try to verify with each guardian's key
            for guardian in &self.guardian_info {
                // Skip if this guardian already signed
                if signed_guardians.contains(&guardian.pubkey) {
                    continue;
                }
                
                // Create PublicKey and Signature from bytes
                let pubkey = match PublicKey::from_bytes(&guardian.pubkey) {
                    Ok(pk) => pk,
                    Err(_) => continue,
                };
                
                let signature = match Signature::from_bytes(sig_bytes) {
                    Ok(sig) => sig,
                    Err(_) => continue,
                };
                
                // Verify signature
                if pubkey.verify(message, &signature).is_ok() {
                    signed_guardians.insert(guardian.pubkey);
                    valid_count += 1;
                    break;
                }
            }
        }
        
        tracing::info!(
            proposal_id = %hex::encode(&proposal_id.0),
            valid_signatures = valid_count,
            total_signatures = signatures.len(),
            "Guardian signatures verified"
        );
        
        Ok(valid_count)
    }
    
    /// Get all guardians
    pub fn get_guardians(&self) -> &[Guardian] {
        &self.guardian_info
    }
}

impl Default for GuardianSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_guardian_set() {
        let mut set = GuardianSet::new();
        
        let guardian1 = Guardian {
            pubkey: [1u8; 33],
            name: "Guardian 1".to_string(),
            added_at: 1000,
        };
        
        let guardian2 = Guardian {
            pubkey: [2u8; 33],
            name: "Guardian 2".to_string(),
            added_at: 1000,
        };
        
        set.add_guardian(guardian1.clone()).unwrap();
        set.add_guardian(guardian2.clone()).unwrap();
        
        assert_eq!(set.count(), 2);
        assert!(set.is_guardian(&[1u8; 33]));
        assert!(set.is_guardian(&[2u8; 33]));
        assert!(!set.is_guardian(&[3u8; 33]));
    }
    
    #[test]
    fn test_remove_guardian() {
        let mut set = GuardianSet::new();
        
        let guardian = Guardian {
            pubkey: [1u8; 33],
            name: "Guardian".to_string(),
            added_at: 1000,
        };
        
        set.add_guardian(guardian).unwrap();
        assert_eq!(set.count(), 1);
        
        set.remove_guardian(&[1u8; 33]).unwrap();
        assert_eq!(set.count(), 0);
        assert!(!set.is_guardian(&[1u8; 33]));
    }
    
    #[test]
    fn test_guardian_with_initial() {
        let guardians = vec![
            Guardian {
                pubkey: [1u8; 33],
                name: "G1".to_string(),
                added_at: 1000,
            },
            Guardian {
                pubkey: [2u8; 33],
                name: "G2".to_string(),
                added_at: 1000,
            },
        ];
        
        let set = GuardianSet::with_guardians(guardians);
        assert_eq!(set.count(), 2);
    }
}
