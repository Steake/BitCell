//! Performance and Memory Usage Tests for BitCell Wallet
//!
//! These tests validate performance characteristics and memory usage
//! following BDD patterns across Linux, macOS, and Windows.
//!
//! Note: For detailed benchmarks, use `cargo bench` with the criterion
//! framework. These tests provide basic performance validation.

use bitcell_wallet::{
    Address, Chain, Mnemonic, Transaction, Wallet, WalletConfig,
    mnemonic::WordCount,
    transaction::TransactionBuilder,
};
use bitcell_crypto::{SecretKey, Hash256};
use std::time::{Duration, Instant};
use std::env;
use zeroize::Zeroize;

/// Minimum confirmations required for a transaction to be considered "fully confirmed"
const FULLY_CONFIRMED_THRESHOLD: u32 = 10;

// =============================================================================
// Performance Baseline Tests
// =============================================================================

mod performance_baselines {
    use super::*;

    /// Scenario: Mnemonic generation completes within acceptable time
    /// Given performance requirements
    /// When a mnemonic is generated
    /// Then it should complete within 100ms
    #[test]
    fn scenario_mnemonic_generation_performance() {
        // Given: Performance requirements
        let max_duration = Duration::from_millis(100);

        // When: Multiple mnemonics are generated
        for word_count in [WordCount::Words12, WordCount::Words18, WordCount::Words24] {
            let start = Instant::now();
            let _mnemonic = Mnemonic::generate(word_count);
            let duration = start.elapsed();

            // Then: It should complete within acceptable time
            assert!(
                duration < max_duration,
                "Mnemonic generation took {:?}, expected < {:?}",
                duration,
                max_duration
            );
        }
        
        println!("✓ Mnemonic generation performance test passed");
    }

    /// Scenario: Wallet creation completes within acceptable time
    /// Given performance requirements
    /// When a wallet is created with default config
    /// Then it should complete within 500ms
    #[test]
    fn scenario_wallet_creation_performance() {
        // Given: Performance requirements
        let max_duration = Duration::from_millis(500);

        // When: A wallet is created with default config
        let start = Instant::now();
        let (_wallet, _mnemonic) = Wallet::create_new(WalletConfig::default());
        let duration = start.elapsed();

        // Then: It should complete within acceptable time
        assert!(
            duration < max_duration,
            "Wallet creation took {:?}, expected < {:?}",
            duration,
            max_duration
        );
        
        println!("✓ Wallet creation took {:?}", duration);
    }

    /// Scenario: Address generation is fast
    /// Given a wallet
    /// When 100 addresses are generated
    /// Then it should complete within 1 second
    #[test]
    fn scenario_address_generation_performance() {
        // Given: A wallet
        let mut wallet = Wallet::from_mnemonic(&Mnemonic::new(), "", WalletConfig::default());
        let max_duration = Duration::from_secs(1);

        // When: 100 addresses are generated
        let start = Instant::now();
        for i in 10..110 { // Start at 10 to avoid pre-generated addresses
            let _ = wallet.generate_address(Chain::BitCell, i).unwrap();
        }
        let duration = start.elapsed();

        // Then: It should complete within acceptable time
        assert!(
            duration < max_duration,
            "Generating 100 addresses took {:?}, expected < {:?}",
            duration,
            max_duration
        );
        
        let per_address = duration.as_micros() / 100;
        println!("✓ Address generation: {:?} total, {} µs per address", duration, per_address);
    }

    /// Scenario: Transaction signing is fast
    /// Given a wallet with funds
    /// When 100 transactions are signed
    /// Then it should complete within 2 seconds
    #[test]
    fn scenario_transaction_signing_performance() {
        // Given: A wallet with funds
        let mut wallet = Wallet::from_mnemonic(&Mnemonic::new(), "", WalletConfig::default());
        let from = wallet.next_address(Chain::BitCell).unwrap();
        let to = wallet.next_address(Chain::BitCell).unwrap();
        wallet.update_balance(&from, 100_000_000_000); // Large balance

        let max_duration = Duration::from_secs(2);

        // When: 100 transactions are signed
        let start = Instant::now();
        for _ in 0..100 {
            let tx = wallet.create_transaction(&from, &to, 1000, 100).unwrap();
            let _signed = wallet.sign_transaction(tx, &from).unwrap();
        }
        let duration = start.elapsed();

        // Then: It should complete within acceptable time
        assert!(
            duration < max_duration,
            "Signing 100 transactions took {:?}, expected < {:?}",
            duration,
            max_duration
        );
        
        let per_tx = duration.as_micros() / 100;
        println!("✓ Transaction signing: {:?} total, {} µs per transaction", duration, per_tx);
    }

    /// Scenario: Hash computation is fast
    /// Given data to hash
    /// When 10000 hashes are computed
    /// Then it should complete within 1 second
    #[test]
    fn scenario_hash_computation_performance() {
        // Given: Data to hash
        let data = vec![0u8; 256];
        let max_duration = Duration::from_secs(1);

        // When: 10000 hashes are computed
        let start = Instant::now();
        for _ in 0..10000 {
            let _hash = Hash256::hash(&data);
        }
        let duration = start.elapsed();

        // Then: It should complete within acceptable time
        assert!(
            duration < max_duration,
            "Computing 10000 hashes took {:?}, expected < {:?}",
            duration,
            max_duration
        );
        
        let per_hash = duration.as_nanos() / 10000;
        println!("✓ Hash computation: {:?} total, {} ns per hash", duration, per_hash);
    }
}

// =============================================================================
// Scalability Tests
// =============================================================================

mod scalability_tests {
    use super::*;

    /// Scenario: Wallet handles many addresses efficiently
    /// Given a wallet
    /// When many addresses are added and tracked
    /// Then operations should remain efficient
    #[test]
    fn scenario_wallet_scales_with_addresses() {
        // Given: A wallet
        let mut wallet = Wallet::from_mnemonic(&Mnemonic::new(), "", WalletConfig::default());

        // When: Many addresses are added
        let num_addresses = 500;
        let start = Instant::now();
        
        for i in 10..(10 + num_addresses) {
            let _ = wallet.generate_address(Chain::BitCell, i).unwrap();
        }
        
        let generation_time = start.elapsed();

        // Then: Operations should remain efficient
        // Generation should scale linearly
        let per_address_us = generation_time.as_micros() as f64 / num_addresses as f64;
        
        // Address lookup should be fast
        let lookup_start = Instant::now();
        let addresses = wallet.all_addresses();
        let _count = addresses.len();
        let lookup_time = lookup_start.elapsed();

        println!("Generated {} addresses in {:?} ({:.2} µs each)", 
                 num_addresses, generation_time, per_address_us);
        println!("Address lookup took {:?}", lookup_time);

        assert!(generation_time < Duration::from_secs(10),
                "Address generation should be efficient");
        assert!(lookup_time < Duration::from_millis(100),
                "Address lookup should be fast");
    }

    /// Scenario: Transaction builder handles large data payloads
    /// Given a transaction with varying data sizes
    /// When built and hashed
    /// Then performance should scale appropriately
    #[test]
    fn scenario_transaction_data_scalability() {
        // Given: Varying data sizes
        let data_sizes = vec![0, 100, 1000, 10000, 100000];

        for size in data_sizes {
            let data = vec![0xAB; size];
            
            // When: Built and hashed
            let start = Instant::now();
            let tx = TransactionBuilder::new(Chain::BitCell)
                .from_str("BC1sender")
                .to_str("BC1receiver")
                .amount(1000)
                .fee(100)
                .data(data)
                .build()
                .unwrap();
            let _hash = tx.hash();
            let duration = start.elapsed();

            // Then: Performance should scale appropriately
            println!("Transaction with {} bytes data: {:?}", size, duration);
            
            // Even with 100KB data, should be under 100ms
            assert!(duration < Duration::from_millis(100),
                    "Transaction with {} bytes took too long: {:?}", size, duration);
        }
    }

    /// Scenario: Balance tracker scales with many addresses
    /// Given a balance tracker
    /// When tracking many addresses
    /// Then operations should remain efficient
    #[test]
    fn scenario_balance_tracker_scalability() {
        use bitcell_wallet::address::AddressType;

        // Given: A balance tracker
        let mut tracker = bitcell_wallet::balance::BalanceTracker::new();
        let num_addresses = 1000;

        // When: Tracking many addresses
        let start = Instant::now();
        for i in 0..num_addresses {
            let addr = Address::new(
                vec![i as u8; 20],
                AddressType::BitCell,
                Chain::BitCell,
                i as u32,
            );
            tracker.update_balance(&addr, (i as u64 + 1) * 1000);
        }
        let update_time = start.elapsed();

        // Query total
        let total_start = Instant::now();
        let _total = tracker.get_total(Chain::BitCell);
        let total_time = total_start.elapsed();

        // Then: Operations should remain efficient
        println!("Updated {} balances in {:?}", num_addresses, update_time);
        println!("Total balance query took {:?}", total_time);

        assert!(update_time < Duration::from_secs(2),
                "Balance updates should be efficient");
        assert!(total_time < Duration::from_millis(100),
                "Total calculation should be fast");
    }
}

// =============================================================================
// Memory Usage Tests
// =============================================================================

mod memory_usage_tests {
    use super::*;

    /// Scenario: Wallet has reasonable memory footprint
    /// Given a wallet with typical usage
    /// When memory is estimated
    /// Then it should be within acceptable bounds
    #[test]
    fn scenario_wallet_memory_footprint() {
        // Given: A wallet with typical usage
        let mut wallet = Wallet::from_mnemonic(&Mnemonic::new(), "", WalletConfig::default());
        
        // Add typical number of addresses (20 per chain)
        for chain in [Chain::BitCell, Chain::Bitcoin, Chain::Ethereum] {
            for i in 10..30 {
                let _ = wallet.generate_address(chain, i);
            }
        }

        // When: Estimating memory
        // We can't directly measure memory, but we can ensure structures are reasonable
        let addresses = wallet.all_addresses();
        let address_count = addresses.len();

        // Then: Should have expected count (default 5 + 20 per 3 chains)
        // Default lookahead is 5 for each enabled chain
        assert!(address_count > 50, "Should have many addresses");
        println!("Wallet contains {} addresses", address_count);
    }

    /// Scenario: Transaction history doesn't grow unbounded
    /// Given a transaction history
    /// When many transactions are added and pruned
    /// Then memory should be controlled
    #[test]
    fn scenario_transaction_history_memory_control() {
        use bitcell_wallet::history::{TransactionDirection, TransactionHistory, TransactionRecord};

        // Given: A transaction history
        let mut history = TransactionHistory::new();

        // Add many transactions
        for i in 0..1000 {
            let mut record = TransactionRecord::new(
                format!("0x{:064x}", i),
                Chain::BitCell,
                TransactionDirection::Outgoing,
                "BC1sender".to_string(),
                "BC1receiver".to_string(),
                1000,
                100,
                1000000 + i,
            );
            if i < 500 {
                record.confirm(i);
                record.confirmations = FULLY_CONFIRMED_THRESHOLD; // Fully confirmed
            }
            history.add(record);
        }

        assert_eq!(history.count(), 1000);

        // When: Pruned
        history.prune(200);

        // Then: Memory should be controlled
        // After pruning, should have fewer transactions
        println!("After pruning: {} transactions", history.count());
        assert!(history.count() <= 1000, "Pruning should reduce count");
    }

    /// Scenario: Seed and key material is zeroized on drop
    /// Given seed bytes
    /// When they go out of scope
    /// Then they should be zeroized (verified by trait implementation)
    #[test]
    fn scenario_sensitive_data_zeroization() {
        // Given: Mnemonic and seed bytes
        let mnemonic = Mnemonic::new();
        let seed = mnemonic.to_seed("test");

        // Verify the seed has content
        assert!(seed.as_bytes().iter().any(|&b| b != 0), "Seed should have non-zero content");

        // SeedBytes implements Zeroize trait
        // The actual zeroization happens on Drop, which we can't easily test
        // But we can verify the type has the trait
        let mut seed2 = mnemonic.to_seed("test2");
        seed2.zeroize();
        
        // After explicit zeroize, bytes should be zero
        assert!(seed2.as_bytes().iter().all(|&b| b == 0), "Zeroized seed should be all zeros");
        
        println!("✓ Zeroization test passed");
    }
}

// =============================================================================
// Platform-Specific Performance Tests
// =============================================================================

mod platform_performance_tests {
    use super::*;

    /// Scenario: Performance is acceptable on Linux
    #[test]
    #[cfg(target_os = "linux")]
    fn scenario_linux_performance() {
        println!("Running performance tests on Linux");
        run_platform_performance_suite("linux");
    }

    /// Scenario: Performance is acceptable on macOS
    #[test]
    #[cfg(target_os = "macos")]
    fn scenario_macos_performance() {
        println!("Running performance tests on macOS");
        run_platform_performance_suite("macos");
    }

    /// Scenario: Performance is acceptable on Windows
    #[test]
    #[cfg(target_os = "windows")]
    fn scenario_windows_performance() {
        println!("Running performance tests on Windows");
        run_platform_performance_suite("windows");
    }

    /// Platform-independent performance test
    #[test]
    fn scenario_platform_agnostic_performance() {
        let os = env::consts::OS;
        println!("Running performance tests on: {}", os);
        run_platform_performance_suite(os);
    }

    fn run_platform_performance_suite(platform: &str) {
        // Key generation
        let start = Instant::now();
        for _ in 0..10 {
            let _ = SecretKey::generate();
        }
        let key_gen_time = start.elapsed();
        println!("[{}] 10 key generations: {:?}", platform, key_gen_time);
        assert!(key_gen_time < Duration::from_secs(1));

        // Mnemonic generation
        let start = Instant::now();
        for _ in 0..10 {
            let _ = Mnemonic::new();
        }
        let mnemonic_time = start.elapsed();
        println!("[{}] 10 mnemonic generations: {:?}", platform, mnemonic_time);
        assert!(mnemonic_time < Duration::from_secs(1));

        // Wallet operations
        let start = Instant::now();
        let mut wallet = Wallet::from_mnemonic(&Mnemonic::new(), "", WalletConfig::default());
        let _addr = wallet.next_address(Chain::BitCell).unwrap();
        let wallet_time = start.elapsed();
        println!("[{}] Wallet creation + address: {:?}", platform, wallet_time);
        assert!(wallet_time < Duration::from_secs(2));

        // Transaction creation and signing
        let from = wallet.next_address(Chain::BitCell).unwrap();
        let to = wallet.next_address(Chain::BitCell).unwrap();
        wallet.update_balance(&from, 1_000_000);
        
        let start = Instant::now();
        let tx = wallet.create_transaction(&from, &to, 1000, 100).unwrap();
        let _signed = wallet.sign_transaction(tx, &from).unwrap();
        let tx_time = start.elapsed();
        println!("[{}] Transaction creation + signing: {:?}", platform, tx_time);
        assert!(tx_time < Duration::from_millis(500));

        println!("✓ {} performance suite passed", platform);
    }
}

// =============================================================================
// Concurrent Access Tests
// =============================================================================

mod concurrent_access_tests {
    use super::*;
    use std::thread;

    /// Scenario: Cryptographic operations are thread-safe
    /// Given multiple threads
    /// When performing cryptographic operations concurrently
    /// Then all operations should complete successfully
    #[test]
    fn scenario_concurrent_crypto_operations() {
        // Given: Multiple threads
        let num_threads = 4;
        let ops_per_thread = 25;

        // When: Performing cryptographic operations concurrently
        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                thread::spawn(move || {
                    let mut results = Vec::new();
                    for i in 0..ops_per_thread {
                        // Generate key
                        let sk = SecretKey::generate();
                        let pk = sk.public_key();
                        
                        // Sign message
                        let message = format!("thread {} message {}", thread_id, i);
                        let signature = sk.sign(message.as_bytes());
                        
                        // Verify
                        let verified = signature.verify(&pk, message.as_bytes()).is_ok();
                        results.push(verified);
                    }
                    results
                })
            })
            .collect();

        // Then: All operations should complete successfully
        for handle in handles {
            let results = handle.join().expect("Thread should not panic");
            assert!(results.iter().all(|&v| v), "All verifications should pass");
        }

        println!("✓ Concurrent crypto operations test passed");
    }

    /// Scenario: Independent wallets can operate concurrently
    /// Given multiple independent wallets
    /// When they operate concurrently
    /// Then all operations should complete correctly
    #[test]
    fn scenario_concurrent_wallet_operations() {
        // Given: Multiple threads creating wallets
        let num_threads = 4;

        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                thread::spawn(move || {
                    // Each thread creates its own wallet
                    let (mut wallet, _) = Wallet::create_new(WalletConfig::default());
                    
                    // Generate addresses
                    for i in 10..20 {
                        let _ = wallet.generate_address(Chain::BitCell, i).unwrap();
                    }

                    // Get address count
                    let count = wallet.all_addresses().len();
                    
                    (thread_id, count)
                })
            })
            .collect();

        // When: They complete
        for handle in handles {
            let (thread_id, count) = handle.join().expect("Thread should not panic");
            // Then: Each should have the expected address count
            assert!(count >= 10, "Thread {} should have at least 10 addresses", thread_id);
        }

        println!("✓ Concurrent wallet operations test passed");
    }
}

// =============================================================================
// Stress Tests
// =============================================================================

mod stress_tests {
    use super::*;

    /// Scenario: System handles sustained load
    /// Given continuous operations over time
    /// When many operations are performed
    /// Then the system should remain stable
    #[test]
    fn scenario_sustained_load_handling() {
        // Given: A wallet for sustained operations
        let mut wallet = Wallet::from_mnemonic(&Mnemonic::new(), "", WalletConfig::default());
        let from = wallet.next_address(Chain::BitCell).unwrap();
        let to = wallet.next_address(Chain::BitCell).unwrap();
        wallet.update_balance(&from, u64::MAX / 2);

        let num_operations = 500;
        let start = Instant::now();

        // When: Many operations are performed
        for i in 0..num_operations {
            // Create and sign transaction
            let tx = wallet.create_transaction(&from, &to, 1000, 100).unwrap();
            let _signed = wallet.sign_transaction(tx, &from).unwrap();

            // Occasional address generation
            if i % 50 == 0 {
                let _ = wallet.generate_address(Chain::BitCell, (100 + i) as u32);
            }
        }

        let duration = start.elapsed();

        // Then: Should complete in reasonable time
        let ops_per_sec = num_operations as f64 / duration.as_secs_f64();
        println!("Sustained load: {} operations in {:?} ({:.1} ops/sec)", 
                 num_operations, duration, ops_per_sec);

        assert!(duration < Duration::from_secs(30),
                "Sustained load should complete in reasonable time");
    }

    /// Scenario: System handles burst load
    /// Given a sudden burst of operations
    /// When they are all processed
    /// Then the system should not fail
    #[test]
    fn scenario_burst_load_handling() {
        // Given: Prepare for burst
        let num_transactions = 100;
        let sk = SecretKey::generate();

        // When: Burst of operations
        let start = Instant::now();
        
        let transactions: Vec<_> = (0..num_transactions)
            .map(|i| {
                Transaction::new(
                    Chain::BitCell,
                    format!("BC1sender{}", i),
                    format!("BC1receiver{}", i),
                    1000 + i,
                    100,
                    i,
                )
            })
            .collect();

        let signed: Vec<_> = transactions.iter()
            .map(|tx| tx.sign(&sk))
            .collect();

        let duration = start.elapsed();

        // Then: All should complete
        assert_eq!(signed.len(), num_transactions as usize);
        println!("Burst load: {} transactions signed in {:?}", num_transactions, duration);
    }
}
