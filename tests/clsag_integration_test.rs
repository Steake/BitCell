//! Integration tests for CLSAG signatures with tournament protocol
//!
//! These tests demonstrate how CLSAG ring signatures integrate with
//! the tournament system for anonymous commitments and double-spend prevention.

#[cfg(test)]
mod tests {
    use bitcell_crypto::{
        ClsagSecretKey, ClsagSignature, Hash256, MIN_RING_SIZE, DEFAULT_RING_SIZE,
    };
    use bitcell_state::KeyImageRegistry;

    #[test]
    fn test_tournament_commitment_flow() {
        // Simulate a tournament with eligible miners
        let mut miners = vec![];
        let mut ring = vec![];
        
        for _ in 0..DEFAULT_RING_SIZE {
            let sk = ClsagSecretKey::generate();
            miners.push(sk.clone());
            ring.push(sk.public_key());
        }

        // Miner 0 creates a commitment
        let commitment_data = b"glider_pattern_hash||nonce";
        let commitment_hash = Hash256::hash(commitment_data);

        // Sign the commitment anonymously
        let signature = ClsagSignature::sign(
            &miners[0],
            &ring,
            commitment_hash.as_bytes(),
        ).unwrap();

        // Verify the signature (anyone can verify, but can't identify signer)
        assert!(signature.verify(&ring, commitment_hash.as_bytes()).is_ok());

        // Track key image to prevent double commitments
        let mut key_images = KeyImageRegistry::new();
        assert!(key_images.mark_used(*signature.key_image()).is_ok());
    }

    #[test]
    fn test_double_commitment_prevention() {
        // Setup
        let mut miners = vec![];
        let mut ring = vec![];
        
        for _ in 0..DEFAULT_RING_SIZE {
            let sk = ClsagSecretKey::generate();
            miners.push(sk.clone());
            ring.push(sk.public_key());
        }

        let mut key_images = KeyImageRegistry::new();

        // First commitment from miner 0
        let commitment1 = b"first_commitment";
        let hash1 = Hash256::hash(commitment1);
        let sig1 = ClsagSignature::sign(&miners[0], &ring, hash1.as_bytes()).unwrap();
        
        // Should succeed
        assert!(key_images.mark_used(*sig1.key_image()).is_ok());

        // Second commitment from same miner (different message)
        let commitment2 = b"second_commitment";
        let hash2 = Hash256::hash(commitment2);
        let sig2 = ClsagSignature::sign(&miners[0], &ring, hash2.as_bytes()).unwrap();
        
        // Should fail - same key image
        assert!(key_images.mark_used(*sig2.key_image()).is_err());
        assert_eq!(sig1.key_image(), sig2.key_image());
    }

    #[test]
    fn test_multiple_miners_same_round() {
        // Setup tournament with multiple miners
        let mut miners = vec![];
        let mut ring = vec![];
        
        for _ in 0..DEFAULT_RING_SIZE {
            let sk = ClsagSecretKey::generate();
            miners.push(sk.clone());
            ring.push(sk.public_key());
        }

        let mut key_images = KeyImageRegistry::new();

        // Each miner submits one commitment
        for (i, miner) in miners.iter().enumerate() {
            let commitment_data = format!("miner_{}_commitment", i);
            let hash = Hash256::hash(commitment_data.as_bytes());
            let sig = ClsagSignature::sign(miner, &ring, hash.as_bytes()).unwrap();
            
            // Each miner's first commitment should succeed
            assert!(key_images.mark_used(*sig.key_image()).is_ok());
        }

        // Verify all key images are tracked
        assert_eq!(key_images.len(), DEFAULT_RING_SIZE);
    }

    #[test]
    fn test_signature_with_minimum_ring() {
        // Test with minimum ring size (edge case)
        let mut miners = vec![];
        let mut ring = vec![];
        
        for _ in 0..MIN_RING_SIZE {
            let sk = ClsagSecretKey::generate();
            miners.push(sk.clone());
            ring.push(sk.public_key());
        }

        let commitment = b"min_ring_commitment";
        let hash = Hash256::hash(commitment);
        let sig = ClsagSignature::sign(&miners[5], &ring, hash.as_bytes()).unwrap();
        
        assert!(sig.verify(&ring, hash.as_bytes()).is_ok());
    }

    #[test]
    fn test_invalid_ring_size_rejection() {
        // Try to create signature with too small ring
        let sk = ClsagSecretKey::generate();
        let mut ring = vec![];
        
        // Only 10 members (below minimum of 11)
        for _ in 0..10 {
            ring.push(ClsagSecretKey::generate().public_key());
        }
        ring[0] = sk.public_key();

        let result = ClsagSignature::sign(&sk, &ring, b"test");
        assert!(result.is_err());
    }

    #[test]
    fn test_anonymity_set_verification() {
        // Verify that you can't determine which miner signed
        let mut miners = vec![];
        let mut ring = vec![];
        
        for _ in 0..DEFAULT_RING_SIZE {
            let sk = ClsagSecretKey::generate();
            miners.push(sk.clone());
            ring.push(sk.public_key());
        }

        // Miner 7 creates a signature
        let message = b"anonymous_message";
        let sig = ClsagSignature::sign(&miners[7], &ring, message).unwrap();

        // Verification succeeds but doesn't reveal signer
        assert!(sig.verify(&ring, message).is_ok());

        // Key image is unique to miner 7
        let ki_7 = miners[7].key_image();
        assert_eq!(*sig.key_image(), ki_7);

        // But different from all other miners
        for (i, miner) in miners.iter().enumerate() {
            if i != 7 {
                assert_ne!(*sig.key_image(), miner.key_image());
            }
        }
    }

    #[test]
    fn test_multi_round_tournament() {
        // Simulate multiple tournament rounds
        let mut miners = vec![];
        let mut ring = vec![];
        
        for _ in 0..DEFAULT_RING_SIZE {
            let sk = ClsagSecretKey::generate();
            miners.push(sk.clone());
            ring.push(sk.public_key());
        }

        // Round 1
        let mut round1_key_images = KeyImageRegistry::new();
        for miner in &miners {
            let commitment = b"round_1_commitment";
            let sig = ClsagSignature::sign(miner, &ring, commitment).unwrap();
            assert!(round1_key_images.mark_used(*sig.key_image()).is_ok());
        }

        // Round 2 (new key images needed - simulate with new keys)
        // In practice, you'd clear key images between rounds or use per-round registries
        let mut round2_key_images = KeyImageRegistry::new();
        for miner in &miners {
            let commitment = b"round_2_commitment";
            let sig = ClsagSignature::sign(miner, &ring, commitment).unwrap();
            assert!(round2_key_images.mark_used(*sig.key_image()).is_ok());
        }

        // Same miner in both rounds uses same key image for same key
        let test_commitment_1 = b"round_1_commitment";
        let test_commitment_2 = b"round_2_commitment";
        let sig1 = ClsagSignature::sign(&miners[0], &ring, test_commitment_1).unwrap();
        let sig2 = ClsagSignature::sign(&miners[0], &ring, test_commitment_2).unwrap();
        assert_eq!(sig1.key_image(), sig2.key_image());
    }

    #[test]
    fn test_chain_reorganization_rollback() {
        // Simulate a chain reorganization requiring key image rollback
        let sk = ClsagSecretKey::generate();
        let mut ring = vec![sk.public_key()];
        
        for _ in 1..DEFAULT_RING_SIZE {
            ring.push(ClsagSecretKey::generate().public_key());
        }

        let mut key_images = KeyImageRegistry::new();

        // Block 1: Commitment accepted
        let commitment1 = b"block_1_commitment";
        let sig1 = ClsagSignature::sign(&sk, &ring, commitment1).unwrap();
        assert!(key_images.mark_used(*sig1.key_image()).is_ok());

        // Chain reorg: Rollback block 1
        assert!(key_images.remove(sig1.key_image()));

        // After rollback, same key image can be used again
        let commitment2 = b"block_1_alternative_commitment";
        let sig2 = ClsagSignature::sign(&sk, &ring, commitment2).unwrap();
        assert!(key_images.mark_used(*sig2.key_image()).is_ok());
    }

    #[test]
    fn test_signature_tampering_detection() {
        // Verify that tampering with signature is detected
        let mut miners = vec![];
        let mut ring = vec![];
        
        for _ in 0..DEFAULT_RING_SIZE {
            let sk = ClsagSecretKey::generate();
            miners.push(sk.clone());
            ring.push(sk.public_key());
        }

        let message = b"original_message";
        let sig = ClsagSignature::sign(&miners[0], &ring, message).unwrap();

        // Signature verifies with correct message
        assert!(sig.verify(&ring, message).is_ok());

        // Signature fails with tampered message
        let tampered_message = b"tampered_message";
        assert!(sig.verify(&ring, tampered_message).is_err());
    }

    #[test]
    fn test_concurrent_commitments() {
        // Test that multiple miners can commit simultaneously
        use std::sync::{Arc, Mutex};
        use std::thread;

        let mut miners = vec![];
        let mut ring = vec![];
        
        for _ in 0..DEFAULT_RING_SIZE {
            let sk = ClsagSecretKey::generate();
            miners.push(sk.clone());
            ring.push(sk.public_key());
        }

        let key_images = Arc::new(Mutex::new(KeyImageRegistry::new()));
        let ring = Arc::new(ring);
        
        let mut handles = vec![];
        
        // Spawn threads for each miner
        for (i, miner) in miners.into_iter().enumerate() {
            let key_images = Arc::clone(&key_images);
            let ring = Arc::clone(&ring);
            
            let handle = thread::spawn(move || {
                let commitment = format!("miner_{}_commitment", i);
                let hash = Hash256::hash(commitment.as_bytes());
                let sig = ClsagSignature::sign(&miner, &ring, hash.as_bytes()).unwrap();
                
                // Atomically mark as used
                let mut ki_registry = key_images.lock().unwrap();
                ki_registry
                    .mark_used(*sig.key_image())
                    .expect("Each miner should have unique key image in concurrent test");
            });
            
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all commitments were recorded
        let ki_registry = key_images.lock().unwrap();
        assert_eq!(ki_registry.len(), DEFAULT_RING_SIZE);
    }

    #[test]
    fn test_concurrent_double_spend_prevention() {
        // Test that concurrent attempts to use the same key image are properly prevented
        use std::sync::{Arc, Mutex};
        use std::thread;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let sk = ClsagSecretKey::generate();
        let mut ring = vec![sk.public_key()];
        
        for _ in 1..DEFAULT_RING_SIZE {
            ring.push(ClsagSecretKey::generate().public_key());
        }

        let key_images = Arc::new(Mutex::new(KeyImageRegistry::new()));
        let ring = Arc::new(ring);
        let sk = Arc::new(sk);
        let success_count = Arc::new(AtomicUsize::new(0));
        let failure_count = Arc::new(AtomicUsize::new(0));
        
        let mut handles = vec![];
        
        // Spawn multiple threads trying to use the same key
        for i in 0..10 {
            let key_images = Arc::clone(&key_images);
            let ring = Arc::clone(&ring);
            let sk = Arc::clone(&sk);
            let success_count = Arc::clone(&success_count);
            let failure_count = Arc::clone(&failure_count);
            
            let handle = thread::spawn(move || {
                let commitment = format!("attempt_{}_commitment", i);
                let hash = Hash256::hash(commitment.as_bytes());
                let sig = ClsagSignature::sign(&*sk, &ring, hash.as_bytes()).unwrap();
                
                // Try to atomically mark as used
                let mut ki_registry = key_images.lock().unwrap();
                match ki_registry.mark_used(*sig.key_image()) {
                    Ok(()) => {
                        success_count.fetch_add(1, Ordering::SeqCst);
                    }
                    Err(_) => {
                        failure_count.fetch_add(1, Ordering::SeqCst);
                    }
                }
            });
            
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Exactly one thread should succeed, 9 should fail
        assert_eq!(success_count.load(Ordering::SeqCst), 1);
        assert_eq!(failure_count.load(Ordering::SeqCst), 9);
    }
}
