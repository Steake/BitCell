//! Benchmarks for BitCell cryptographic operations
//!
//! Run with: cargo bench -p bitcell-crypto

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use bitcell_crypto::{
    Hash256,
    poseidon::{poseidon_hash_two, poseidon_hash_one, poseidon_hash_many, PoseidonParams, PoseidonHasher},
    SecretKey,
    MerkleTree,
    ClsagSecretKey, ClsagPublicKey, ClsagSignature,
    MIN_RING_SIZE, DEFAULT_RING_SIZE, MAX_RING_SIZE,
    EcvrfSecretKey,
};
use ark_bn254::Fr;

/// Benchmark Poseidon hash operations
fn bench_poseidon_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("poseidon");
    
    // Benchmark single element hash
    let input = Fr::from(12345u64);
    group.bench_function("hash_one", |b| {
        b.iter(|| poseidon_hash_one(black_box(input)))
    });
    
    // Benchmark two-element compression
    let left = Fr::from(12345u64);
    let right = Fr::from(67890u64);
    group.bench_function("hash_two", |b| {
        b.iter(|| poseidon_hash_two(black_box(left), black_box(right)))
    });
    
    // Benchmark sponge hash with various input sizes
    for size in [4, 8, 16, 32, 64].iter() {
        let inputs: Vec<Fr> = (0..*size).map(|i| Fr::from(i as u64)).collect();
        group.bench_with_input(BenchmarkId::new("hash_many", size), size, |b, _| {
            b.iter(|| poseidon_hash_many(black_box(&inputs)))
        });
    }
    
    group.finish();
}

/// Benchmark SHA-256 hash operations
fn bench_sha256_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("sha256");
    
    // Various input sizes
    for size in [32, 64, 128, 256, 1024].iter() {
        let data = vec![0u8; *size];
        group.bench_with_input(BenchmarkId::new("hash", size), size, |b, _| {
            b.iter(|| Hash256::hash(black_box(&data)))
        });
    }
    
    // Multiple items hash
    let items: Vec<&[u8]> = vec![&[0u8; 32][..], &[1u8; 32][..], &[2u8; 32][..]];
    group.bench_function("hash_multiple", |b| {
        b.iter(|| Hash256::hash_multiple(black_box(&items)))
    });
    
    group.finish();
}

/// Benchmark signature operations
fn bench_signatures(c: &mut Criterion) {
    let mut group = c.benchmark_group("signatures");
    
    // Key generation
    group.bench_function("key_generation", |b| {
        b.iter(|| SecretKey::generate())
    });
    
    // Signing
    let sk = SecretKey::generate();
    let message = [0u8; 32];
    group.bench_function("sign", |b| {
        b.iter(|| sk.sign(black_box(&message)))
    });
    
    // Verification
    let pk = sk.public_key();
    let signature = sk.sign(&message);
    group.bench_function("verify", |b| {
        b.iter(|| signature.verify(black_box(&pk), black_box(&message)))
    });
    
    group.finish();
}

/// Benchmark Merkle tree operations
fn bench_merkle_tree(c: &mut Criterion) {
    let mut group = c.benchmark_group("merkle_tree");
    
    // Tree construction with various sizes
    for size in [4, 16, 64, 256, 1024].iter() {
        let leaves: Vec<Hash256> = (0..*size)
            .map(|i| Hash256::hash(&(i as u64).to_le_bytes()))
            .collect();
        
        group.bench_with_input(BenchmarkId::new("construct", size), size, |b, _| {
            b.iter(|| MerkleTree::new(black_box(leaves.clone())))
        });
    }
    
    // Proof generation
    let leaves: Vec<Hash256> = (0..1024)
        .map(|i| Hash256::hash(&(i as u64).to_le_bytes()))
        .collect();
    let tree = MerkleTree::new(leaves);
    
    group.bench_function("prove", |b| {
        b.iter(|| tree.prove(black_box(512)))
    });
    
    // Proof verification
    let proof = tree.prove(512).unwrap();
    let root = tree.root();
    group.bench_function("verify_proof", |b| {
        b.iter(|| MerkleTree::verify_proof(black_box(root), black_box(&proof)))
    });
    
    group.finish();
}

/// Benchmark Poseidon parameter generation
fn bench_poseidon_params(c: &mut Criterion) {
    let mut group = c.benchmark_group("poseidon_params");
    
    // Parameter generation (should be cached in practice)
    group.bench_function("bn254_2_to_1", |b| {
        b.iter(|| PoseidonParams::bn254_2_to_1())
    });
    
    // Hasher creation
    let params = PoseidonParams::bn254_2_to_1();
    group.bench_function("hasher_new", |b| {
        b.iter(|| PoseidonHasher::new(black_box(&params)))
    });
    
    group.finish();
}

/// Benchmark comparison: Poseidon vs SHA256 for Merkle tree use case
fn bench_hash_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_comparison");
    
    // Compare hashing two 32-byte values (typical Merkle tree operation)
    let data_a = [1u8; 32];
    let data_b = [2u8; 32];
    
    // SHA-256
    group.bench_function("sha256_merkle_node", |b| {
        b.iter(|| Hash256::hash_multiple(&[black_box(&data_a[..]), black_box(&data_b[..])]))
    });
    
    // Poseidon (for ZK circuits)
    let fr_a = Fr::from(123u64);
    let fr_b = Fr::from(456u64);
    group.bench_function("poseidon_merkle_node", |b| {
        b.iter(|| poseidon_hash_two(black_box(fr_a), black_box(fr_b)))
    });
    
    group.finish();
}

/// Benchmark CLSAG ring signature operations
fn bench_clsag_signatures(c: &mut Criterion) {
    let mut group = c.benchmark_group("clsag");
    
    // Test different ring sizes: minimum, default, and maximum
    let ring_sizes = vec![MIN_RING_SIZE, DEFAULT_RING_SIZE, 32, MAX_RING_SIZE];
    
    for ring_size in ring_sizes {
        // Generate ring of keys
        let secret_keys: Vec<ClsagSecretKey> = (0..ring_size)
            .map(|_| ClsagSecretKey::generate())
            .collect();
        let ring: Vec<ClsagPublicKey> = secret_keys
            .iter()
            .map(|sk| sk.public_key())
            .collect();
        
        let signer_sk = &secret_keys[ring_size / 2]; // Use middle key
        let message = b"tournament commitment";
        
        // Benchmark signing
        group.bench_with_input(
            BenchmarkId::new("sign", ring_size),
            &ring_size,
            |b, _| {
                b.iter(|| {
                    ClsagSignature::sign(
                        black_box(signer_sk),
                        black_box(&ring),
                        black_box(message)
                    )
                })
            },
        );
        
        // Pre-generate signature for verification benchmark
        let signature = ClsagSignature::sign(signer_sk, &ring, message).unwrap();
        
        // Benchmark verification
        group.bench_with_input(
            BenchmarkId::new("verify", ring_size),
            &ring_size,
            |b, _| {
                b.iter(|| {
                    signature.verify(black_box(&ring), black_box(message))
                })
            },
        );
    }
    
    // Benchmark key image generation
    let sk = ClsagSecretKey::generate();
    group.bench_function("key_image", |b| {
        b.iter(|| black_box(&sk).key_image())
    });
    
    group.finish();
}

/// Benchmark ECVRF operations (for block proposer selection)
fn bench_ecvrf(c: &mut Criterion) {
    let mut group = c.benchmark_group("ecvrf");
    
    // Key generation
    group.bench_function("key_generation", |b| {
        b.iter(|| EcvrfSecretKey::generate())
    });
    
    // VRF prove operation (block proposer generates proof)
    let sk = EcvrfSecretKey::generate();
    let message = b"block_hash_for_vrf_input_test_message_32";
    
    group.bench_function("prove", |b| {
        b.iter(|| sk.prove(black_box(message)))
    });
    
    // VRF verify operation (validators verify proof)
    let pk = sk.public_key();
    let (_, proof) = sk.prove(message);
    
    group.bench_function("verify", |b| {
        b.iter(|| proof.verify(black_box(&pk), black_box(message)))
    });
    
    // Benchmark with different message sizes (block hash inputs)
    for size in [32, 64, 128, 256].iter() {
        let msg = vec![0u8; *size];
        group.bench_with_input(
            BenchmarkId::new("prove_variable_input", size), 
            size, 
            |b, _| {
                b.iter(|| sk.prove(black_box(&msg)))
            }
        );
    }
    
    group.finish();
}

/// Benchmark VRF chaining (simulating blockchain block production)
fn bench_vrf_chaining(c: &mut Criterion) {
    let mut group = c.benchmark_group("vrf_chaining");
    
    let sk = EcvrfSecretKey::generate();
    
    // Simulate producing 10 blocks with VRF chaining
    group.bench_function("produce_10_blocks", |b| {
        b.iter(|| {
            let genesis_seed = b"genesis_block_seed_for_chain";
            let (mut prev_output, _) = sk.prove(genesis_seed);
            
            // Generate 10 chained VRF outputs
            for _ in 0..10 {
                let (output, _) = sk.prove(prev_output.as_bytes());
                prev_output = output;
            }
        })
    });
    
    // Benchmark single block VRF (using previous VRF output)
    let genesis_seed = b"genesis_seed";
    let (prev_output, _) = sk.prove(genesis_seed);
    
    group.bench_function("single_chained_block", |b| {
        b.iter(|| sk.prove(black_box(prev_output.as_bytes())))
    });
    
    group.finish();
}

/// Benchmark VRF for multiple proposers (validator selection)
fn bench_vrf_multiple_proposers(c: &mut Criterion) {
    let mut group = c.benchmark_group("vrf_proposer_selection");
    
    // Create multiple validator keys
    let validators: Vec<EcvrfSecretKey> = (0..10)
        .map(|_| EcvrfSecretKey::generate())
        .collect();
    
    let block_hash = b"shared_block_hash_for_proposer_selection_input";
    
    // Benchmark all validators generating VRF proofs for proposer selection
    group.bench_function("10_validators_prove", |b| {
        b.iter(|| {
            for validator_sk in &validators {
                let _ = validator_sk.prove(black_box(block_hash));
            }
        })
    });
    
    // Benchmark verifying all proofs
    let proofs: Vec<_> = validators
        .iter()
        .map(|sk| {
            let pk = sk.public_key();
            let (_, proof) = sk.prove(block_hash);
            (pk, proof)
        })
        .collect();
    
    group.bench_function("10_validators_verify", |b| {
        b.iter(|| {
            for (pk, proof) in &proofs {
                let _ = proof.verify(black_box(pk), black_box(block_hash));
            }
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_poseidon_hash,
    bench_sha256_hash,
    bench_signatures,
    bench_merkle_tree,
    bench_poseidon_params,
    bench_hash_comparison,
    bench_clsag_signatures,
    bench_ecvrf,
    bench_vrf_chaining,
    bench_vrf_multiple_proposers,
);

criterion_main!(benches);
