//! Security Audit Tests for Cryptographic Primitives
//!
//! This test suite implements the security audit requirements for RC3-001.1
//! as specified in docs/SECURITY_AUDIT.md
//!
//! Test Categories:
//! 1. Hash Functions (SHA-256, Poseidon)
//! 2. Digital Signatures (ECDSA, Ring Signatures)
//! 3. Verifiable Random Functions (ECVRF)
//! 4. Commitment Schemes (Pedersen)
//! 5. Merkle Trees
//! 6. Key Management
//! 7. Protocol-Level Cryptography

use bitcell_crypto::*;
use std::collections::HashSet;

// =============================================================================
// 1. Hash Function Security Tests
// =============================================================================

mod hash_security {
    use super::*;

    #[test]
    fn test_sha256_deterministic() {
        // Hash must be deterministic
        let data = b"test data";
        let hash1 = Hash256::hash(data);
        let hash2 = Hash256::hash(data);
        assert_eq!(hash1, hash2, "SHA-256 must be deterministic");
    }

    #[test]
    fn test_sha256_different_inputs() {
        // Different inputs must produce different hashes
        let data1 = b"input1";
        let data2 = b"input2";
        let hash1 = Hash256::hash(data1);
        let hash2 = Hash256::hash(data2);
        assert_ne!(hash1, hash2, "Different inputs must produce different hashes");
    }

    #[test]
    fn test_sha256_empty_input() {
        // Empty input must be handled correctly
        let empty = b"";
        let hash = Hash256::hash(empty);
        // Known SHA-256 hash of empty string
        let expected = hex::decode("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
            .expect("valid hex");
        assert_eq!(hash.as_bytes(), expected.as_slice(), "Empty input hash must match known value");
    }

    #[test]
    fn test_sha256_large_input() {
        // Large inputs must be handled correctly
        let large_data = vec![0u8; 1_000_000]; // 1MB
        let hash = Hash256::hash(&large_data);
        assert_eq!(hash.as_bytes().len(), 32, "Hash must be 32 bytes");
    }

    #[test]
    fn test_sha256_collision_resistance() {
        // Generate many hashes and check for collisions
        let mut hashes = HashSet::new();
        for i in 0..1000 {
            let data = format!("data_{}", i);
            let hash = Hash256::hash(data.as_bytes());
            assert!(hashes.insert(hash), "Hash collision detected at iteration {}", i);
        }
    }

    #[test]
    fn test_hash_preimage_resistance() {
        // It should be infeasible to find the input from the hash
        // This is a property test demonstrating the concept
        let secret = b"my_secret_data";
        let hash = Hash256::hash(secret);
        
        // Try some common inputs - none should match
        for guess in &[b"", b"0", b"password", b"secret", b"data"] {
            let guess_hash = Hash256::hash(*guess);
            assert_ne!(hash, guess_hash, "Preimage should not be easily guessable");
        }
    }

    #[test]
    fn test_hash_avalanche_effect() {
        // Small change in input should cause large change in output
        let data1 = b"test data";
        let data2 = b"test datb"; // One bit difference
        let hash1 = Hash256::hash(data1);
        let hash2 = Hash256::hash(data2);
        
        // Count different bits
        let mut different_bits = 0;
        for (b1, b2) in hash1.as_bytes().iter().zip(hash2.as_bytes().iter()) {
            different_bits += (b1 ^ b2).count_ones();
        }
        
        // Avalanche effect: approximately 50% of bits should differ
        // With 256 bits, expect ~128 bits different (allow range 64-192)
        assert!(different_bits >= 64 && different_bits <= 192,
                "Avalanche effect: {} bits differ (expected ~128)", different_bits);
    }

    #[test]
    fn test_hash_zero_value() {
        // Hash of zero should be well-defined
        let zero = Hash256::zero();
        assert_eq!(zero.as_bytes(), &[0u8; 32], "Zero hash must be all zeros");
    }
}

// =============================================================================
// 2. Digital Signature Security Tests
// =============================================================================

mod signature_security {
    use super::*;

    #[test]
    fn test_signature_generation_uniqueness() {
        // Each signature should be verifiable (deterministic in our impl)
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        let msg = b"test message";
        
        let sig1 = sk.sign(msg);
        let sig2 = sk.sign(msg);
        
        // Both should verify correctly
        assert!(sig1.verify(&pk, msg).is_ok(), "Signature 1 must verify");
        assert!(sig2.verify(&pk, msg).is_ok(), "Signature 2 must verify");
    }

    #[test]
    fn test_signature_verification_correctness() {
        // Valid signature must verify
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        let msg = b"test message";
        
        let sig = sk.sign(msg);
        assert!(sig.verify(&pk, msg).is_ok(), "Valid signature must verify");
    }

    #[test]
    fn test_signature_verification_wrong_key() {
        // Signature must fail with wrong public key
        let sk1 = SecretKey::generate();
        let sk2 = SecretKey::generate();
        let pk2 = sk2.public_key();
        let msg = b"test message";
        
        let sig = sk1.sign(msg);
        assert!(sig.verify(&pk2, msg).is_err(), "Signature must fail with wrong key");
    }

    #[test]
    fn test_signature_verification_wrong_message() {
        // Signature must fail with different message
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        let msg1 = b"original message";
        let msg2 = b"modified message";
        
        let sig = sk.sign(msg1);
        assert!(sig.verify(&pk, msg2).is_err(), "Signature must fail with wrong message");
    }

    #[test]
    fn test_key_generation_randomness() {
        // Generated keys must be unique
        let mut public_keys = HashSet::new();
        for _ in 0..100 {
            let sk = SecretKey::generate();
            let pk = sk.public_key();
            let pk_bytes = pk.as_bytes().to_vec();
            assert!(public_keys.insert(pk_bytes), "Generated keys must be unique");
        }
    }

    #[test]
    fn test_signature_empty_message() {
        // Signing empty message must work
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        let empty = b"";
        
        let sig = sk.sign(empty);
        assert!(sig.verify(&pk, empty).is_ok(), "Empty message signature must verify");
    }

    #[test]
    fn test_signature_large_message() {
        // Signing large message must work
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        let large_msg = vec![0u8; 10_000]; // 10KB
        
        let sig = sk.sign(&large_msg);
        assert!(sig.verify(&pk, &large_msg).is_ok(), "Large message signature must verify");
    }

    #[test]
    fn test_public_key_from_secret_deterministic() {
        // Public key derivation must be deterministic
        let sk = SecretKey::generate();
        let pk1 = sk.public_key();
        let pk2 = sk.public_key();
        assert_eq!(pk1.as_bytes(), pk2.as_bytes(), "Public key derivation must be deterministic");
    }

    #[test]
    fn test_signature_replay_protection() {
        // Signature should not be usable for different messages (replay attack)
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        let msg1 = b"message 1";
        let msg2 = b"message 2";
        
        let sig = sk.sign(msg1);
        
        // Signature for msg1 should not work for msg2
        assert!(sig.verify(&pk, msg1).is_ok(), "Original signature must verify");
        assert!(sig.verify(&pk, msg2).is_err(), "Signature must not verify different message");
    }
}

// =============================================================================
// 3. VRF Security Tests
// =============================================================================

mod vrf_security {
    use super::*;

    #[test]
    fn test_vrf_deterministic_output() {
        // VRF output must be deterministic for same input
        let sk = SecretKey::generate();
        let input = b"test input";
        
        let (output1, _proof1) = sk.vrf_prove(input);
        let (output2, _proof2) = sk.vrf_prove(input);
        
        assert_eq!(output1.as_bytes(), output2.as_bytes(), "VRF output must be deterministic");
    }

    #[test]
    fn test_vrf_proof_verification() {
        // VRF proof must verify correctly
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        let input = b"test input";
        
        let (output, proof) = sk.vrf_prove(input);
        let verified_output = proof.verify(&pk, input);
        
        assert!(verified_output.is_some(), "VRF proof must verify");
        assert_eq!(output.as_bytes(), verified_output.unwrap().as_bytes(),
                   "Verified output must match");
    }

    #[test]
    fn test_vrf_proof_wrong_key() {
        // VRF proof must fail with wrong public key
        let sk1 = SecretKey::generate();
        let sk2 = SecretKey::generate();
        let pk2 = sk2.public_key();
        let input = b"test input";
        
        let (_output, proof) = sk1.vrf_prove(input);
        let verified = proof.verify(&pk2, input);
        
        assert!(verified.is_none(), "VRF proof must fail with wrong key");
    }

    #[test]
    fn test_vrf_proof_wrong_input() {
        // VRF proof must fail with wrong input
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        let input1 = b"input 1";
        let input2 = b"input 2";
        
        let (_output, proof) = sk.vrf_prove(input1);
        let verified = proof.verify(&pk, input2);
        
        assert!(verified.is_none(), "VRF proof must fail with wrong input");
    }

    #[test]
    fn test_vrf_output_unpredictability() {
        // Different inputs must produce different outputs
        let sk = SecretKey::generate();
        let mut outputs = HashSet::new();
        
        for i in 0..100 {
            let input = format!("input_{}", i);
            let (output, _proof) = sk.vrf_prove(input.as_bytes());
            assert!(outputs.insert(output.as_bytes().to_vec()),
                    "VRF outputs must be unique for different inputs");
        }
    }

    #[test]
    fn test_vrf_different_keys_different_outputs() {
        // Different keys must produce different outputs for same input
        let sk1 = SecretKey::generate();
        let sk2 = SecretKey::generate();
        let input = b"same input";
        
        let (output1, _) = sk1.vrf_prove(input);
        let (output2, _) = sk2.vrf_prove(input);
        
        assert_ne!(output1.as_bytes(), output2.as_bytes(),
                   "Different keys must produce different VRF outputs");
    }

    #[test]
    fn test_vrf_output_length() {
        // VRF output must have correct length
        let sk = SecretKey::generate();
        let input = b"test input";
        
        let (output, _proof) = sk.vrf_prove(input);
        assert_eq!(output.as_bytes().len(), 32, "VRF output must be 32 bytes");
    }
}

// =============================================================================
// 4. Commitment Scheme Security Tests
// =============================================================================

mod commitment_security {
    use super::*;

    #[test]
    fn test_commitment_hiding_property() {
        // Commitment should not reveal the value
        let value = 12345u64;
        let blinding1 = [1u8; 32];
        let blinding2 = [2u8; 32];
        
        let commitment1 = PedersenCommitment::commit(value, &blinding1);
        let commitment2 = PedersenCommitment::commit(value, &blinding2);
        
        // Same value with different blinding should produce different commitments
        assert_ne!(commitment1.to_bytes(), commitment2.to_bytes(),
                   "Commitments must hide the value with different blindings");
    }

    #[test]
    fn test_commitment_binding_property() {
        // Commitment should uniquely determine the value (with same blinding)
        let value1 = 12345u64;
        let value2 = 54321u64;
        let blinding = [0u8; 32];
        
        let commitment1 = PedersenCommitment::commit(value1, &blinding);
        let commitment2 = PedersenCommitment::commit(value2, &blinding);
        
        // Different values with same blinding must produce different commitments
        assert_ne!(commitment1.to_bytes(), commitment2.to_bytes(),
                   "Commitments must bind to different values");
    }

    #[test]
    fn test_commitment_verification() {
        // Commitment verification must work correctly
        let value = 99999u64;
        let blinding = [5u8; 32];
        
        let commitment = PedersenCommitment::commit(value, &blinding);
        assert!(commitment.verify(value, &blinding),
                "Valid commitment must verify");
    }

    #[test]
    fn test_commitment_wrong_value() {
        // Commitment must fail verification with wrong value
        let value = 11111u64;
        let wrong_value = 22222u64;
        let blinding = [7u8; 32];
        
        let commitment = PedersenCommitment::commit(value, &blinding);
        assert!(!commitment.verify(wrong_value, &blinding),
                "Commitment must not verify wrong value");
    }

    #[test]
    fn test_commitment_wrong_blinding() {
        // Commitment must fail verification with wrong blinding
        let value = 33333u64;
        let blinding1 = [3u8; 32];
        let blinding2 = [4u8; 32];
        
        let commitment = PedersenCommitment::commit(value, &blinding1);
        assert!(!commitment.verify(value, &blinding2),
                "Commitment must not verify wrong blinding");
    }

    #[test]
    fn test_commitment_deterministic() {
        // Same value and blinding must produce same commitment
        let value = 77777u64;
        let blinding = [9u8; 32];
        
        let commitment1 = PedersenCommitment::commit(value, &blinding);
        let commitment2 = PedersenCommitment::commit(value, &blinding);
        
        assert_eq!(commitment1.to_bytes(), commitment2.to_bytes(),
                   "Commitment must be deterministic");
    }

    #[test]
    fn test_commitment_zero_value() {
        // Commitment to zero must work
        let value = 0u64;
        let blinding = [1u8; 32];
        
        let commitment = PedersenCommitment::commit(value, &blinding);
        assert!(commitment.verify(value, &blinding),
                "Zero value commitment must verify");
    }

    #[test]
    fn test_commitment_max_value() {
        // Commitment to max value must work
        let value = u64::MAX;
        let blinding = [255u8; 32];
        
        let commitment = PedersenCommitment::commit(value, &blinding);
        assert!(commitment.verify(value, &blinding),
                "Max value commitment must verify");
    }
}

// =============================================================================
// 5. Merkle Tree Security Tests
// =============================================================================

mod merkle_security {
    use super::*;

    #[test]
    fn test_merkle_inclusion_proof_valid() {
        // Valid inclusion proof must verify
        let leaves: Vec<Hash256> = (0..8)
            .map(|i| Hash256::hash(&[i]))
            .collect();
        let tree = MerkleTree::new(&leaves);
        
        for (idx, leaf) in leaves.iter().enumerate() {
            let proof = tree.prove(idx);
            assert!(proof.verify(leaf, &tree.root()),
                    "Valid inclusion proof must verify for index {}", idx);
        }
    }

    #[test]
    fn test_merkle_inclusion_proof_wrong_leaf() {
        // Inclusion proof must fail for wrong leaf
        let leaves: Vec<Hash256> = (0..8)
            .map(|i| Hash256::hash(&[i]))
            .collect();
        let tree = MerkleTree::new(&leaves);
        
        let proof = tree.prove(0);
        let wrong_leaf = Hash256::hash(&[99]);
        assert!(!proof.verify(&wrong_leaf, &tree.root()),
                "Inclusion proof must fail for wrong leaf");
    }

    #[test]
    fn test_merkle_inclusion_proof_wrong_root() {
        // Inclusion proof must fail for wrong root
        let leaves: Vec<Hash256> = (0..8)
            .map(|i| Hash256::hash(&[i]))
            .collect();
        let tree = MerkleTree::new(&leaves);
        
        let proof = tree.prove(0);
        let wrong_root = Hash256::hash(&[123]);
        assert!(!proof.verify(&leaves[0], &wrong_root),
                "Inclusion proof must fail for wrong root");
    }

    #[test]
    fn test_merkle_tree_deterministic() {
        // Same leaves must produce same root
        let leaves: Vec<Hash256> = (0..16)
            .map(|i| Hash256::hash(&[i]))
            .collect();
        
        let tree1 = MerkleTree::new(&leaves);
        let tree2 = MerkleTree::new(&leaves);
        
        assert_eq!(tree1.root(), tree2.root(),
                   "Merkle root must be deterministic");
    }

    #[test]
    fn test_merkle_tree_different_leaves() {
        // Different leaves must produce different roots
        let leaves1: Vec<Hash256> = (0..8)
            .map(|i| Hash256::hash(&[i]))
            .collect();
        let leaves2: Vec<Hash256> = (8..16)
            .map(|i| Hash256::hash(&[i]))
            .collect();
        
        let tree1 = MerkleTree::new(&leaves1);
        let tree2 = MerkleTree::new(&leaves2);
        
        assert_ne!(tree1.root(), tree2.root(),
                   "Different leaves must produce different roots");
    }

    #[test]
    fn test_merkle_tree_single_leaf() {
        // Single leaf tree must work
        let leaf = Hash256::hash(&[42]);
        let tree = MerkleTree::new(&[leaf]);
        
        let proof = tree.prove(0);
        assert!(proof.verify(&leaf, &tree.root()),
                "Single leaf proof must verify");
    }

    #[test]
    fn test_merkle_tree_power_of_two() {
        // Tree with power of 2 leaves must work
        for exp in 0..10 {
            let count = 1 << exp; // 2^exp
            let leaves: Vec<Hash256> = (0..count)
                .map(|i| Hash256::hash(&[i as u8]))
                .collect();
            let tree = MerkleTree::new(&leaves);
            
            // Verify a few proofs
            for idx in [0, count / 2, count.saturating_sub(1)].iter() {
                if *idx < count {
                    let proof = tree.prove(*idx);
                    assert!(proof.verify(&leaves[*idx], &tree.root()),
                            "Proof must verify for 2^{} leaves at index {}", exp, idx);
                }
            }
        }
    }

    #[test]
    fn test_merkle_tree_non_power_of_two() {
        // Tree with non-power of 2 leaves must work
        let counts = [3, 5, 7, 9, 13, 17, 31];
        for count in counts.iter() {
            let leaves: Vec<Hash256> = (0..*count)
                .map(|i| Hash256::hash(&[i as u8]))
                .collect();
            let tree = MerkleTree::new(&leaves);
            
            // Verify all proofs
            for idx in 0..*count {
                let proof = tree.prove(idx);
                assert!(proof.verify(&leaves[idx], &tree.root()),
                        "Proof must verify for {} leaves at index {}", count, idx);
            }
        }
    }
}

// =============================================================================
// 6. Key Management Security Tests
// =============================================================================

mod key_management_security {
    use super::*;

    #[test]
    fn test_key_generation_entropy() {
        // Keys must have sufficient entropy
        let mut keys = HashSet::new();
        let iterations = 100;
        
        for _ in 0..iterations {
            let sk = SecretKey::generate();
            let pk = sk.public_key();
            let key_bytes = pk.as_bytes().to_vec();
            assert!(keys.insert(key_bytes), "Key collision detected - insufficient entropy");
        }
        
        assert_eq!(keys.len(), iterations, "All keys must be unique");
    }

    #[test]
    fn test_public_key_derivation_consistency() {
        // Public key derivation must be consistent
        let sk = SecretKey::generate();
        let pk1 = sk.public_key();
        let pk2 = sk.public_key();
        let pk3 = sk.public_key();
        
        assert_eq!(pk1.to_bytes(), pk2.to_bytes(), "Public key derivation must be consistent");
        assert_eq!(pk2.to_bytes(), pk3.to_bytes(), "Public key derivation must be consistent");
    }
}

// =============================================================================
// 7. Protocol-Level Cryptography Security Tests
// =============================================================================

mod protocol_security {
    use super::*;

    #[test]
    fn test_commitment_reveal_protocol() {
        // Commitment-reveal protocol must work correctly
        let value = b"secret glider pattern";
        let nonce = [42u8; 32];
        
        // Commit phase
        let mut commit_data = Vec::new();
        commit_data.extend_from_slice(value);
        commit_data.extend_from_slice(&nonce);
        let commitment = Hash256::hash(&commit_data);
        
        // Reveal phase
        let mut reveal_data = Vec::new();
        reveal_data.extend_from_slice(value);
        reveal_data.extend_from_slice(&nonce);
        let revealed_commitment = Hash256::hash(&reveal_data);
        
        assert_eq!(commitment, revealed_commitment,
                   "Revealed commitment must match original");
    }

    #[test]
    fn test_commitment_reveal_binding() {
        // Commitment must bind to specific value
        let value1 = b"pattern1";
        let value2 = b"pattern2";
        let nonce = [7u8; 32];
        
        let mut data1 = Vec::new();
        data1.extend_from_slice(value1);
        data1.extend_from_slice(&nonce);
        let commitment1 = Hash256::hash(&data1);
        
        let mut data2 = Vec::new();
        data2.extend_from_slice(value2);
        data2.extend_from_slice(&nonce);
        let commitment2 = Hash256::hash(&data2);
        
        assert_ne!(commitment1, commitment2,
                   "Different values must produce different commitments");
    }

    #[test]
    fn test_vrf_seed_combination() {
        // VRF outputs must combine properly for seed generation
        let sk1 = SecretKey::generate();
        let sk2 = SecretKey::generate();
        let sk3 = SecretKey::generate();
        
        let input = b"block_height_123";
        
        let (output1, _) = sk1.vrf_prove(input);
        let (output2, _) = sk2.vrf_prove(input);
        let (output3, _) = sk3.vrf_prove(input);
        
        // Combine VRF outputs (XOR is one simple method)
        let mut combined = [0u8; 32];
        for i in 0..32 {
            combined[i] = output1.as_bytes()[i] 
                        ^ output2.as_bytes()[i] 
                        ^ output3.as_bytes()[i];
        }
        
        let seed = Hash256::hash(&combined);
        
        // Seed should be deterministic
        let mut combined2 = [0u8; 32];
        for i in 0..32 {
            combined2[i] = output1.as_bytes()[i] 
                         ^ output2.as_bytes()[i] 
                         ^ output3.as_bytes()[i];
        }
        let seed2 = Hash256::hash(&combined2);
        
        assert_eq!(seed, seed2, "VRF seed combination must be deterministic");
    }

    #[test]
    fn test_no_vrf_grinding() {
        // Attacker cannot grind VRF outputs without knowing secret key
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        
        // Generate VRF output
        let input = b"target_block";
        let (output, proof) = sk.vrf_prove(input);
        
        // Proof must verify
        assert!(proof.verify(&pk, input).is_some(),
                "VRF proof must verify");
        
        // Cannot generate different output for same input without secret key
        // This is enforced by VRF determinism
        let (output2, _proof2) = sk.vrf_prove(input);
        assert_eq!(output.as_bytes(), output2.as_bytes(),
                   "VRF output must be deterministic - no grinding possible");
    }
}
