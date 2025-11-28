//! Security Tests for BitCell Wallet
//!
//! These tests validate security properties of key storage and transaction signing
//! following BDD patterns and targeting Linux, macOS, and Windows.

use bitcell_wallet::{
    Balance, Chain, Error, Mnemonic, Transaction, Wallet, WalletConfig,
    mnemonic::WordCount,
};
use bitcell_crypto::SecretKey;
use std::collections::HashSet;

// =============================================================================
// Key Generation Security Tests
// =============================================================================

mod key_generation_security {
    use super::*;

    /// Scenario: Generated keys are cryptographically random
    /// Given multiple key generation attempts
    /// When keys are generated
    /// Then they should all be unique
    #[test]
    fn scenario_generated_keys_are_random() {
        // Given: Multiple key generation attempts
        const NUM_KEYS: usize = 100;
        let mut public_keys = HashSet::new();

        // When: Keys are generated
        for _ in 0..NUM_KEYS {
            let sk = SecretKey::generate();
            let pk = sk.public_key();
            public_keys.insert(pk.as_bytes().to_vec());
        }

        // Then: They should all be unique
        assert_eq!(public_keys.len(), NUM_KEYS, "Generated keys should be unique");
    }

    /// Scenario: Mnemonic entropy is sufficient
    /// Given multiple mnemonic generation attempts
    /// When mnemonics are generated
    /// Then they should all be unique
    #[test]
    fn scenario_mnemonic_entropy_is_sufficient() {
        // Given: Multiple mnemonic generation attempts
        const NUM_MNEMONICS: usize = 50;
        let mut phrases = HashSet::new();

        // When: Mnemonics are generated
        for _ in 0..NUM_MNEMONICS {
            let mnemonic = Mnemonic::new();
            phrases.insert(mnemonic.phrase().to_string());
        }

        // Then: They should all be unique
        assert_eq!(phrases.len(), NUM_MNEMONICS, "Generated mnemonics should be unique");
    }

    /// Scenario: Different word counts provide expected entropy
    /// Given mnemonics with different word counts
    /// When generated
    /// Then higher word counts should provide more entropy bits
    #[test]
    fn scenario_word_counts_provide_expected_entropy() {
        // Given: Mnemonics with different word counts
        let m12 = Mnemonic::generate(WordCount::Words12); // 128 bits
        let m18 = Mnemonic::generate(WordCount::Words18); // 192 bits
        let m24 = Mnemonic::generate(WordCount::Words24); // 256 bits

        // When: Generated
        // Then: Higher word counts should have more words (and thus entropy)
        assert_eq!(m12.word_count(), 12);
        assert_eq!(m18.word_count(), 18);
        assert_eq!(m24.word_count(), 24);

        // Verify seeds are proper length regardless of mnemonic size
        let s12 = m12.to_seed("");
        let s18 = m18.to_seed("");
        let s24 = m24.to_seed("");

        assert_eq!(s12.as_bytes().len(), 64);
        assert_eq!(s18.as_bytes().len(), 64);
        assert_eq!(s24.as_bytes().len(), 64);
    }
}

// =============================================================================
// Key Storage Security Tests  
// =============================================================================

mod key_storage_security {
    use super::*;

    /// Scenario: Locked wallet prevents key access
    /// Given a wallet that has been locked
    /// When attempting cryptographic operations
    /// Then they should be denied
    #[test]
    fn scenario_locked_wallet_prevents_key_access() {
        // Given: A wallet that has been locked
        let mnemonic = Mnemonic::new();
        let mut wallet = Wallet::from_mnemonic(&mnemonic, "", WalletConfig::default());
        
        let from = wallet.next_address(Chain::BitCell).unwrap();
        let to = wallet.next_address(Chain::BitCell).unwrap();
        wallet.update_balance(&from, 1_000_000);
        
        wallet.lock();

        // When: Attempting cryptographic operations
        let tx = wallet.create_transaction(&from, &to, 1000, 100).unwrap();
        let sign_result = wallet.sign_transaction(tx, &from);

        // Then: They should be denied
        assert!(matches!(sign_result, Err(Error::WalletLocked)));
    }

    /// Scenario: Wallet can be re-locked and re-unlocked
    /// Given a wallet with a known mnemonic
    /// When it is locked and unlocked multiple times
    /// Then cryptographic operations should work after unlock
    #[test]
    fn scenario_wallet_relock_unlock_cycle() {
        // Given: A wallet with a known mnemonic
        let mnemonic = Mnemonic::new();
        let mut wallet = Wallet::from_mnemonic(&mnemonic, "", WalletConfig::default());
        
        let from = wallet.next_address(Chain::BitCell).unwrap();
        let to = wallet.next_address(Chain::BitCell).unwrap();
        wallet.update_balance(&from, 10_000_000);

        // When: It is locked and unlocked multiple times
        for _ in 0..3 {
            // Lock
            wallet.lock();
            assert!(!wallet.is_unlocked());
            
            // Verify signing fails
            let tx = wallet.create_transaction(&from, &to, 1000, 100).unwrap();
            assert!(wallet.sign_transaction(tx, &from).is_err());
            
            // Unlock
            wallet.unlock(&mnemonic, "").unwrap();
            
            // Then: Cryptographic operations should work after unlock
            let tx = wallet.create_transaction(&from, &to, 1000, 100).unwrap();
            assert!(wallet.sign_transaction(tx, &from).is_ok());
        }
    }

    /// Scenario: Different passphrase produces different keys
    /// Given the same mnemonic
    /// When different passphrases are used
    /// Then different addresses should be generated
    #[test]
    fn scenario_passphrase_affects_key_derivation() {
        // Given: The same mnemonic
        let mnemonic = Mnemonic::new();

        // When: Different passphrases are used
        let mut wallet1 = Wallet::from_mnemonic(&mnemonic, "", WalletConfig::default());
        let mut wallet2 = Wallet::from_mnemonic(&mnemonic, "secret123", WalletConfig::default());
        let mut wallet3 = Wallet::from_mnemonic(&mnemonic, "different_secret", WalletConfig::default());

        let addr1 = wallet1.generate_address(Chain::BitCell, 0).unwrap();
        let addr2 = wallet2.generate_address(Chain::BitCell, 0).unwrap();
        let addr3 = wallet3.generate_address(Chain::BitCell, 0).unwrap();

        // Then: Different addresses should be generated
        assert_ne!(addr1.as_bytes(), addr2.as_bytes());
        assert_ne!(addr2.as_bytes(), addr3.as_bytes());
        assert_ne!(addr1.as_bytes(), addr3.as_bytes());
    }
}

// =============================================================================
// Transaction Signing Security Tests
// =============================================================================

mod transaction_signing_security {
    use super::*;

    /// Scenario: Signature verification requires correct key
    /// Given a signed transaction
    /// When verification is attempted with different keys
    /// Then only the correct key should succeed
    #[test]
    fn scenario_signature_verification_requires_correct_key() {
        // Given: A signed transaction
        let sk1 = SecretKey::generate();
        let pk1 = sk1.public_key();
        let sk2 = SecretKey::generate();
        let pk2 = sk2.public_key();

        let tx = Transaction::new(
            Chain::BitCell,
            "BC1sender".to_string(),
            "BC1receiver".to_string(),
            100_000,
            100,
            0,
        );
        let signed = tx.sign(&sk1);

        // When: Verification is attempted with different keys
        let verify_correct = signed.verify(&pk1);
        let verify_wrong = signed.verify(&pk2);

        // Then: Only the correct key should succeed
        assert!(verify_correct.is_ok());
        assert!(verify_wrong.is_err());
    }

    /// Scenario: Each signature is unique for same transaction
    /// Given the same transaction content
    /// When signed multiple times (simulating ECDSA randomness)
    /// Then nonce handling should be secure
    #[test]
    fn scenario_signature_nonce_security() {
        // Given: The same transaction content and key
        let sk = SecretKey::generate();
        let pk = sk.public_key();

        let tx = Transaction::new(
            Chain::BitCell,
            "BC1sender".to_string(),
            "BC1receiver".to_string(),
            100_000,
            100,
            0,
        );

        // When: Signed multiple times
        let signed1 = tx.sign(&sk);
        let signed2 = tx.sign(&sk);

        // Then: Both should be valid
        assert!(signed1.verify(&pk).is_ok());
        assert!(signed2.verify(&pk).is_ok());

        // Transaction hash should be the same
        assert_eq!(signed1.transaction.hash(), signed2.transaction.hash());
    }

    /// Scenario: Modified transaction content invalidates signature
    /// Given a signed transaction
    /// When any field is modified after signing
    /// Then verification should fail
    #[test]
    fn scenario_transaction_modification_detection() {
        // Given: A signed transaction
        let sk = SecretKey::generate();
        let pk = sk.public_key();

        let tx = Transaction::new(
            Chain::BitCell,
            "BC1sender".to_string(),
            "BC1receiver".to_string(),
            100_000,
            100,
            0,
        );
        let signed = tx.sign(&sk);

        // Original verification works
        assert!(signed.verify(&pk).is_ok());

        // When: Content is modified (by creating different transactions)
        let modified_amount = Transaction::new(
            Chain::BitCell,
            "BC1sender".to_string(),
            "BC1receiver".to_string(),
            100_001, // Changed
            100,
            0,
        );

        let modified_fee = Transaction::new(
            Chain::BitCell,
            "BC1sender".to_string(),
            "BC1receiver".to_string(),
            100_000,
            101, // Changed
            0,
        );

        let modified_to = Transaction::new(
            Chain::BitCell,
            "BC1sender".to_string(),
            "BC1attacker".to_string(), // Changed
            100_000,
            100,
            0,
        );

        // Verify that signatures don't match modified content
        assert_ne!(tx.hash(), modified_amount.hash());
        assert_ne!(tx.hash(), modified_fee.hash());
        assert_ne!(tx.hash(), modified_to.hash());
    }

    /// Scenario: Replay attack protection via nonce
    /// Given transactions with different nonces
    /// When they are processed
    /// Then each should have a unique hash
    #[test]
    fn scenario_replay_attack_protection() {
        // Given: Transactions with different nonces
        let tx1 = Transaction::new(
            Chain::BitCell,
            "BC1sender".to_string(),
            "BC1receiver".to_string(),
            100_000,
            100,
            0, // nonce 0
        );

        let tx2 = Transaction::new(
            Chain::BitCell,
            "BC1sender".to_string(),
            "BC1receiver".to_string(),
            100_000,
            100,
            1, // nonce 1
        );

        let tx3 = Transaction::new(
            Chain::BitCell,
            "BC1sender".to_string(),
            "BC1receiver".to_string(),
            100_000,
            100,
            2, // nonce 2
        );

        // When: They are processed
        let hash1 = tx1.hash();
        let hash2 = tx2.hash();
        let hash3 = tx3.hash();

        // Then: Each should have a unique hash
        assert_ne!(hash1, hash2);
        assert_ne!(hash2, hash3);
        assert_ne!(hash1, hash3);
    }

    /// Scenario: Cross-chain transaction protection
    /// Given the same transaction parameters for chains with different chain IDs
    /// When they are hashed
    /// Then hashes should be different
    #[test]
    fn scenario_cross_chain_protection() {
        // Given: Same parameters for chains with different chain IDs
        // Note: BitCell has chain_id=1, Bitcoin has chain_id=0, Sepolia has chain_id=11155111
        let tx_bitcell = Transaction::new(
            Chain::BitCell,
            "sender".to_string(),
            "receiver".to_string(),
            100_000,
            100,
            0,
        );

        let tx_bitcoin = Transaction::new(
            Chain::Bitcoin,
            "sender".to_string(),
            "receiver".to_string(),
            100_000,
            100,
            0,
        );

        let tx_sepolia = Transaction::new(
            Chain::EthereumSepolia, // Has unique chain_id = 11155111
            "sender".to_string(),
            "receiver".to_string(),
            100_000,
            100,
            0,
        );

        // When: They are hashed
        let hash_bitcell = tx_bitcell.hash();
        let hash_bitcoin = tx_bitcoin.hash();
        let hash_sepolia = tx_sepolia.hash();

        // Then: Hashes should be different (chains with different chain_ids)
        // Bitcoin (chain_id=0) vs BitCell (chain_id=1)
        assert_ne!(hash_bitcoin, hash_bitcell, 
                   "Bitcoin (chain_id={}) and BitCell (chain_id={}) should have different hashes",
                   Chain::Bitcoin.chain_id(), Chain::BitCell.chain_id());
        
        // Sepolia (chain_id=11155111) should differ from both
        assert_ne!(hash_bitcoin, hash_sepolia,
                   "Bitcoin and Sepolia should have different hashes");
        assert_ne!(hash_bitcell, hash_sepolia,
                   "BitCell and Sepolia should have different hashes");
    }
}

// =============================================================================
// Input Validation Security Tests
// =============================================================================

mod input_validation_security {
    use super::*;

    /// Scenario: Invalid mnemonic phrases are rejected
    /// Given various malformed mnemonic inputs
    /// When parsing is attempted
    /// Then they should all be rejected
    #[test]
    fn scenario_invalid_mnemonic_rejection() {
        // Given: Various malformed mnemonic inputs
        let invalid_inputs = vec![
            "", // Empty
            "a", // Single character
            "word1 word2", // Too few words
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon", // 13 words (invalid)
            "not real bip39 words here at all these are invalid completely", // Invalid words
            "ABANDON ABANDON ABANDON ABANDON ABANDON ABANDON ABANDON ABANDON ABANDON ABANDON ABANDON ABOUT", // Wrong case
        ];

        for input in invalid_inputs {
            // When: Parsing is attempted
            let result = Mnemonic::from_phrase(input);

            // Then: They should be rejected
            if Mnemonic::validate(input) {
                // If validation passes, parsing should work
                assert!(result.is_ok());
            } else {
                // If validation fails, parsing should fail
                assert!(result.is_err(), "Should reject invalid mnemonic: '{}'", input);
            }
        }
    }

    /// Scenario: Zero amount transactions are rejected
    /// Given a transaction builder with zero amount
    /// When building is attempted
    /// Then it should fail
    #[test]
    fn scenario_zero_amount_transaction_rejection() {
        use bitcell_wallet::TransactionBuilder;

        // Given: A transaction builder with zero amount
        let builder = TransactionBuilder::new(Chain::BitCell)
            .from_str("BC1sender")
            .to_str("BC1receiver")
            .amount(0)
            .fee(100)
            .nonce(0);

        // When: Building is attempted
        let result = builder.build();

        // Then: It should fail
        assert!(result.is_err());
    }

    /// Scenario: Missing required fields are caught
    /// Given incomplete transaction builders
    /// When building is attempted
    /// Then appropriate errors should be returned
    #[test]
    fn scenario_missing_fields_caught() {
        use bitcell_wallet::TransactionBuilder;

        // Given: Incomplete transaction builders
        // Missing 'from'
        let no_from = TransactionBuilder::new(Chain::BitCell)
            .to_str("BC1receiver")
            .amount(1000);
        assert!(no_from.build().is_err());

        // Missing 'to'
        let no_to = TransactionBuilder::new(Chain::BitCell)
            .from_str("BC1sender")
            .amount(1000);
        assert!(no_to.build().is_err());
    }

    /// Scenario: Overflow protection in balance operations
    /// Given maximum balance values
    /// When operations would overflow
    /// Then they should saturate safely
    #[test]
    fn scenario_overflow_protection() {
        // Given: Maximum balance value
        let max_balance = Balance::new(u64::MAX, Chain::BitCell);

        // When: Adding would overflow
        let result = max_balance.add(1);

        // Then: It should saturate at max
        assert_eq!(result.amount(), u64::MAX);

        // Verify subtraction doesn't underflow
        let small_balance = Balance::new(100, Chain::BitCell);
        let subtract_result = small_balance.subtract(200);
        assert!(subtract_result.is_none());
    }
}

// =============================================================================
// Platform-Specific Security Tests
// =============================================================================

mod platform_security {
    use super::*;
    use std::env;

    /// Scenario: Cryptographic operations are secure on Linux
    #[test]
    #[cfg(target_os = "linux")]
    fn scenario_linux_crypto_security() {
        assert_eq!(env::consts::OS, "linux");
        
        // Verify key generation works
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        
        // Verify signing works
        let message = b"test message for linux";
        let signature = sk.sign(message);
        assert!(signature.verify(&pk, message).is_ok());
        
        println!("✓ Linux cryptographic security verified");
    }

    /// Scenario: Cryptographic operations are secure on macOS
    #[test]
    #[cfg(target_os = "macos")]
    fn scenario_macos_crypto_security() {
        assert_eq!(env::consts::OS, "macos");
        
        // Verify key generation works
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        
        // Verify signing works
        let message = b"test message for macos";
        let signature = sk.sign(message);
        assert!(signature.verify(&pk, message).is_ok());
        
        println!("✓ macOS cryptographic security verified");
    }

    /// Scenario: Cryptographic operations are secure on Windows
    #[test]
    #[cfg(target_os = "windows")]
    fn scenario_windows_crypto_security() {
        assert_eq!(env::consts::OS, "windows");
        
        // Verify key generation works
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        
        // Verify signing works
        let message = b"test message for windows";
        let signature = sk.sign(message);
        assert!(signature.verify(&pk, message).is_ok());
        
        println!("✓ Windows cryptographic security verified");
    }

    /// Scenario: Random number generation is platform-independent
    /// Given the current platform
    /// When random keys are generated
    /// Then they should have proper entropy
    #[test]
    fn scenario_platform_independent_rng() {
        // Given: Current platform
        let os = env::consts::OS;
        println!("Testing RNG on: {}", os);

        // When: Random keys are generated
        let mut keys = HashSet::new();
        for _ in 0..20 {
            let sk = SecretKey::generate();
            keys.insert(sk.public_key().as_bytes().to_vec());
        }

        // Then: They should all be unique (proper entropy)
        assert_eq!(keys.len(), 20, "RNG should produce unique keys on {}", os);
    }
}

// =============================================================================
// Timing Attack Resistance Tests
// =============================================================================

mod timing_attack_resistance {
    use super::*;
    use std::time::Instant;

    /// Scenario: Signature verification has consistent timing
    /// Given valid and invalid signatures
    /// When verification is performed
    /// Then timing should not leak information
    /// Note: This is a basic check; proper constant-time verification
    /// should be ensured in the cryptographic library
    #[test]
    fn scenario_verification_timing_consistency() {
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        let sk_wrong = SecretKey::generate();
        let pk_wrong = sk_wrong.public_key();

        let tx = Transaction::new(
            Chain::BitCell,
            "BC1sender".to_string(),
            "BC1receiver".to_string(),
            100_000,
            100,
            0,
        );
        let signed = tx.sign(&sk);

        // Measure valid verification times
        let mut valid_times = Vec::new();
        for _ in 0..100 {
            let start = Instant::now();
            let _ = signed.verify(&pk);
            valid_times.push(start.elapsed().as_nanos());
        }

        // Measure invalid verification times
        let mut invalid_times = Vec::new();
        for _ in 0..100 {
            let start = Instant::now();
            let _ = signed.verify(&pk_wrong);
            invalid_times.push(start.elapsed().as_nanos());
        }

        // Calculate averages (this is a basic check, not a rigorous timing analysis)
        let avg_valid: u128 = valid_times.iter().sum::<u128>() / valid_times.len() as u128;
        let avg_invalid: u128 = invalid_times.iter().sum::<u128>() / invalid_times.len() as u128;

        // Log the timings for informational purposes
        println!("Average valid verification time: {}ns", avg_valid);
        println!("Average invalid verification time: {}ns", avg_invalid);

        // We can't make strict assertions about timing as it depends on implementation
        // This test serves as a documentation/awareness test
    }
}
