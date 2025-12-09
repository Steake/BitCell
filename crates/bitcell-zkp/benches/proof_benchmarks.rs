//! Performance benchmarks for proof generation and verification
//!
//! Measures performance metrics to ensure we meet the requirements:
//! - Block verification time < 5ms
//! - Proof generation < 10s per block
//! - Aggregation commitment size < 1KB

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use bitcell_zkp::{
    BattleCircuit, ProofAggregator,
};
use ark_bn254::Fr;
use ark_ff::One;
use std::time::Duration;

fn bench_battle_proof_generation(c: &mut Criterion) {
    let (pk, _vk) = BattleCircuit::setup().expect("Setup should succeed");
    
    c.bench_function("battle_proof_generation", |b| {
        b.iter(|| {
            let circuit = BattleCircuit::new(
                black_box(Fr::one()),
                black_box(Fr::one()),
                black_box(1),
                black_box(100),
                black_box(200),
            );
            circuit.prove(&pk).expect("Proof should succeed")
        });
    });
}

fn bench_battle_proof_verification(c: &mut Criterion) {
    let (pk, vk) = BattleCircuit::setup().expect("Setup should succeed");
    
    let circuit = BattleCircuit::new(Fr::one(), Fr::one(), 1, 100, 200);
    let proof = circuit.prove(&pk).expect("Proof should succeed");
    let public_inputs = vec![Fr::one(), Fr::one(), Fr::from(1u8)];
    
    c.bench_function("battle_proof_verification", |b| {
        b.iter(|| {
            BattleCircuit::verify(&vk, &proof, black_box(&public_inputs))
                .expect("Verification should succeed")
        });
    });
}

fn bench_aggregation_commitment(c: &mut Criterion) {
    let (pk, _vk) = BattleCircuit::setup().expect("Setup should succeed");
    
    // Generate proofs
    let mut proofs = Vec::new();
    for i in 0..10 {
        let circuit = BattleCircuit::new(Fr::one(), Fr::one(), (i % 3) as u8, 100, 200);
        let proof = circuit.prove(&pk).expect("Proof should succeed");
        proofs.push(proof);
    }
    
    c.bench_function("aggregation_commitment_10_proofs", |b| {
        b.iter(|| {
            ProofAggregator::create_aggregation_commitment(black_box(&proofs))
        });
    });
}

criterion_group! {
    name = proof_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(5))
        .sample_size(100);  // Increased for more stable benchmarks
    targets = 
        bench_battle_proof_generation,
        bench_battle_proof_verification,
        bench_aggregation_commitment
}

criterion_main!(proof_benches);
