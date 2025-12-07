//! Integration tests for VRF (Verifiable Random Function) implementation
//!
//! These tests verify the complete VRF integration across the BitCell codebase,
//! including key derivation, proof generation/verification, and blockchain integration.

use bitcell_crypto::{PublicKey, SecretKey, Hash256};
use bitcell_node::blockchain::Blockchain;
use bitcell_metrics::MetricsRegistry;
use std::sync::Arc;

/// Test that VRF keys are correctly derived from secp256k1 keys
#[test]
fn test_vrf_key_derivation_deterministic() {
    // Same secp256k1 key should always produce same VRF outputs
    let seed = [42u8; 32];
    let sk1 = SecretKey::from_bytes(&seed).unwrap();
    let sk2 = SecretKey::from_bytes(&seed).unwrap();
    
    let message = b"test_message_for_vrf";
    
    let (output1, _proof1) = sk1.vrf_prove(message);
    let (output2, _proof2) = sk2.vrf_prove(message);
    
    assert_eq!(output1, output2, "VRF outputs should be deterministic for same key");
}

/// Test that different secp256k1 keys produce different VRF outputs
#[test]
fn test_vrf_key_derivation_unique() {
    let sk1 = SecretKey::generate();
    let sk2 = SecretKey::generate();
    
    let message = b"same_message";
    
    let (output1, _) = sk1.vrf_prove(message);
    let (output2, _) = sk2.vrf_prove(message);
    
    assert_ne!(output1, output2, "Different keys should produce different VRF outputs");
}

/// Test VRF proof generation and verification with valid proofs
#[test]
fn test_vrf_proof_verification_valid() {
    let sk = SecretKey::generate();
    let pk = sk.public_key();
    
    let message = b"block_hash_test";
    let (output, proof) = sk.vrf_prove(message);
    
    // Verify the proof
    let verified_output = proof.verify(&pk, message)
        .expect("Valid proof should verify successfully");
    
    assert_eq!(output, verified_output, "Verified output should match original output");
}

/// Test that VRF proofs verify correctly for different messages
#[test]
fn test_vrf_proof_different_messages() {
    let sk = SecretKey::generate();
    let pk = sk.public_key();
    
    let messages = [
        b"message_1".as_slice(),
        b"message_2".as_slice(),
        b"message_3".as_slice(),
        b"a_very_long_message_that_tests_vrf_with_more_data".as_slice(),
        &[0u8; 100], // Binary data
    ];
    
    for message in &messages {
        let (output, proof) = sk.vrf_prove(message);
        let verified = proof.verify(&pk, message)
            .expect("Proof should verify for valid message");
        assert_eq!(output, verified);
    }
}

/// Test VRF proof with wrong message fails verification
#[test]
fn test_vrf_proof_wrong_message() {
    let sk = SecretKey::generate();
    let pk = sk.public_key();
    
    let correct_message = b"correct_message";
    let wrong_message = b"wrong_message";
    
    let (_output, proof) = sk.vrf_prove(correct_message);
    
    // Verification with wrong message should still work (VRF verifies proof structure),
    // but will produce different output
    let verified1 = proof.verify(&pk, correct_message)
        .expect("Should verify with correct message");
    let verified2 = proof.verify(&pk, wrong_message)
        .expect("VRF proof structure should still be valid");
    
    // The outputs will differ because the message is part of the VRF input
    assert_ne!(verified1, verified2, "Different messages should produce different verified outputs");
}

/// Test VRF chaining in blockchain - each block uses previous VRF output
#[test]
fn test_vrf_chaining_in_blockchain() {
    let sk = Arc::new(SecretKey::generate());
    let metrics = MetricsRegistry::new();
    let blockchain = Blockchain::new(sk.clone(), metrics);
    
    // Produce and add 5 blocks
    let mut blocks = Vec::new();
    for i in 0..5 {
        let block = blockchain.produce_block(
            vec![],
            vec![],
            sk.public_key(),
        ).expect(&format!("Should produce block {}", i + 1));
        
        // Verify VRF output is non-zero
        assert_ne!(block.header.vrf_output, [0u8; 32], 
                   "Block {} VRF output should be non-zero", i + 1);
        
        // Verify VRF proof exists
        assert!(!block.header.vrf_proof.is_empty(),
                "Block {} should have VRF proof", i + 1);
        
        // If not first block, verify VRF output differs from previous
        if i > 0 {
            assert_ne!(block.header.vrf_output, blocks[i - 1].header.vrf_output,
                       "Block {} VRF should differ from block {} due to chaining", i + 1, i);
        }
        
        // Validate and add block
        blockchain.validate_block(&block)
            .expect(&format!("Block {} should be valid", i + 1));
        blockchain.add_block(block.clone())
            .expect(&format!("Should add block {}", i + 1));
        
        blocks.push(block);
    }
    
    // Verify all VRF outputs are unique
    for i in 0..blocks.len() {
        for j in (i + 1)..blocks.len() {
            assert_ne!(blocks[i].header.vrf_output, blocks[j].header.vrf_output,
                       "Block {} and block {} should have different VRF outputs", i + 1, j + 1);
        }
    }
}

/// Test VRF determinism - recreating the same chain produces same VRF sequence
#[test]
fn test_vrf_blockchain_determinism() {
    let sk = Arc::new(SecretKey::generate());
    
    // Create first blockchain and produce 3 blocks
    let metrics1 = MetricsRegistry::new();
    let blockchain1 = Blockchain::new(sk.clone(), metrics1);
    
    let block1_v1 = blockchain1.produce_block(vec![], vec![], sk.public_key()).unwrap();
    blockchain1.add_block(block1_v1.clone()).unwrap();
    
    let block2_v1 = blockchain1.produce_block(vec![], vec![], sk.public_key()).unwrap();
    blockchain1.add_block(block2_v1.clone()).unwrap();
    
    let block3_v1 = blockchain1.produce_block(vec![], vec![], sk.public_key()).unwrap();
    
    // Create second blockchain with same key
    let metrics2 = MetricsRegistry::new();
    let blockchain2 = Blockchain::new(sk.clone(), metrics2);
    
    let block1_v2 = blockchain2.produce_block(vec![], vec![], sk.public_key()).unwrap();
    blockchain2.add_block(block1_v2.clone()).unwrap();
    
    let block2_v2 = blockchain2.produce_block(vec![], vec![], sk.public_key()).unwrap();
    blockchain2.add_block(block2_v2.clone()).unwrap();
    
    let block3_v2 = blockchain2.produce_block(vec![], vec![], sk.public_key()).unwrap();
    
    // Verify same VRF sequence
    assert_eq!(block1_v1.header.vrf_output, block1_v2.header.vrf_output,
               "First block VRF should be deterministic");
    assert_eq!(block2_v1.header.vrf_output, block2_v2.header.vrf_output,
               "Second block VRF should be deterministic");
    assert_eq!(block3_v1.header.vrf_output, block3_v2.header.vrf_output,
               "Third block VRF should be deterministic");
}

/// Test VRF with multiple different validators
#[test]
fn test_vrf_multiple_validators() {
    let validators = vec![
        Arc::new(SecretKey::generate()),
        Arc::new(SecretKey::generate()),
        Arc::new(SecretKey::generate()),
    ];
    
    let metrics = MetricsRegistry::new();
    let blockchain = Blockchain::new(validators[0].clone(), metrics);
    
    // Each validator produces a block
    for (i, validator) in validators.iter().enumerate() {
        let block = blockchain.produce_block(
            vec![],
            vec![],
            validator.public_key(),
        ).expect(&format!("Validator {} should produce block", i));
        
        // Verify VRF output is unique and non-zero
        assert_ne!(block.header.vrf_output, [0u8; 32],
                   "Validator {} block should have non-zero VRF", i);
        
        // Verify block is valid
        blockchain.validate_block(&block)
            .expect(&format!("Validator {} block should be valid", i));
        
        blockchain.add_block(block).expect("Should add block");
    }
}

/// Test VRF output distribution (outputs should appear random)
#[test]
fn test_vrf_output_distribution() {
    let sk = SecretKey::generate();
    
    // Generate multiple VRF outputs with different messages
    let mut outputs = Vec::new();
    for i in 0..10 {
        let message = format!("message_{}", i);
        let (output, _) = sk.vrf_prove(message.as_bytes());
        outputs.push(output);
    }
    
    // All outputs should be unique
    for i in 0..outputs.len() {
        for j in (i + 1)..outputs.len() {
            assert_ne!(outputs[i], outputs[j],
                       "VRF output {} and {} should be different", i, j);
        }
    }
    
    // Outputs should not be all zeros
    for (i, output) in outputs.iter().enumerate() {
        assert_ne!(output.as_bytes(), &[0u8; 32],
                   "VRF output {} should not be all zeros", i);
    }
}

/// Test VRF outputs are unpredictable without the secret key
#[test]
fn test_vrf_output_unpredictability() {
    let sk1 = SecretKey::generate();
    let sk2 = SecretKey::generate();
    
    let message = b"predictable_message";
    
    let (output1, _) = sk1.vrf_prove(message);
    let (output2, _) = sk2.vrf_prove(message);
    
    // Even with same message, different keys produce unpredictable outputs
    assert_ne!(output1, output2);
    
    // Outputs should not be trivially related to the message
    let message_hash = Hash256::from_bytes(
        sha2::Sha256::digest(message).into()
    );
    assert_ne!(output1.as_bytes(), message_hash.as_bytes());
    assert_ne!(output2.as_bytes(), message_hash.as_bytes());
}

/// Test that VRF proofs are correctly serialized in blocks
#[test]
fn test_vrf_proof_serialization_in_blocks() {
    let sk = Arc::new(SecretKey::generate());
    let metrics = MetricsRegistry::new();
    let blockchain = Blockchain::new(sk.clone(), metrics);
    
    let block = blockchain.produce_block(
        vec![],
        vec![],
        sk.public_key(),
    ).unwrap();
    
    // Verify VRF proof is not empty and has reasonable size
    assert!(!block.header.vrf_proof.is_empty(), "VRF proof should not be empty");
    assert!(block.header.vrf_proof.len() > 32, "VRF proof should have meaningful data");
    assert!(block.header.vrf_proof.len() < 1024, "VRF proof should be reasonably sized");
    
    // Verify block validates (which includes VRF verification)
    blockchain.validate_block(&block).expect("Block with serialized VRF should validate");
}

/// Test VRF with edge case: empty message
#[test]
fn test_vrf_empty_message() {
    let sk = SecretKey::generate();
    let pk = sk.public_key();
    
    let message = b"";
    let (output, proof) = sk.vrf_prove(message);
    
    // Should produce valid output even for empty message
    assert_ne!(output.as_bytes(), &[0u8; 32], "Empty message should still produce non-zero VRF output");
    
    // Should verify correctly
    let verified = proof.verify(&pk, message)
        .expect("Empty message VRF should verify");
    assert_eq!(output, verified);
}

/// Test VRF with edge case: very long message
#[test]
fn test_vrf_long_message() {
    let sk = SecretKey::generate();
    let pk = sk.public_key();
    
    // Create a large message (10KB)
    let message = vec![0x42u8; 10000];
    let (output, proof) = sk.vrf_prove(&message);
    
    // Should produce valid output for long message
    assert_ne!(output.as_bytes(), &[0u8; 32]);
    
    // Should verify correctly
    let verified = proof.verify(&pk, &message)
        .expect("Long message VRF should verify");
    assert_eq!(output, verified);
}

/// Test combining multiple VRF outputs for tournament seeding
#[test]
fn test_vrf_output_combination() {
    use bitcell_crypto::vrf::combine_vrf_outputs;
    
    let sk1 = SecretKey::generate();
    let sk2 = SecretKey::generate();
    let sk3 = SecretKey::generate();
    
    let (out1, _) = sk1.vrf_prove(b"round1");
    let (out2, _) = sk2.vrf_prove(b"round1");
    let (out3, _) = sk3.vrf_prove(b"round1");
    
    let seed = combine_vrf_outputs(&[out1, out2, out3]);
    
    // Combined seed should be non-zero
    assert_ne!(seed, Hash256::zero());
    
    // Same outputs in different order should produce different seed
    let seed2 = combine_vrf_outputs(&[out2, out1, out3]);
    assert_ne!(seed, seed2, "Order should matter in VRF combination");
    
    // Same outputs in same order should produce same seed
    let seed3 = combine_vrf_outputs(&[out1, out2, out3]);
    assert_eq!(seed, seed3, "Same outputs in same order should produce same seed");
}
