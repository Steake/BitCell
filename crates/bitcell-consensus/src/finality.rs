//! Finality Gadget: BFT-style finality for BitCell blocks
//!
//! Implements a Byzantine Fault Tolerant finality mechanism inspired by GRANDPA/Tendermint:
//! - Validators vote on blocks (prevote, precommit)
//! - 2/3+ stake threshold required for finality
//! - Finalized blocks are irreversible
//! - Equivocation (double-signing) triggers slashing
//! - Target: <1 minute finality time

use bitcell_crypto::{Hash256, PublicKey, Signature};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vote type in the finality protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum VoteType {
    /// First round vote (prevote)
    Prevote,
    /// Second round vote (precommit)
    Precommit,
}

/// A finality vote from a validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalityVote {
    /// Block hash being voted on
    pub block_hash: Hash256,
    
    /// Block height
    pub block_height: u64,
    
    /// Type of vote
    pub vote_type: VoteType,
    
    /// Voting round number (for handling network delays)
    pub round: u64,
    
    /// Validator public key
    pub validator: PublicKey,
    
    /// Signature over (block_hash, block_height, vote_type, round)
    pub signature: Signature,
}

impl FinalityVote {
    /// Create message to sign for this vote
    pub fn sign_message(&self) -> Vec<u8> {
        let mut msg = Vec::new();
        msg.extend_from_slice(self.block_hash.as_bytes());
        msg.extend_from_slice(&self.block_height.to_le_bytes());
        msg.push(match self.vote_type {
            VoteType::Prevote => 0,
            VoteType::Precommit => 1,
        });
        msg.extend_from_slice(&self.round.to_le_bytes());
        msg
    }
    
    /// Verify this vote's signature
    pub fn verify(&self) -> bool {
        let msg = self.sign_message();
        self.signature.verify(&self.validator, &msg).is_ok()
    }
}

/// Evidence of equivocation (double-signing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivocationEvidence {
    /// First conflicting vote
    pub vote1: FinalityVote,
    
    /// Second conflicting vote (different block, same height/round/type)
    pub vote2: FinalityVote,
    
    /// Height where evidence was submitted
    pub evidence_height: u64,
}

impl EquivocationEvidence {
    /// Validate that this is valid equivocation evidence
    pub fn is_valid(&self) -> bool {
        // Must be from same validator
        if self.vote1.validator != self.vote2.validator {
            return false;
        }
        
        // Must be for same height
        if self.vote1.block_height != self.vote2.block_height {
            return false;
        }
        
        // Must be for same round
        if self.vote1.round != self.vote2.round {
            return false;
        }
        
        // Must be same vote type
        if self.vote1.vote_type != self.vote2.vote_type {
            return false;
        }
        
        // Must be for different blocks (the actual equivocation)
        if self.vote1.block_hash == self.vote2.block_hash {
            return false;
        }
        
        // Both signatures must be valid
        if !self.vote1.verify() || !self.vote2.verify() {
            return false;
        }
        
        true
    }
}

/// Status of a block's finality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinalityStatus {
    /// Block is pending finality
    Pending,
    
    /// Block has received 2/3+ prevotes
    Prevoted,
    
    /// Block has received 2/3+ precommits (finalized)
    Finalized,
}

impl Default for FinalityStatus {
    fn default() -> Self {
        FinalityStatus::Pending
    }
}

/// Vote tracker for a specific block
#[derive(Debug, Clone)]
struct VoteTracker {
    /// Prevotes received (validator -> signature)
    prevotes: HashMap<PublicKey, Signature>,
    
    /// Precommits received (validator -> signature)
    precommits: HashMap<PublicKey, Signature>,
    
    /// Total stake that prevoted
    prevote_stake: u64,
    
    /// Total stake that precommitted
    precommit_stake: u64,
}

impl VoteTracker {
    fn new() -> Self {
        Self {
            prevotes: HashMap::new(),
            precommits: HashMap::new(),
            prevote_stake: 0,
            precommit_stake: 0,
        }
    }
}

/// The finality gadget - tracks votes and determines finality
pub struct FinalityGadget {
    /// Current round number
    current_round: u64,
    
    /// Votes per block (block_hash -> tracker)
    vote_trackers: HashMap<Hash256, VoteTracker>,
    
    /// Finality status per block
    finality_status: HashMap<Hash256, FinalityStatus>,
    
    /// Validator set with stakes (validator -> stake)
    validator_stakes: HashMap<PublicKey, u64>,
    
    /// Total stake in validator set
    total_stake: u64,
    
    /// Detected equivocations (validator -> evidence)
    equivocations: HashMap<PublicKey, Vec<EquivocationEvidence>>,
    
    /// Vote history for equivocation detection
    /// (height, round, vote_type, validator) -> block_hash
    vote_history: HashMap<(u64, u64, VoteType, PublicKey), Hash256>,
}

impl FinalityGadget {
    /// Create a new finality gadget with validator set
    pub fn new(validator_stakes: HashMap<PublicKey, u64>) -> Self {
        let total_stake: u64 = validator_stakes.values().sum();
        
        Self {
            current_round: 0,
            vote_trackers: HashMap::new(),
            finality_status: HashMap::new(),
            validator_stakes,
            total_stake,
            equivocations: HashMap::new(),
            vote_history: HashMap::new(),
        }
    }
    
    /// Update validator set (called at epoch boundaries)
    pub fn update_validators(&mut self, validator_stakes: HashMap<PublicKey, u64>) {
        self.validator_stakes = validator_stakes;
        self.total_stake = self.validator_stakes.values().sum();
    }
    
    /// Get current finality status for a block
    pub fn get_finality_status(&self, block_hash: &Hash256) -> FinalityStatus {
        self.finality_status.get(block_hash)
            .copied()
            .unwrap_or(FinalityStatus::Pending)
    }
    
    /// Check if a block is finalized
    pub fn is_finalized(&self, block_hash: &Hash256) -> bool {
        matches!(
            self.get_finality_status(block_hash),
            FinalityStatus::Finalized
        )
    }
    
    /// Add a vote and update finality status
    /// Returns Ok(()) if vote was processed, Err if equivocation detected
    pub fn add_vote(&mut self, vote: FinalityVote) -> Result<(), EquivocationEvidence> {
        // Verify vote signature
        if !vote.verify() {
            return Ok(()); // Ignore invalid votes
        }
        
        // Check if validator is in the set
        let stake = match self.validator_stakes.get(&vote.validator) {
            Some(s) => *s,
            None => return Ok(()), // Ignore votes from non-validators
        };
        
        // Check for equivocation
        let key = (vote.block_height, vote.round, vote.vote_type, vote.validator.clone());
        if let Some(existing_hash) = self.vote_history.get(&key) {
            if *existing_hash != vote.block_hash {
                // Equivocation detected! Try to create evidence
                if let Some(existing_vote) = self.try_reconstruct_vote(
                    *existing_hash,
                    vote.block_height,
                    vote.round,
                    vote.vote_type,
                    vote.validator.clone(),
                ) {
                    let evidence = EquivocationEvidence {
                        vote1: existing_vote,
                        vote2: vote.clone(),
                        evidence_height: vote.block_height,
                    };
                    
                    // Record equivocation
                    self.equivocations.entry(vote.validator.clone())
                        .or_insert_with(Vec::new)
                        .push(evidence.clone());
                    
                    return Err(evidence);
                } else {
                    // Cannot reconstruct vote (data may have been pruned)
                    // Just record the new vote and continue
                    // Note: This is a rare edge case where vote data was pruned
                }
            }
        } else {
            // Record this vote in history
            self.vote_history.insert(key, vote.block_hash);
        }
        
        // Get or create vote tracker for this block
        let tracker = self.vote_trackers.entry(vote.block_hash)
            .or_insert_with(VoteTracker::new);
        
        // Add vote to tracker
        match vote.vote_type {
            VoteType::Prevote => {
                // Only add stake if this is a new vote from this validator
                if !tracker.prevotes.contains_key(&vote.validator) {
                    tracker.prevote_stake += stake;
                }
                tracker.prevotes.insert(vote.validator.clone(), vote.signature);
            }
            VoteType::Precommit => {
                // Only add stake if this is a new vote from this validator
                if !tracker.precommits.contains_key(&vote.validator) {
                    tracker.precommit_stake += stake;
                }
                tracker.precommits.insert(vote.validator.clone(), vote.signature);
            }
        }
        
        // Update finality status
        self.update_finality_status(vote.block_hash);
        
        Ok(())
    }
    
    /// Update finality status based on current votes
    fn update_finality_status(&mut self, block_hash: Hash256) {
        let tracker = match self.vote_trackers.get(&block_hash) {
            Some(t) => t,
            None => return,
        };
        
        // Calculate 2/3+ threshold with proper rounding
        // We need > 2/3, which means we need at least floor(2*total/3) + 1
        let threshold = (self.total_stake * 2) / 3;
        
        let current_status = self.get_finality_status(&block_hash);
        
        // Check for finalization (2/3+ precommits)
        if tracker.precommit_stake > threshold {
            self.finality_status.insert(block_hash, FinalityStatus::Finalized);
        }
        // Check for prevoted (2/3+ prevotes)
        else if tracker.prevote_stake > threshold && current_status == FinalityStatus::Pending {
            self.finality_status.insert(block_hash, FinalityStatus::Prevoted);
        }
    }
    
    /// Helper to try to reconstruct a vote from history (for equivocation evidence)
    /// Returns None if the vote data is not available (e.g., pruned)
    fn try_reconstruct_vote(
        &self,
        block_hash: Hash256,
        block_height: u64,
        round: u64,
        vote_type: VoteType,
        validator: PublicKey,
    ) -> Option<FinalityVote> {
        let tracker = self.vote_trackers.get(&block_hash)?;
        let signature = match vote_type {
            VoteType::Prevote => tracker.prevotes.get(&validator)?,
            VoteType::Precommit => tracker.precommits.get(&validator)?,
        };
        
        Some(FinalityVote {
            block_hash,
            block_height,
            vote_type,
            round,
            validator,
            signature: signature.clone(),
        })
    }
    
    /// Get all detected equivocations
    pub fn get_equivocations(&self) -> &HashMap<PublicKey, Vec<EquivocationEvidence>> {
        &self.equivocations
    }
    
    /// Get equivocations for a specific validator
    pub fn get_validator_equivocations(&self, validator: &PublicKey) -> Vec<EquivocationEvidence> {
        self.equivocations.get(validator)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Advance to next round (called on timeout)
    pub fn advance_round(&mut self) {
        self.current_round += 1;
    }
    
    /// Get current round
    pub fn current_round(&self) -> u64 {
        self.current_round
    }
    
    /// Get vote statistics for a block
    pub fn get_vote_stats(&self, block_hash: &Hash256) -> Option<(u64, u64)> {
        self.vote_trackers.get(block_hash)
            .map(|t| (t.prevote_stake, t.precommit_stake))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcell_crypto::SecretKey;
    
    fn create_test_validators(count: usize) -> (Vec<SecretKey>, HashMap<PublicKey, u64>) {
        let mut keys = Vec::new();
        let mut stakes = HashMap::new();
        
        for _ in 0..count {
            let sk = SecretKey::generate();
            stakes.insert(sk.public_key(), 100);
            keys.push(sk);
        }
        
        (keys, stakes)
    }
    
    fn create_vote(
        sk: &SecretKey,
        block_hash: Hash256,
        height: u64,
        vote_type: VoteType,
        round: u64,
    ) -> FinalityVote {
        let validator = sk.public_key();
        
        let vote = FinalityVote {
            block_hash,
            block_height: height,
            vote_type,
            round,
            validator: validator.clone(),
            signature: sk.sign(b"placeholder"), // Will be replaced
        };
        
        let msg = vote.sign_message();
        let signature = sk.sign(&msg);
        
        FinalityVote {
            signature,
            ..vote
        }
    }
    
    #[test]
    fn test_vote_verification() {
        let sk = SecretKey::generate();
        let block_hash = Hash256::hash(b"test block");
        
        let vote = create_vote(&sk, block_hash, 1, VoteType::Prevote, 0);
        assert!(vote.verify());
    }
    
    #[test]
    fn test_finality_threshold() {
        let (keys, stakes) = create_test_validators(4);
        let mut gadget = FinalityGadget::new(stakes);
        
        let block_hash = Hash256::hash(b"test block");
        
        // Add 3 prevotes (75% > 66.67%)
        for i in 0..3 {
            let vote = create_vote(&keys[i], block_hash, 1, VoteType::Prevote, 0);
            gadget.add_vote(vote).unwrap();
        }
        
        // Should be prevoted
        assert_eq!(gadget.get_finality_status(&block_hash), FinalityStatus::Prevoted);
        
        // Add 3 precommits
        for i in 0..3 {
            let vote = create_vote(&keys[i], block_hash, 1, VoteType::Precommit, 0);
            gadget.add_vote(vote).unwrap();
        }
        
        // Should be finalized
        assert_eq!(gadget.get_finality_status(&block_hash), FinalityStatus::Finalized);
        assert!(gadget.is_finalized(&block_hash));
    }
    
    #[test]
    fn test_equivocation_detection() {
        let (keys, stakes) = create_test_validators(1);
        let mut gadget = FinalityGadget::new(stakes);
        
        let block_hash1 = Hash256::hash(b"block 1");
        let block_hash2 = Hash256::hash(b"block 2");
        
        // Vote for first block
        let vote1 = create_vote(&keys[0], block_hash1, 1, VoteType::Prevote, 0);
        gadget.add_vote(vote1).unwrap();
        
        // Try to vote for different block at same height/round
        let vote2 = create_vote(&keys[0], block_hash2, 1, VoteType::Prevote, 0);
        let result = gadget.add_vote(vote2);
        
        // Should detect equivocation
        assert!(result.is_err());
        
        let evidence = result.unwrap_err();
        assert!(evidence.is_valid());
        assert_eq!(evidence.vote1.block_hash, block_hash1);
        assert_eq!(evidence.vote2.block_hash, block_hash2);
    }
    
    #[test]
    fn test_equivocation_different_rounds_ok() {
        let (keys, stakes) = create_test_validators(1);
        let mut gadget = FinalityGadget::new(stakes);
        
        let block_hash1 = Hash256::hash(b"block 1");
        let block_hash2 = Hash256::hash(b"block 2");
        
        // Vote for first block in round 0
        let vote1 = create_vote(&keys[0], block_hash1, 1, VoteType::Prevote, 0);
        gadget.add_vote(vote1).unwrap();
        
        // Vote for different block in round 1 - this is OK
        let vote2 = create_vote(&keys[0], block_hash2, 1, VoteType::Prevote, 1);
        let result = gadget.add_vote(vote2);
        
        // Should NOT detect equivocation (different rounds)
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_insufficient_votes() {
        let (keys, stakes) = create_test_validators(4);
        let mut gadget = FinalityGadget::new(stakes);
        
        let block_hash = Hash256::hash(b"test block");
        
        // Add only 2 votes (50% < 66.67%)
        for i in 0..2 {
            let vote = create_vote(&keys[i], block_hash, 1, VoteType::Prevote, 0);
            gadget.add_vote(vote).unwrap();
        }
        
        // Should still be pending
        assert_eq!(gadget.get_finality_status(&block_hash), FinalityStatus::Pending);
    }
    
    #[test]
    fn test_equivocation_evidence_validation() {
        let sk = SecretKey::generate();
        let block_hash1 = Hash256::hash(b"block 1");
        let block_hash2 = Hash256::hash(b"block 2");
        
        let vote1 = create_vote(&sk, block_hash1, 1, VoteType::Prevote, 0);
        let vote2 = create_vote(&sk, block_hash2, 1, VoteType::Prevote, 0);
        
        let evidence = EquivocationEvidence {
            vote1,
            vote2,
            evidence_height: 1,
        };
        
        assert!(evidence.is_valid());
    }
    
    #[test]
    fn test_vote_stats() {
        let (keys, stakes) = create_test_validators(4);
        let mut gadget = FinalityGadget::new(stakes);
        
        let block_hash = Hash256::hash(b"test block");
        
        // Add 2 prevotes
        for i in 0..2 {
            let vote = create_vote(&keys[i], block_hash, 1, VoteType::Prevote, 0);
            gadget.add_vote(vote).unwrap();
        }
        
        // Add 3 precommits
        for i in 0..3 {
            let vote = create_vote(&keys[i], block_hash, 1, VoteType::Precommit, 0);
            gadget.add_vote(vote).unwrap();
        }
        
        let (prevote_stake, precommit_stake) = gadget.get_vote_stats(&block_hash).unwrap();
        assert_eq!(prevote_stake, 200); // 2 validators * 100 stake
        assert_eq!(precommit_stake, 300); // 3 validators * 100 stake
    }
}
