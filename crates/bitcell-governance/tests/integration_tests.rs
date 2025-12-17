//! Comprehensive integration tests for governance system

use bitcell_governance::*;

const CELL: u64 = 100_000_000; // 1 CELL in smallest units

#[test]
fn test_full_proposal_lifecycle() {
    let mut gov = GovernanceManager::new();
    let proposer = [1u8; 33];
    let timestamp = 1000;
    
    // Submit proposal
    let proposal_id = gov.submit_proposal(
        proposer,
        ProposalType::ParameterChange {
            parameter: "max_block_size".to_string(),
            new_value: "2000000".to_string(),
        },
        "Increase block size".to_string(),
        timestamp,
    ).unwrap();
    
    // Vote with sufficient quorum
    gov.vote(proposal_id, [2u8; 33], true, 8000 * CELL, timestamp + 100).unwrap();
    gov.vote(proposal_id, [3u8; 33], true, 5000 * CELL, timestamp + 200).unwrap();
    gov.vote(proposal_id, [4u8; 33], false, 1000 * CELL, timestamp + 300).unwrap();
    
    // Try to finalize before timelock
    let result = gov.finalize_proposal(proposal_id, timestamp + 1000);
    assert!(matches!(result, Err(Error::TimelockNotExpired { .. })));
    
    // Finalize after timelock (2 days for parameter change)
    let after_timelock = timestamp + (2 * 24 * 60 * 60) + 1;
    let passed = gov.finalize_proposal(proposal_id, after_timelock).unwrap();
    
    assert!(passed);
    
    let proposal = gov.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Passed);
    assert_eq!(proposal.votes_for, 13000 * CELL);
    assert_eq!(proposal.votes_against, 1000 * CELL);
}

#[test]
fn test_treasury_spending_shorter_timelock() {
    let mut gov = GovernanceManager::new();
    let proposer = [1u8; 33];
    let timestamp = 1000;
    
    // Submit treasury spending proposal
    let proposal_id = gov.submit_proposal(
        proposer,
        ProposalType::TreasurySpending {
            recipient: [5u8; 33],
            amount: 10000 * CELL,
            reason: "Development grant".to_string(),
        },
        "Fund Q1 development".to_string(),
        timestamp,
    ).unwrap();
    
    // Vote with quorum
    gov.vote(proposal_id, [2u8; 33], true, 12000 * CELL, timestamp + 100).unwrap();
    
    // Should be able to finalize after 6 hours (not 2 days)
    let after_6_hours = timestamp + (6 * 60 * 60) + 1;
    let passed = gov.finalize_proposal(proposal_id, after_6_hours).unwrap();
    
    assert!(passed);
}

#[test]
fn test_quorum_failure() {
    let mut gov = GovernanceManager::new();
    let proposer = [1u8; 33];
    let timestamp = 1000;
    
    let proposal_id = gov.submit_proposal(
        proposer,
        ProposalType::ParameterChange {
            parameter: "test".to_string(),
            new_value: "value".to_string(),
        },
        "Test".to_string(),
        timestamp,
    ).unwrap();
    
    // Vote with insufficient quorum (less than 10,000 CELL)
    gov.vote(proposal_id, [2u8; 33], true, 5000 * CELL, timestamp + 100).unwrap();
    
    // Should fail due to quorum
    let after_timelock = timestamp + (2 * 24 * 60 * 60) + 1;
    let result = gov.finalize_proposal(proposal_id, after_timelock);
    
    assert!(matches!(result, Err(Error::QuorumNotReached { .. })));
    
    let proposal = gov.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Rejected);
}

#[test]
fn test_quadratic_voting() {
    let config = GovernanceConfig {
        voting_method: VotingMethod::Quadratic,
        ..Default::default()
    };
    
    let mut gov = GovernanceManager::with_config(config, GuardianSet::new());
    let proposer = [1u8; 33];
    let timestamp = 1000;
    
    let proposal_id = gov.submit_proposal(
        proposer,
        ProposalType::ParameterChange {
            parameter: "test".to_string(),
            new_value: "value".to_string(),
        },
        "Test quadratic".to_string(),
        timestamp,
    ).unwrap();
    
    // Vote with 10,000 CELL -> sqrt(10,000) = 100 effective votes
    gov.vote(proposal_id, [2u8; 33], true, 10000 * CELL, timestamp + 100).unwrap();
    
    let proposal = gov.get_proposal(&proposal_id).unwrap();
    // sqrt(10000) = 100
    assert_eq!(proposal.votes_for, 100);
    
    // Vote with 100 CELL -> sqrt(100) = 10 effective votes
    gov.vote(proposal_id, [3u8; 33], false, 100 * CELL, timestamp + 200).unwrap();
    
    let proposal = gov.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.votes_against, 10);
}

#[test]
fn test_delegation_increases_voting_power() {
    let mut gov = GovernanceManager::new();
    
    let alice = [1u8; 33];
    let bob = [2u8; 33];
    let carol = [3u8; 33];
    
    // Alice and Carol delegate to Bob
    gov.delegate(alice, bob, 5000 * CELL).unwrap();
    gov.delegate(carol, bob, 3000 * CELL).unwrap();
    
    // Bob has his own 2000 CELL
    let bob_power = gov.get_voting_power(&bob, 2000 * CELL);
    
    // Should be 2000 + 5000 + 3000 = 10000 CELL
    assert_eq!(bob_power, 10000 * CELL);
}

#[test]
fn test_undelegate_reduces_power() {
    let mut gov = GovernanceManager::new();
    
    let alice = [1u8; 33];
    let bob = [2u8; 33];
    
    gov.delegate(alice, bob, 5000 * CELL).unwrap();
    assert_eq!(gov.get_voting_power(&bob, 0), 5000 * CELL);
    
    gov.undelegate(alice, bob).unwrap();
    assert_eq!(gov.get_voting_power(&bob, 0), 0);
}

#[test]
fn test_self_delegation_fails() {
    let mut gov = GovernanceManager::new();
    let address = [1u8; 33];
    
    let result = gov.delegate(address, address, 1000 * CELL);
    assert!(matches!(result, Err(Error::InvalidDelegation)));
}

#[test]
fn test_double_vote_prevention() {
    let mut gov = GovernanceManager::new();
    let proposer = [1u8; 33];
    let voter = [2u8; 33];
    let timestamp = 1000;
    
    let proposal_id = gov.submit_proposal(
        proposer,
        ProposalType::ParameterChange {
            parameter: "test".to_string(),
            new_value: "value".to_string(),
        },
        "Test".to_string(),
        timestamp,
    ).unwrap();
    
    // First vote succeeds
    gov.vote(proposal_id, voter, true, 1000 * CELL, timestamp + 100).unwrap();
    
    // Second vote from same address fails
    let result = gov.vote(proposal_id, voter, false, 500 * CELL, timestamp + 200);
    assert!(matches!(result, Err(Error::DuplicateVote)));
}

#[test]
fn test_vote_on_finalized_proposal_fails() {
    let mut gov = GovernanceManager::new();
    let proposer = [1u8; 33];
    let timestamp = 1000;
    
    let proposal_id = gov.submit_proposal(
        proposer,
        ProposalType::TreasurySpending {
            recipient: [3u8; 33],
            amount: 1000 * CELL,
            reason: "Test".to_string(),
        },
        "Test".to_string(),
        timestamp,
    ).unwrap();
    
    // Vote and finalize
    gov.vote(proposal_id, [2u8; 33], true, 15000 * CELL, timestamp + 100).unwrap();
    let after_timelock = timestamp + (6 * 60 * 60) + 1;
    gov.finalize_proposal(proposal_id, after_timelock).unwrap();
    
    // Try to vote after finalization
    let result = gov.vote(proposal_id, [3u8; 33], false, 1000 * CELL, timestamp + 1000);
    assert!(matches!(result, Err(Error::ProposalFinalized)));
}

#[test]
fn test_proposal_rejection_on_negative_votes() {
    let mut gov = GovernanceManager::new();
    let proposer = [1u8; 33];
    let timestamp = 1000;
    
    let proposal_id = gov.submit_proposal(
        proposer,
        ProposalType::ParameterChange {
            parameter: "test".to_string(),
            new_value: "value".to_string(),
        },
        "Test".to_string(),
        timestamp,
    ).unwrap();
    
    // More votes against than for
    gov.vote(proposal_id, [2u8; 33], true, 4000 * CELL, timestamp + 100).unwrap();
    gov.vote(proposal_id, [3u8; 33], false, 8000 * CELL, timestamp + 200).unwrap();
    
    // Finalize
    let after_timelock = timestamp + (2 * 24 * 60 * 60) + 1;
    let passed = gov.finalize_proposal(proposal_id, after_timelock).unwrap();
    
    assert!(!passed);
    
    let proposal = gov.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Rejected);
}

#[test]
fn test_guardian_cancel() {
    use bitcell_crypto::{SecretKey, PublicKey};
    
    // Create guardian keys
    let guardian1 = SecretKey::generate();
    let guardian2 = SecretKey::generate();
    let guardian3 = SecretKey::generate();
    
    let guardians = GuardianSet::with_guardians(vec![
        Guardian {
            pubkey: guardian1.public_key().to_bytes(),
            name: "G1".to_string(),
            added_at: 1000,
        },
        Guardian {
            pubkey: guardian2.public_key().to_bytes(),
            name: "G2".to_string(),
            added_at: 1000,
        },
        Guardian {
            pubkey: guardian3.public_key().to_bytes(),
            name: "G3".to_string(),
            added_at: 1000,
        },
    ]);
    
    let mut gov = GovernanceManager::with_config(
        GovernanceConfig::default(),
        guardians,
    );
    
    let proposer = [1u8; 33];
    let timestamp = 1000;
    
    let proposal_id = gov.submit_proposal(
        proposer,
        ProposalType::ParameterChange {
            parameter: "malicious".to_string(),
            new_value: "bad".to_string(),
        },
        "Malicious proposal".to_string(),
        timestamp,
    ).unwrap();
    
    // Get guardian signatures
    let message = &proposal_id.0;
    let sig1 = guardian1.sign(message).to_bytes();
    let sig2 = guardian2.sign(message).to_bytes();
    
    // Guardian override to cancel
    gov.guardian_override(
        proposal_id,
        GuardianAction::Cancel,
        vec![sig1, sig2],
    ).unwrap();
    
    let proposal = gov.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Cancelled);
}

#[test]
fn test_guardian_insufficient_signatures() {
    use bitcell_crypto::SecretKey;
    
    let guardian1 = SecretKey::generate();
    
    let guardians = GuardianSet::with_guardians(vec![
        Guardian {
            pubkey: guardian1.public_key().to_bytes(),
            name: "G1".to_string(),
            added_at: 1000,
        },
        Guardian {
            pubkey: [2u8; 33],
            name: "G2".to_string(),
            added_at: 1000,
        },
        Guardian {
            pubkey: [3u8; 33],
            name: "G3".to_string(),
            added_at: 1000,
        },
    ]);
    
    let mut gov = GovernanceManager::with_config(
        GovernanceConfig::default(),
        guardians,
    );
    
    let proposal_id = gov.submit_proposal(
        [1u8; 33],
        ProposalType::ParameterChange {
            parameter: "test".to_string(),
            new_value: "value".to_string(),
        },
        "Test".to_string(),
        1000,
    ).unwrap();
    
    // Only one valid signature (need 2 of 3)
    let sig1 = guardian1.sign(&proposal_id.0).to_bytes();
    
    let result = gov.guardian_override(
        proposal_id,
        GuardianAction::Cancel,
        vec![sig1],
    );
    
    assert!(matches!(result, Err(Error::InsufficientGuardianApprovals { .. })));
}

#[test]
fn test_multiple_proposals() {
    let mut gov = GovernanceManager::new();
    let proposer = [1u8; 33];
    let timestamp = 1000;
    
    // Submit multiple proposals
    let id1 = gov.submit_proposal(
        proposer,
        ProposalType::ParameterChange {
            parameter: "param1".to_string(),
            new_value: "value1".to_string(),
        },
        "Proposal 1".to_string(),
        timestamp,
    ).unwrap();
    
    let id2 = gov.submit_proposal(
        proposer,
        ProposalType::TreasurySpending {
            recipient: [2u8; 33],
            amount: 1000 * CELL,
            reason: "Grant".to_string(),
        },
        "Proposal 2".to_string(),
        timestamp + 1,
    ).unwrap();
    
    // IDs should be different
    assert_ne!(id1, id2);
    
    // Both should exist
    assert!(gov.get_proposal(&id1).is_some());
    assert!(gov.get_proposal(&id2).is_some());
}

#[test]
fn test_vote_percentage_calculation() {
    let mut proposal = Proposal::new(
        [1u8; 33],
        ProposalType::ParameterChange {
            parameter: "test".to_string(),
            new_value: "value".to_string(),
        },
        "Test".to_string(),
        1000,
    );
    
    proposal.votes_for = 750 * CELL;
    proposal.votes_against = 250 * CELL;
    
    assert_eq!(proposal.total_votes(), 1000 * CELL);
    assert_eq!(proposal.vote_percentage_for(), 75.0);
}

#[test]
fn test_saturating_arithmetic() {
    let mut gov = GovernanceManager::new();
    let proposer = [1u8; 33];
    
    let proposal_id = gov.submit_proposal(
        proposer,
        ProposalType::ParameterChange {
            parameter: "test".to_string(),
            new_value: "value".to_string(),
        },
        "Test overflow protection".to_string(),
        1000,
    ).unwrap();
    
    // Vote with maximum value
    gov.vote(proposal_id, [2u8; 33], true, u64::MAX, 1100).unwrap();
    
    // Another vote should saturate, not overflow
    gov.vote(proposal_id, [3u8; 33], true, 1000 * CELL, 1200).unwrap();
    
    let proposal = gov.get_proposal(&proposal_id).unwrap();
    // Should be u64::MAX (saturated)
    assert_eq!(proposal.votes_for, u64::MAX);
}
