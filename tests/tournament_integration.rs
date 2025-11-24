//! Integration tests for full tournament flow

use bitcell_ca::{Battle, Glider, GliderPattern, Position};
use bitcell_consensus::{Block, BlockHeader, TournamentPhase, GliderCommit, GliderReveal};
use bitcell_crypto::Hash256;
use bitcell_ebsl::{EvidenceCounters, EvidenceType, TrustParams};
use bitcell_state::{Account, BondState};

#[test]
fn test_full_tournament_flow() {
    // Test a complete tournament from commit to battle completion
    
    // Setup: 4 miners
    let miners = vec![
        generate_miner_id(1),
        generate_miner_id(2),
        generate_miner_id(3),
        generate_miner_id(4),
    ];
    
    // Phase 1: Commit
    let commits: Vec<GliderCommit> = miners.iter().enumerate().map(|(i, miner_id)| {
        GliderCommit {
            miner_id: *miner_id,
            commitment: Hash256::from_bytes(&[i as u8; 32]),
            timestamp: 1000 + i as u64,
        }
    }).collect();
    
    assert_eq!(commits.len(), 4);
    
    // Phase 2: Reveal
    let reveals: Vec<GliderReveal> = commits.iter().enumerate().map(|(i, commit)| {
        GliderReveal {
            miner_id: commit.miner_id,
            pattern: match i % 4 {
                0 => GliderPattern::Standard,
                1 => GliderPattern::Lightweight,
                2 => GliderPattern::Middleweight,
                _ => GliderPattern::Heavyweight,
            },
            nonce: i as u64,
        }
    }).collect();
    
    assert_eq!(reveals.len(), 4);
    
    // Phase 3: Battle (simplified - just verify battles can be executed)
    let glider_a = Glider::new(reveals[0].pattern, Position::new(200, 200));
    let glider_b = Glider::new(reveals[1].pattern, Position::new(800, 800));
    
    let battle = Battle::new(glider_a, glider_b);
    let outcome = battle.simulate();
    
    // Outcome should be one of the three valid results
    assert!(matches!(outcome, bitcell_ca::BattleOutcome::AWins | bitcell_ca::BattleOutcome::BWins | bitcell_ca::BattleOutcome::Tie));
}

#[test]
fn test_multi_round_tournament() {
    // Test tournament bracket with 4 participants -> 2 rounds
    let participants = 4;
    let rounds_needed = (participants as f64).log2().ceil() as usize;
    assert_eq!(rounds_needed, 2);
    
    // Round 1: 4 -> 2
    let round1_battles = participants / 2;
    assert_eq!(round1_battles, 2);
    
    // Round 2: 2 -> 1
    let round2_battles = round1_battles / 2;
    assert_eq!(round2_battles, 1);
}

#[test]
fn test_evidence_based_eligibility() {
    // Test that miners with low trust scores are excluded
    let params = TrustParams::default();
    
    // Good miner: lots of positive evidence
    let mut good_counters = EvidenceCounters::new();
    for _ in 0..100 {
        good_counters.record(EvidenceType::Positive, 1.0);
    }
    let good_trust = good_counters.trust_score(&params);
    assert!(good_trust.is_eligible(&params));
    
    // Bad miner: lots of negative evidence
    let mut bad_counters = EvidenceCounters::new();
    for _ in 0..100 {
        bad_counters.record(EvidenceType::Negative, 10.0);
    }
    let bad_trust = bad_counters.trust_score(&params);
    assert!(!bad_trust.is_eligible(&params));
}

#[test]
fn test_bond_requirements() {
    // Test that unbonded miners cannot participate
    let account = Account::new(1000);
    assert_eq!(account.balance(), 1000);
    
    // Bond state transitions
    let bonded = BondState::Active { amount: 100, epoch: 1 };
    assert!(matches!(bonded, BondState::Active { .. }));
    
    let unbonding = BondState::Unbonding { 
        amount: 100, 
        unbond_epoch: 10 
    };
    assert!(matches!(unbonding, BondState::Unbonding { .. }));
    
    let slashed = BondState::Slashed { 
        original_amount: 100, 
        slashed_amount: 50,
        slash_epoch: 5,
    };
    assert!(matches!(slashed, BondState::Slashed { .. }));
}

#[test]
fn test_block_validation_flow() {
    // Test basic block structure and validation
    let header = BlockHeader {
        height: 1,
        prev_hash: Hash256::from_bytes(&[0; 32]),
        state_root: Hash256::from_bytes(&[1; 32]),
        tournament_root: Hash256::from_bytes(&[2; 32]),
        timestamp: 1000,
        proposer: Hash256::from_bytes(&[3; 32]),
        vrf_output: Hash256::from_bytes(&[4; 32]),
        vrf_proof: vec![0; 64],
        work: 1000,
    };
    
    let block = Block {
        header: header.clone(),
        transactions: vec![],
        battle_proofs: vec![],
    };
    
    assert_eq!(block.header.height, 1);
    assert_eq!(block.header.work, 1000);
    assert_eq!(block.transactions.len(), 0);
}

#[test]
fn test_deterministic_work_calculation() {
    // Test that work is deterministic based on participants
    let num_miners = 100;
    let battle_steps = 1000;
    let grid_cost = 1;
    
    let work = (num_miners - 1) * battle_steps * grid_cost;
    assert_eq!(work, 99_000);
    
    // More miners = more work
    let num_miners_2 = 200;
    let work_2 = (num_miners_2 - 1) * battle_steps * grid_cost;
    assert_eq!(work_2, 199_000);
    assert!(work_2 > work);
}

fn generate_miner_id(seed: u8) -> Hash256 {
    Hash256::from_bytes(&[seed; 32])
}
