//! Integration tests for CLSAG ring signatures and key image tracking
//!
//! These tests verify the complete CLSAG integration including:
//! - Ring signature generation and verification with various ring sizes
//! - Key image uniqueness and linkability
//! - Double-spend prevention with KeyImageRegistry
//! - Concurrent access to the registry

use bitcell_crypto::{
    ClsagSecretKey, ClsagPublicKey, ClsagSignature,
    MIN_RING_SIZE, DEFAULT_RING_SIZE, MAX_RING_SIZE,
};
use bitcell_state::KeyImageRegistry;
use std::sync::Arc;
use std::thread;

/// Test basic ring signature with minimum ring size
#[test]
fn test_clsag_minimum_ring_size() {
    let sks: Vec<ClsagSecretKey> = (0..MIN_RING_SIZE)
        .map(|_| ClsagSecretKey::generate())
        .collect();
    let ring: Vec<ClsagPublicKey> = sks.iter().map(|sk| sk.public_key()).collect();

    let message = b"tournament entry";
    let signer_idx = 5;
    
    let signature = ClsagSignature::sign(&sks[signer_idx], &ring, message).unwrap();
    assert!(signature.verify(&ring, message).is_ok());
}

/// Test ring signature with default ring size
#[test]
fn test_clsag_default_ring_size() {
    let sks: Vec<ClsagSecretKey> = (0..DEFAULT_RING_SIZE)
        .map(|_| ClsagSecretKey::generate())
        .collect();
    let ring: Vec<ClsagPublicKey> = sks.iter().map(|sk| sk.public_key()).collect();

    let message = b"tournament entry";
    let signer_idx = DEFAULT_RING_SIZE / 2;
    
    let signature = ClsagSignature::sign(&sks[signer_idx], &ring, message).unwrap();
    assert!(signature.verify(&ring, message).is_ok());
}

/// Test ring signature with maximum ring size
#[test]
fn test_clsag_maximum_ring_size() {
    let sks: Vec<ClsagSecretKey> = (0..MAX_RING_SIZE)
        .map(|_| ClsagSecretKey::generate())
        .collect();
    let ring: Vec<ClsagPublicKey> = sks.iter().map(|sk| sk.public_key()).collect();

    let message = b"tournament entry";
    let signer_idx = MAX_RING_SIZE / 2;
    
    let signature = ClsagSignature::sign(&sks[signer_idx], &ring, message).unwrap();
    assert!(signature.verify(&ring, message).is_ok());
}

/// Test that ring size below minimum is rejected
#[test]
fn test_clsag_below_minimum_ring_size() {
    let sks: Vec<ClsagSecretKey> = (0..MIN_RING_SIZE - 1)
        .map(|_| ClsagSecretKey::generate())
        .collect();
    let ring: Vec<ClsagPublicKey> = sks.iter().map(|sk| sk.public_key()).collect();

    let message = b"should fail";
    let result = ClsagSignature::sign(&sks[0], &ring, message);
    
    assert!(result.is_err());
}

/// Test that ring size above maximum is rejected
#[test]
fn test_clsag_above_maximum_ring_size() {
    let sks: Vec<ClsagSecretKey> = (0..MAX_RING_SIZE + 1)
        .map(|_| ClsagSecretKey::generate())
        .collect();
    let ring: Vec<ClsagPublicKey> = sks.iter().map(|sk| sk.public_key()).collect();

    let message = b"should fail";
    let result = ClsagSignature::sign(&sks[0], &ring, message);
    
    assert!(result.is_err());
}

/// Test key image linkability: same signer produces same key image
#[test]
fn test_key_image_linkability() {
    let sks: Vec<ClsagSecretKey> = (0..DEFAULT_RING_SIZE)
        .map(|_| ClsagSecretKey::generate())
        .collect();
    let ring: Vec<ClsagPublicKey> = sks.iter().map(|sk| sk.public_key()).collect();

    let signer = &sks[5];
    
    // Same signer, different messages
    let sig1 = ClsagSignature::sign(signer, &ring, b"message1").unwrap();
    let sig2 = ClsagSignature::sign(signer, &ring, b"message2").unwrap();
    
    // Key images should be identical
    assert_eq!(sig1.key_image(), sig2.key_image());
}

/// Test key image uniqueness: different signers produce different key images
#[test]
fn test_key_image_uniqueness() {
    let sks: Vec<ClsagSecretKey> = (0..DEFAULT_RING_SIZE)
        .map(|_| ClsagSecretKey::generate())
        .collect();
    let ring: Vec<ClsagPublicKey> = sks.iter().map(|sk| sk.public_key()).collect();

    let message = b"same message";
    
    // Different signers
    let sig1 = ClsagSignature::sign(&sks[3], &ring, message).unwrap();
    let sig2 = ClsagSignature::sign(&sks[7], &ring, message).unwrap();
    
    // Key images should be different
    assert_ne!(sig1.key_image(), sig2.key_image());
}

/// Test KeyImageRegistry basic functionality
#[test]
fn test_key_image_registry_basic() {
    let mut registry = KeyImageRegistry::new();
    let sk = ClsagSecretKey::generate();
    let key_image = sk.key_image();
    
    // Initially should not contain the key image
    assert!(!registry.contains(&key_image));
    
    // Mark as used
    assert!(registry.mark(key_image).is_ok());
    
    // Now should contain it
    assert!(registry.contains(&key_image));
    
    // Second mark should fail (double-spend)
    assert!(registry.mark(key_image).is_err());
}

/// Test double-spending detection in tournament scenario
#[test]
fn test_double_spend_detection() {
    let mut registry = KeyImageRegistry::new();
    
    // Setup ring
    let sks: Vec<ClsagSecretKey> = (0..DEFAULT_RING_SIZE)
        .map(|_| ClsagSecretKey::generate())
        .collect();
    let ring: Vec<ClsagPublicKey> = sks.iter().map(|sk| sk.public_key()).collect();
    
    let signer = &sks[5];
    
    // First tournament entry
    let sig1 = ClsagSignature::sign(signer, &ring, b"tournament_1").unwrap();
    assert!(registry.check_and_mark(*sig1.key_image()).is_ok());
    
    // Attempt second entry with same key (should fail)
    let sig2 = ClsagSignature::sign(signer, &ring, b"tournament_2").unwrap();
    let result = registry.check_and_mark(*sig2.key_image());
    
    assert!(result.is_err());
    assert_eq!(sig1.key_image(), sig2.key_image());
}

/// Test multiple participants can enter once
#[test]
fn test_multiple_participants_single_entry() {
    let mut registry = KeyImageRegistry::new();
    
    // Setup ring
    let sks: Vec<ClsagSecretKey> = (0..DEFAULT_RING_SIZE)
        .map(|_| ClsagSecretKey::generate())
        .collect();
    let ring: Vec<ClsagPublicKey> = sks.iter().map(|sk| sk.public_key()).collect();
    
    let message = b"tournament_entry";
    
    // Each participant signs and registers
    for (idx, sk) in sks.iter().enumerate() {
        let sig = ClsagSignature::sign(sk, &ring, message).unwrap();
        let result = registry.check_and_mark(*sig.key_image());
        assert!(result.is_ok(), "Participant {} failed to register", idx);
    }
    
    // All should be registered
    assert_eq!(registry.len(), DEFAULT_RING_SIZE);
    
    // Attempting to register any again should fail
    for sk in &sks {
        let sig = ClsagSignature::sign(sk, &ring, message).unwrap();
        let result = registry.check_and_mark(*sig.key_image());
        assert!(result.is_err(), "Double-spend not detected");
    }
}

/// Test concurrent access to KeyImageRegistry
#[test]
fn test_concurrent_key_image_registry() {
    let registry = KeyImageRegistry::new_shared();
    let num_threads = 10;
    let entries_per_thread = 5;
    
    let mut handles = vec![];
    
    for thread_id in 0..num_threads {
        let reg = Arc::clone(&registry);
        
        let handle = thread::spawn(move || {
            let mut success_count = 0;
            
            for i in 0..entries_per_thread {
                let sk = ClsagSecretKey::generate();
                let ki = sk.key_image();
                
                let mut r = reg.lock().unwrap();
                if r.check_and_mark(ki).is_ok() {
                    success_count += 1;
                }
            }
            
            success_count
        });
        
        handles.push(handle);
    }
    
    let mut total_registered = 0;
    for handle in handles {
        total_registered += handle.join().unwrap();
    }
    
    // All entries should have succeeded (no conflicts expected)
    assert_eq!(total_registered, num_threads * entries_per_thread);
    
    let reg = registry.lock().unwrap();
    assert_eq!(reg.len(), num_threads * entries_per_thread);
}

/// Test concurrent double-spend attempts
#[test]
fn test_concurrent_double_spend_prevention() {
    let registry = KeyImageRegistry::new_shared();
    
    // Generate a single key that multiple threads will try to use
    let sk = ClsagSecretKey::generate();
    let ki = sk.key_image();
    
    let num_threads = 10;
    let mut handles = vec![];
    
    for _ in 0..num_threads {
        let reg = Arc::clone(&registry);
        
        let handle = thread::spawn(move || {
            let mut r = reg.lock().unwrap();
            r.check_and_mark(ki)
        });
        
        handles.push(handle);
    }
    
    let mut successes = 0;
    let mut failures = 0;
    
    for handle in handles {
        match handle.join().unwrap() {
            Ok(_) => successes += 1,
            Err(_) => failures += 1,
        }
    }
    
    // Exactly one should succeed, rest should fail
    assert_eq!(successes, 1);
    assert_eq!(failures, num_threads - 1);
    
    let reg = registry.lock().unwrap();
    assert_eq!(reg.len(), 1);
}

/// Test anonymity: verifier cannot determine which ring member signed
#[test]
fn test_ring_signature_anonymity() {
    let sks: Vec<ClsagSecretKey> = (0..DEFAULT_RING_SIZE)
        .map(|_| ClsagSecretKey::generate())
        .collect();
    let ring: Vec<ClsagPublicKey> = sks.iter().map(|sk| sk.public_key()).collect();

    let message = b"anonymous transaction";
    
    // Multiple different signers
    let sig1 = ClsagSignature::sign(&sks[2], &ring, message).unwrap();
    let sig2 = ClsagSignature::sign(&sks[8], &ring, message).unwrap();
    let sig3 = ClsagSignature::sign(&sks[13], &ring, message).unwrap();
    
    // All should verify successfully against the same ring
    assert!(sig1.verify(&ring, message).is_ok());
    assert!(sig2.verify(&ring, message).is_ok());
    assert!(sig3.verify(&ring, message).is_ok());
    
    // But key images should be different
    assert_ne!(sig1.key_image(), sig2.key_image());
    assert_ne!(sig2.key_image(), sig3.key_image());
    assert_ne!(sig1.key_image(), sig3.key_image());
}

/// Test signature fails with wrong ring
#[test]
fn test_signature_verification_wrong_ring() {
    let sks: Vec<ClsagSecretKey> = (0..DEFAULT_RING_SIZE + 5)
        .map(|_| ClsagSecretKey::generate())
        .collect();
    
    let ring1: Vec<ClsagPublicKey> = sks[0..DEFAULT_RING_SIZE]
        .iter()
        .map(|sk| sk.public_key())
        .collect();
    
    let ring2: Vec<ClsagPublicKey> = sks[5..DEFAULT_RING_SIZE + 5]
        .iter()
        .map(|sk| sk.public_key())
        .collect();
    
    let message = b"test message";
    let sig = ClsagSignature::sign(&sks[5], &ring1, message).unwrap();
    
    // Verify with correct ring should succeed
    assert!(sig.verify(&ring1, message).is_ok());
    
    // Verify with wrong ring should fail
    assert!(sig.verify(&ring2, message).is_err());
}

/// Test signature fails with tampered message
#[test]
fn test_signature_verification_tampered_message() {
    let sks: Vec<ClsagSecretKey> = (0..DEFAULT_RING_SIZE)
        .map(|_| ClsagSecretKey::generate())
        .collect();
    let ring: Vec<ClsagPublicKey> = sks.iter().map(|sk| sk.public_key()).collect();
    
    let original = b"original message";
    let tampered = b"tampered message";
    
    let sig = ClsagSignature::sign(&sks[7], &ring, original).unwrap();
    
    // Verify with original message should succeed
    assert!(sig.verify(&ring, original).is_ok());
    
    // Verify with tampered message should fail
    assert!(sig.verify(&ring, tampered).is_err());
}

/// Test performance characteristic: larger rings take longer
#[test]
fn test_ring_size_performance_scaling() {
    use std::time::Instant;
    
    let sizes = vec![MIN_RING_SIZE, DEFAULT_RING_SIZE, MAX_RING_SIZE];
    let mut timings = vec![];
    
    for size in sizes {
        let sks: Vec<ClsagSecretKey> = (0..size)
            .map(|_| ClsagSecretKey::generate())
            .collect();
        let ring: Vec<ClsagPublicKey> = sks.iter().map(|sk| sk.public_key()).collect();
        
        let message = b"benchmark";
        
        let start = Instant::now();
        let sig = ClsagSignature::sign(&sks[size / 2], &ring, message).unwrap();
        let sign_time = start.elapsed();
        
        let start = Instant::now();
        sig.verify(&ring, message).unwrap();
        let verify_time = start.elapsed();
        
        timings.push((size, sign_time, verify_time));
        
        println!(
            "Ring size {}: sign={:?}, verify={:?}",
            size, sign_time, verify_time
        );
    }
    
    // Sanity check: larger rings should take longer (though this is probabilistic)
    // We just verify all operations complete in reasonable time
    for (size, sign_time, verify_time) in timings {
        assert!(
            sign_time.as_millis() < 100,
            "Sign time for ring size {} too slow: {:?}",
            size,
            sign_time
        );
        assert!(
            verify_time.as_millis() < 100,
            "Verify time for ring size {} too slow: {:?}",
            size,
            verify_time
        );
    }
}
