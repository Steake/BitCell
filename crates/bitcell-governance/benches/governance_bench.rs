use criterion::{black_box, criterion_group, criterion_main, Criterion};
use bitcell_governance::*;

const CELL: u64 = 100_000_000;

fn bench_submit_proposal(c: &mut Criterion) {
    c.bench_function("submit_proposal", |b| {
        let mut gov = GovernanceManager::new();
        let mut counter = 0u64;
        
        b.iter(|| {
            counter += 1;
            gov.submit_proposal(
                black_box([1u8; 33]),
                black_box(ProposalType::ParameterChange {
                    parameter: format!("param_{}", counter),
                    new_value: "value".to_string(),
                }),
                black_box(format!("Proposal {}", counter)),
                black_box(counter),
            ).unwrap()
        });
    });
}

fn bench_vote_linear(c: &mut Criterion) {
    c.bench_function("vote_linear", |b| {
        let mut gov = GovernanceManager::new();
        let proposal_id = gov.submit_proposal(
            [1u8; 33],
            ProposalType::ParameterChange {
                parameter: "test".to_string(),
                new_value: "value".to_string(),
            },
            "Test".to_string(),
            1000,
        ).unwrap();
        
        let mut voter_counter = 2u8;
        
        b.iter(|| {
            voter_counter = voter_counter.wrapping_add(1);
            let voter = [voter_counter; 33];
            gov.vote(
                black_box(proposal_id),
                black_box(voter),
                black_box(true),
                black_box(1000 * CELL),
                black_box(1100),
            ).unwrap()
        });
    });
}

fn bench_vote_quadratic(c: &mut Criterion) {
    c.bench_function("vote_quadratic", |b| {
        let config = GovernanceConfig {
            voting_method: VotingMethod::Quadratic,
            ..Default::default()
        };
        let mut gov = GovernanceManager::with_config(config, GuardianSet::new());
        
        let proposal_id = gov.submit_proposal(
            [1u8; 33],
            ProposalType::ParameterChange {
                parameter: "test".to_string(),
                new_value: "value".to_string(),
            },
            "Test".to_string(),
            1000,
        ).unwrap();
        
        let mut voter_counter = 2u8;
        
        b.iter(|| {
            voter_counter = voter_counter.wrapping_add(1);
            let voter = [voter_counter; 33];
            gov.vote(
                black_box(proposal_id),
                black_box(voter),
                black_box(true),
                black_box(10000 * CELL),
                black_box(1100),
            ).unwrap()
        });
    });
}

fn bench_delegation(c: &mut Criterion) {
    c.bench_function("delegate", |b| {
        let mut gov = GovernanceManager::new();
        let delegatee = [2u8; 33];
        let mut delegator_counter = 3u8;
        
        b.iter(|| {
            delegator_counter = delegator_counter.wrapping_add(1);
            let delegator = [delegator_counter; 33];
            gov.delegate(
                black_box(delegator),
                black_box(delegatee),
                black_box(1000 * CELL),
            ).unwrap()
        });
    });
}

fn bench_get_voting_power(c: &mut Criterion) {
    c.bench_function("get_voting_power", |b| {
        let mut gov = GovernanceManager::new();
        let delegatee = [2u8; 33];
        
        // Add some delegations
        for i in 0..10 {
            gov.delegate([i; 33], delegatee, 1000 * CELL).unwrap();
        }
        
        b.iter(|| {
            gov.get_voting_power(
                black_box(&delegatee),
                black_box(5000 * CELL),
            )
        });
    });
}

fn bench_finalize_proposal(c: &mut Criterion) {
    c.bench_function("finalize_proposal", |b| {
        b.iter_batched(
            || {
                let mut gov = GovernanceManager::new();
                let proposal_id = gov.submit_proposal(
                    [1u8; 33],
                    ProposalType::TreasurySpending {
                        recipient: [2u8; 33],
                        amount: 1000 * CELL,
                        reason: "Test".to_string(),
                    },
                    "Test".to_string(),
                    1000,
                ).unwrap();
                
                // Vote with quorum
                gov.vote(proposal_id, [2u8; 33], true, 15000 * CELL, 1100).unwrap();
                
                (gov, proposal_id)
            },
            |(mut gov, proposal_id)| {
                gov.finalize_proposal(
                    black_box(proposal_id),
                    black_box(1000 + 6 * 60 * 60 + 1),
                ).unwrap()
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_integer_sqrt(c: &mut Criterion) {
    c.bench_function("integer_sqrt", |b| {
        b.iter(|| {
            // Test with various values
            for n in [100, 10000, 1000000, 100000000u64] {
                black_box(bitcell_governance::integer_sqrt(n));
            }
        });
    });
}

criterion_group!(
    benches,
    bench_submit_proposal,
    bench_vote_linear,
    bench_vote_quadratic,
    bench_delegation,
    bench_get_voting_power,
    bench_finalize_proposal,
    bench_integer_sqrt,
);
criterion_main!(benches);
