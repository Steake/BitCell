//! Benchmarks for BitCell cryptographic operations
//!
//! Run with: cargo bench -p bitcell-crypto

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use bitcell_crypto::{
    Hash256,
    poseidon::{poseidon_hash_two, poseidon_hash_one, poseidon_hash_many, PoseidonParams, PoseidonHasher},
    SecretKey, PublicKey, Signature,
    MerkleTree,
    ClsagSecretKey, ClsagPublicKey, ClsagSignature,
    MIN_RING_SIZE, DEFAULT_RING_SIZE, MAX_RING_SIZE,
};
use ark_bn254::Fr;
use ark_ff::One;

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
    
    // Key generation
    group.bench_function("key_generation", |b| {
        b.iter(|| ClsagSecretKey::generate())
    });
    
    // Key image computation
    let sk = ClsagSecretKey::generate();
    group.bench_function("key_image", |b| {
        b.iter(|| sk.key_image())
    });
    
    // Signing with various ring sizes
    let message = b"tournament commitment";
    
    for ring_size in [MIN_RING_SIZE, DEFAULT_RING_SIZE, 32, MAX_RING_SIZE].iter() {
        // Create a ring
        let mut keys = vec![];
        let mut ring = vec![];
        for _ in 0..*ring_size {
            let sk = ClsagSecretKey::generate();
            keys.push(sk.clone());
            ring.push(sk.public_key());
        }
        
        let signer_idx = ring_size / 2;
        
        group.bench_with_input(
            BenchmarkId::new("sign", ring_size),
            ring_size,
            |b, _| {
                b.iter(|| {
                    ClsagSignature::sign(
                        black_box(&keys[signer_idx]),
                        black_box(&ring),
                        black_box(message),
                    )
                })
            },
        );
    }
    
    // Verification with various ring sizes
    for ring_size in [MIN_RING_SIZE, DEFAULT_RING_SIZE, 32, MAX_RING_SIZE].iter() {
        // Create a ring
        let mut keys = vec![];
        let mut ring = vec![];
        for _ in 0..*ring_size {
            let sk = ClsagSecretKey::generate();
            keys.push(sk.clone());
            ring.push(sk.public_key());
        }
        
        let signer_idx = ring_size / 2;
        let sig = ClsagSignature::sign(&keys[signer_idx], &ring, message).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("verify", ring_size),
            ring_size,
            |b, _| {
                b.iter(|| sig.verify(black_box(&ring), black_box(message)))
            },
        );
    }
    
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
);

criterion_main!(benches);
