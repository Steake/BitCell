//! Integration tests for the finality gadget

use bitcell_consensus::{Block, BlockHeader, FinalityGadget, FinalityVote, FinalityStatus, VoteType};
use bitcell_crypto::{Hash256, SecretKey};
use std::collections::HashMap;

/// Helper to create a test block
fn create_test_block(height: u64, proposer_key: &SecretKey) -> Block {
    Block {
        header: BlockHeader {
            height,
            prev_hash: Hash256::zero(),
            tx_root: Hash256::zero(),
            state_root: Hash256::zero(),
            timestamp: 1234567890,
            proposer: proposer_key.public_key(),
            vrf_output: [0u8; 32],
            vrf_proof: vec![],
            work: 1000,
        },
        transactions: vec![],
        battle_proofs: vec![],
        signature: proposer_key.sign(b"block"),
        finality_votes: vec![],
        finality_status: FinalityStatus::Pending,
    }
}

/// Helper to create a finality vote
fn create_finality_vote(
    validator_key: &SecretKey,
    block_hash: Hash256,
    height: u64,
    vote_type: VoteType,
    round: u64,
) -> FinalityVote {
    let vote = FinalityVote {
        block_hash,
        block_height: height,
        vote_type,
        round,
        validator: validator_key.public_key(),
        signature: validator_key.sign(b"temp"), // Will be replaced
    };
    
    let msg = vote.sign_message();
    let signature = validator_key.sign(&msg);
    
    FinalityVote {
        signature,
        ..vote
    }
}

#[test]
fn test_complete_finality_flow() {
    // Setup: 5 validators with equal stake
    let validators: Vec<SecretKey> = (0..5).map(|_| SecretKey::generate()).collect();
    let mut stakes = HashMap::new();
    for validator in &validators {
        stakes.insert(validator.public_key(), 100);
    }
    
    let mut gadget = FinalityGadget::new(stakes);
    
    // Create a block
    let block = create_test_block(1, &validators[0]);
    let block_hash = block.hash();
    
    // Initially, block should be pending
    assert_eq!(gadget.get_finality_status(&block_hash), FinalityStatus::Pending);
    
    // Step 1: Collect prevotes from 4 out of 5 validators (80% > 66.67%)
    for i in 0..4 {
        let vote = create_finality_vote(
            &validators[i],
            block_hash,
            1,
            VoteType::Prevote,
            0,
        );
        gadget.add_vote(vote).expect("Prevote should be accepted");
    }
    
    // After sufficient prevotes, block should be prevoted
    assert_eq!(gadget.get_finality_status(&block_hash), FinalityStatus::Prevoted);
    assert!(!gadget.is_finalized(&block_hash));
    
    // Step 2: Collect precommits from same 4 validators
    for i in 0..4 {
        let vote = create_finality_vote(
            &validators[i],
            block_hash,
            1,
            VoteType::Precommit,
            0,
        );
        gadget.add_vote(vote).expect("Precommit should be accepted");
    }
    
    // After sufficient precommits, block should be finalized
    assert_eq!(gadget.get_finality_status(&block_hash), FinalityStatus::Finalized);
    assert!(gadget.is_finalized(&block_hash));
    
    // Verify vote statistics
    let (prevote_stake, precommit_stake) = gadget.get_vote_stats(&block_hash).unwrap();
    assert_eq!(prevote_stake, 400); // 4 validators * 100 stake
    assert_eq!(precommit_stake, 400);
}

#[test]
fn test_equivocation_prevents_finalization() {
    // Setup: 4 validators
    let validators: Vec<SecretKey> = (0..4).map(|_| SecretKey::generate()).collect();
    let mut stakes = HashMap::new();
    for validator in &validators {
        stakes.insert(validator.public_key(), 100);
    }
    
    let mut gadget = FinalityGadget::new(stakes);
    
    let block1 = create_test_block(1, &validators[0]);
    let block1_hash = block1.hash();
    
    let block2 = create_test_block(1, &validators[1]);
    let block2_hash = block2.hash();
    
    // Validator 0 votes for block1
    let vote1 = create_finality_vote(
        &validators[0],
        block1_hash,
        1,
        VoteType::Precommit,
        0,
    );
    gadget.add_vote(vote1).expect("First vote should succeed");
    
    // Validator 0 tries to vote for block2 (equivocation!)
    let vote2 = create_finality_vote(
        &validators[0],
        block2_hash,
        1,
        VoteType::Precommit,
        0,
    );
    let result = gadget.add_vote(vote2);
    
    // Should detect equivocation
    assert!(result.is_err());
    let evidence = result.unwrap_err();
    assert!(evidence.is_valid());
    
    // Check that equivocation was recorded
    let equivocations = gadget.get_validator_equivocations(&validators[0].public_key());
    assert_eq!(equivocations.len(), 1);
    assert_eq!(equivocations[0].vote1.block_hash, block1_hash);
    assert_eq!(equivocations[0].vote2.block_hash, block2_hash);
}
