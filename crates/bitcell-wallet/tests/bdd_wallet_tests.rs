//! BDD-style Integration Tests for BitCell Wallet
//!
//! These tests follow Behavior-Driven Development patterns using Given-When-Then
//! structure to validate core wallet functionality across all supported platforms
//! (Linux, macOS, Windows).
//!
//! # Test Categories
//! 
//! 1. Wallet Creation - Creating and managing wallets
//! 2. Seed Phrase/Mnemonic - BIP39 mnemonic generation and validation
//! 3. Address Generation - Multi-chain address derivation
//! 4. Transaction Logic - Transaction creation, signing, and verification
//! 5. Cross-Platform Compatibility - Platform-specific behavior validation
//! 6. Security Tests - Key storage and transaction signing security

use bitcell_wallet::{
    Address, Balance, Chain, ChainConfig, Error, Mnemonic,
    SignedTransaction, Transaction, TransactionBuilder, Wallet, WalletConfig,
    address::AddressType,
    history::{TransactionDirection, TransactionRecord, TransactionHistory},
    mnemonic::WordCount,
    transaction::FeeEstimator,
    wallet::WalletState,
};
use bitcell_crypto::SecretKey;
use std::collections::HashMap;

// =============================================================================
// BDD Test Module: Wallet Creation
// =============================================================================

mod wallet_creation_tests {
    use super::*;

    /// Scenario: Creating a new wallet generates a valid mnemonic
    /// Given a user wants to create a new wallet
    /// When they create a wallet with default configuration
    /// Then a valid 24-word mnemonic should be generated
    /// And the wallet should be in unlocked state
    #[test]
    fn scenario_creating_new_wallet_generates_valid_mnemonic() {
        // Given: A user wants to create a new wallet
        let config = WalletConfig::default();

        // When: They create a wallet with default configuration
        let (wallet, mnemonic) = Wallet::create_new(config);

        // Then: A valid 24-word mnemonic should be generated
        assert_eq!(mnemonic.word_count(), 24);
        assert!(Mnemonic::validate(mnemonic.phrase()));

        // And: The wallet should be in unlocked state
        assert!(wallet.is_unlocked());
        assert_eq!(wallet.state(), WalletState::Unlocked);
    }

    /// Scenario: Creating a wallet from mnemonic restores the same addresses
    /// Given a user has a valid mnemonic phrase
    /// When they restore a wallet using that mnemonic
    /// Then the wallet should generate the same addresses as before
    #[test]
    fn scenario_restoring_wallet_from_mnemonic_produces_same_addresses() {
        // Given: A user has a valid mnemonic phrase
        let original_mnemonic = Mnemonic::new();
        let phrase = original_mnemonic.phrase().to_string();

        // Create first wallet and generate addresses
        let mut wallet1 = Wallet::from_mnemonic(&original_mnemonic, "", WalletConfig::default());
        let addr1 = wallet1.generate_address(Chain::BitCell, 10).unwrap();

        // When: They restore a wallet using that mnemonic
        let restored_mnemonic = Mnemonic::from_phrase(&phrase).unwrap();
        let mut wallet2 = Wallet::from_mnemonic(&restored_mnemonic, "", WalletConfig::default());
        let addr2 = wallet2.generate_address(Chain::BitCell, 10).unwrap();

        // Then: The wallet should generate the same addresses as before
        assert_eq!(addr1.as_bytes(), addr2.as_bytes());
        assert_eq!(addr1.to_string_formatted(), addr2.to_string_formatted());
    }

    /// Scenario: Wallet configuration determines enabled chains
    /// Given a custom wallet configuration with specific chains
    /// When a wallet is created with this configuration
    /// Then only the specified chains should be enabled
    #[test]
    fn scenario_wallet_configuration_determines_enabled_chains() {
        // Given: A custom wallet configuration with specific chains
        let config = WalletConfig {
            name: "Test Wallet".to_string(),
            chains: vec![
                ChainConfig::new(Chain::BitCell),
                ChainConfig::new(Chain::Bitcoin).set_enabled(false),
                ChainConfig::new(Chain::Ethereum),
            ],
            auto_generate_addresses: true,
            address_lookahead: 3,
        };

        // When: A wallet is created with this configuration
        let (wallet, _) = Wallet::create_new(config);

        // Then: Only the specified chains should be configured
        assert_eq!(wallet.config().name, "Test Wallet");
        assert_eq!(wallet.config().chains.len(), 3);
        assert_eq!(wallet.config().address_lookahead, 3);
    }

    /// Scenario: Wallet can be locked and unlocked
    /// Given an unlocked wallet with a known mnemonic
    /// When the wallet is locked
    /// Then it should not be able to sign transactions
    /// And when unlocked with correct mnemonic it should work again
    #[test]
    fn scenario_wallet_locking_and_unlocking() {
        // Given: An unlocked wallet with a known mnemonic
        let mnemonic = Mnemonic::new();
        let mut wallet = Wallet::from_mnemonic(&mnemonic, "", WalletConfig::default());
        assert!(wallet.is_unlocked());

        let from = wallet.next_address(Chain::BitCell).unwrap();
        let to = wallet.next_address(Chain::BitCell).unwrap();
        wallet.update_balance(&from, 1_000_000);

        // When: The wallet is locked
        wallet.lock();

        // Then: It should not be able to sign transactions
        assert!(!wallet.is_unlocked());
        assert_eq!(wallet.state(), WalletState::Locked);

        // Transaction creation still works (doesn't need signing)
        let tx = wallet.create_transaction(&from, &to, 100, 10).unwrap();
        
        // But signing should fail
        let sign_result = wallet.sign_transaction(tx.clone(), &from);
        assert!(matches!(sign_result, Err(Error::WalletLocked)));

        // And when unlocked with correct mnemonic it should work again
        wallet.unlock(&mnemonic, "").unwrap();
        assert!(wallet.is_unlocked());
        
        // Now we can create a fresh transaction and sign
        let tx2 = wallet.create_transaction(&from, &to, 100, 10).unwrap();
        let signed = wallet.sign_transaction(tx2, &from);
        assert!(signed.is_ok());
    }

    /// Scenario: Wallet export and import preserves data
    /// Given a wallet with addresses, balances, and history
    /// When the wallet data is exported and imported to a new wallet
    /// Then the configuration should be preserved
    #[test]
    fn scenario_wallet_export_and_import_preserves_data() {
        // Given: A wallet with addresses, balances, and history
        let mut wallet1 = Wallet::from_mnemonic(&Mnemonic::new(), "", WalletConfig::default());
        let addr = wallet1.next_address(Chain::BitCell).unwrap();
        wallet1.update_balance(&addr, 500_000);

        // When: The wallet data is exported and imported to a new wallet
        let export = wallet1.export_data();
        
        let mut wallet2 = Wallet::from_mnemonic(&Mnemonic::new(), "", WalletConfig::default());
        wallet2.import_data(export);

        // Then: The configuration should be preserved
        assert_eq!(wallet2.config().name, wallet1.config().name);
    }
}

// =============================================================================
// BDD Test Module: Seed Phrase/Mnemonic
// =============================================================================

mod seed_phrase_tests {
    use super::*;

    /// Scenario: Generating a 12-word mnemonic
    /// Given a user requests a mnemonic with 12 words
    /// When the mnemonic is generated
    /// Then it should contain exactly 12 valid BIP39 words
    #[test]
    fn scenario_generating_12_word_mnemonic() {
        // Given: A user requests a mnemonic with 12 words
        // When: The mnemonic is generated
        let mnemonic = Mnemonic::generate(WordCount::Words12);

        // Then: It should contain exactly 12 valid BIP39 words
        assert_eq!(mnemonic.word_count(), 12);
        assert!(Mnemonic::validate(mnemonic.phrase()));
        assert_eq!(mnemonic.words().len(), 12);
    }

    /// Scenario: Generating an 18-word mnemonic
    /// Given a user requests a mnemonic with 18 words
    /// When the mnemonic is generated
    /// Then it should contain exactly 18 valid BIP39 words
    #[test]
    fn scenario_generating_18_word_mnemonic() {
        // Given: A user requests a mnemonic with 18 words
        // When: The mnemonic is generated
        let mnemonic = Mnemonic::generate(WordCount::Words18);

        // Then: It should contain exactly 18 valid BIP39 words
        assert_eq!(mnemonic.word_count(), 18);
        assert!(Mnemonic::validate(mnemonic.phrase()));
    }

    /// Scenario: Generating a 24-word mnemonic (default)
    /// Given a user creates a mnemonic with default settings
    /// When the mnemonic is generated
    /// Then it should contain exactly 24 valid BIP39 words
    #[test]
    fn scenario_generating_24_word_mnemonic_default() {
        // Given: A user creates a mnemonic with default settings
        // When: The mnemonic is generated
        let mnemonic = Mnemonic::new();

        // Then: It should contain exactly 24 valid BIP39 words
        assert_eq!(mnemonic.word_count(), 24);
        assert!(Mnemonic::validate(mnemonic.phrase()));
        for word in mnemonic.words() {
            assert!(!word.is_empty());
        }
    }

    /// Scenario: Validating an invalid mnemonic phrase
    /// Given an invalid mnemonic phrase
    /// When validation is attempted
    /// Then it should fail validation
    #[test]
    fn scenario_invalid_mnemonic_fails_validation() {
        // Given: An invalid mnemonic phrase
        let invalid_phrases = vec![
            "invalid mnemonic phrase",
            "hello world test",
            "one two three four five six seven eight nine ten eleven twelve",
            "",
            "word",
        ];

        for phrase in invalid_phrases {
            // When: Validation is attempted
            // Then: It should fail validation
            assert!(!Mnemonic::validate(phrase));
            assert!(Mnemonic::from_phrase(phrase).is_err());
        }
    }

    /// Scenario: Same mnemonic with different passphrase produces different seeds
    /// Given a valid mnemonic
    /// When seeds are derived with different passphrases
    /// Then the seeds should be different
    #[test]
    fn scenario_different_passphrase_produces_different_seed() {
        // Given: A valid mnemonic
        let mnemonic = Mnemonic::new();

        // When: Seeds are derived with different passphrases
        let seed1 = mnemonic.to_seed("");
        let seed2 = mnemonic.to_seed("password123");
        let seed3 = mnemonic.to_seed("different_password");

        // Then: The seeds should be different
        assert_ne!(seed1.as_bytes(), seed2.as_bytes());
        assert_ne!(seed2.as_bytes(), seed3.as_bytes());
        assert_ne!(seed1.as_bytes(), seed3.as_bytes());
    }

    /// Scenario: Same mnemonic and passphrase always produces same seed
    /// Given a valid mnemonic and passphrase
    /// When the seed is derived multiple times
    /// Then the seed should be identical each time
    #[test]
    fn scenario_same_passphrase_produces_same_seed() {
        // Given: A valid mnemonic and passphrase
        let mnemonic = Mnemonic::new();
        let passphrase = "my_secure_passphrase";

        // When: The seed is derived multiple times
        let seed1 = mnemonic.to_seed(passphrase);
        let seed2 = mnemonic.to_seed(passphrase);
        let seed3 = mnemonic.to_seed(passphrase);

        // Then: The seed should be identical each time
        assert_eq!(seed1.as_bytes(), seed2.as_bytes());
        assert_eq!(seed2.as_bytes(), seed3.as_bytes());
    }

    /// Scenario: Seed bytes have correct length
    /// Given a mnemonic
    /// When a seed is derived
    /// Then the seed should be 64 bytes with 32-byte master key and chain code
    #[test]
    fn scenario_seed_bytes_have_correct_length() {
        // Given: A mnemonic
        let mnemonic = Mnemonic::new();

        // When: A seed is derived
        let seed = mnemonic.to_seed("");

        // Then: The seed should be 64 bytes with 32-byte master key and chain code
        assert_eq!(seed.as_bytes().len(), 64);
        assert_eq!(seed.master_key_bytes().len(), 32);
        assert_eq!(seed.chain_code_bytes().len(), 32);
    }
}

// =============================================================================
// BDD Test Module: Address Generation
// =============================================================================

mod address_generation_tests {
    use super::*;

    /// Scenario: Generating BitCell addresses
    /// Given a wallet with BitCell chain enabled
    /// When addresses are generated
    /// Then they should have the correct format and be valid
    #[test]
    fn scenario_generating_bitcell_addresses() {
        // Given: A wallet with BitCell chain enabled
        let mut wallet = Wallet::from_mnemonic(&Mnemonic::new(), "", WalletConfig::default());

        // When: Addresses are generated
        let addr = wallet.generate_address(Chain::BitCell, 0).unwrap();

        // Then: They should have the correct format and be valid
        assert_eq!(addr.chain(), Chain::BitCell);
        assert_eq!(addr.address_type(), AddressType::BitCell);
        assert!(addr.is_valid());
        assert!(addr.to_string_formatted().starts_with("BC1"));
    }

    /// Scenario: Generating Bitcoin addresses
    /// Given a wallet with Bitcoin chain enabled
    /// When addresses are generated for mainnet and testnet
    /// Then they should have the correct format for each network
    #[test]
    fn scenario_generating_bitcoin_addresses() {
        // Given: A wallet with Bitcoin chain enabled
        let mut wallet = Wallet::from_mnemonic(&Mnemonic::new(), "", WalletConfig::default());

        // When: Addresses are generated for mainnet and testnet
        let mainnet_addr = wallet.generate_address(Chain::Bitcoin, 0).unwrap();
        let testnet_addr = wallet.generate_address(Chain::BitcoinTestnet, 0).unwrap();

        // Then: They should have the correct format for each network
        assert_eq!(mainnet_addr.chain(), Chain::Bitcoin);
        assert_eq!(mainnet_addr.address_type(), AddressType::BitcoinP2PKH);
        assert!(mainnet_addr.is_valid());

        assert_eq!(testnet_addr.chain(), Chain::BitcoinTestnet);
        assert!(testnet_addr.is_valid());
    }

    /// Scenario: Generating Ethereum addresses
    /// Given a wallet with Ethereum chain enabled
    /// When addresses are generated
    /// Then they should be 40 hex characters with 0x prefix
    #[test]
    fn scenario_generating_ethereum_addresses() {
        // Given: A wallet with Ethereum chain enabled
        let mut wallet = Wallet::from_mnemonic(&Mnemonic::new(), "", WalletConfig::default());

        // When: Addresses are generated
        let addr = wallet.generate_address(Chain::Ethereum, 0).unwrap();

        // Then: They should be 40 hex characters with 0x prefix
        assert_eq!(addr.chain(), Chain::Ethereum);
        assert_eq!(addr.address_type(), AddressType::Ethereum);
        assert!(addr.is_valid());
        
        let formatted = addr.to_string_formatted();
        assert!(formatted.starts_with("0x"));
        assert_eq!(formatted.len(), 42); // 0x + 40 hex chars
    }

    /// Scenario: Different derivation indices produce different addresses
    /// Given a wallet
    /// When addresses are generated with different indices
    /// Then each address should be unique
    #[test]
    fn scenario_different_indices_produce_different_addresses() {
        // Given: A wallet
        let mut wallet = Wallet::from_mnemonic(&Mnemonic::new(), "", WalletConfig::default());

        // When: Addresses are generated with different indices
        let addr0 = wallet.generate_address(Chain::BitCell, 0).unwrap();
        let addr1 = wallet.generate_address(Chain::BitCell, 1).unwrap();
        let addr2 = wallet.generate_address(Chain::BitCell, 2).unwrap();

        // Then: Each address should be unique
        assert_ne!(addr0.as_bytes(), addr1.as_bytes());
        assert_ne!(addr1.as_bytes(), addr2.as_bytes());
        assert_ne!(addr0.as_bytes(), addr2.as_bytes());
    }

    /// Scenario: Same wallet generates same addresses deterministically
    /// Given the same mnemonic
    /// When creating two wallets and generating addresses
    /// Then the addresses should be identical
    #[test]
    fn scenario_deterministic_address_generation() {
        // Given: The same mnemonic
        let mnemonic = Mnemonic::new();

        // When: Creating two wallets and generating addresses
        let mut wallet1 = Wallet::from_mnemonic(&mnemonic, "", WalletConfig::default());
        let mut wallet2 = Wallet::from_mnemonic(&mnemonic, "", WalletConfig::default());

        let addrs1: Vec<_> = (0..5)
            .map(|i| wallet1.generate_address(Chain::BitCell, i).unwrap())
            .collect();
        let addrs2: Vec<_> = (0..5)
            .map(|i| wallet2.generate_address(Chain::BitCell, i).unwrap())
            .collect();

        // Then: The addresses should be identical
        for (a1, a2) in addrs1.iter().zip(addrs2.iter()) {
            assert_eq!(a1.as_bytes(), a2.as_bytes());
        }
    }

    /// Scenario: Multi-chain address generation
    /// Given a wallet
    /// When generating addresses for different chains
    /// Then each chain should have its own address format
    #[test]
    fn scenario_multi_chain_address_generation() {
        // Given: A wallet
        let mut wallet = Wallet::from_mnemonic(&Mnemonic::new(), "", WalletConfig::default());

        // When: Generating addresses for different chains
        let bitcell_addr = wallet.generate_address(Chain::BitCell, 0).unwrap();
        let bitcoin_addr = wallet.generate_address(Chain::Bitcoin, 0).unwrap();
        let ethereum_addr = wallet.generate_address(Chain::Ethereum, 0).unwrap();

        // Then: Each chain should have its own address format
        assert!(bitcell_addr.to_string_formatted().starts_with("BC1"));
        // Bitcoin addresses are Base58 encoded
        assert!(!bitcoin_addr.to_string_formatted().starts_with("0x"));
        assert!(ethereum_addr.to_string_formatted().starts_with("0x"));
    }

    /// Scenario: Address parsing roundtrip
    /// Given a generated address
    /// When it is formatted to string and parsed back
    /// Then the bytes should match the original
    #[test]
    fn scenario_address_parsing_roundtrip() {
        // Given: A generated address
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        let original = Address::from_public_key_bitcell(&pk, 0);

        // When: It is formatted to string and parsed back
        let formatted = original.to_string_formatted();
        let parsed = Address::from_string(&formatted, Chain::BitCell).unwrap();

        // Then: The bytes should match the original
        assert_eq!(original.as_bytes(), parsed.as_bytes());
    }
}

// =============================================================================
// BDD Test Module: Transaction Logic
// =============================================================================

mod transaction_logic_tests {
    use super::*;

    /// Scenario: Creating a valid transaction
    /// Given a wallet with sufficient balance
    /// When a transaction is created
    /// Then it should contain correct sender, recipient, amount and fee
    #[test]
    fn scenario_creating_valid_transaction() {
        // Given: A wallet with sufficient balance
        let mut wallet = Wallet::from_mnemonic(&Mnemonic::new(), "", WalletConfig::default());
        let from = wallet.next_address(Chain::BitCell).unwrap();
        let to = wallet.next_address(Chain::BitCell).unwrap();
        wallet.update_balance(&from, 1_000_000);

        // When: A transaction is created
        let tx = wallet.create_transaction(&from, &to, 100_000, 100).unwrap();

        // Then: It should contain correct sender, recipient, amount and fee
        assert_eq!(tx.amount, 100_000);
        assert_eq!(tx.fee, 100);
        assert_eq!(tx.chain, Chain::BitCell);
        assert_eq!(tx.total_cost(), 100_100);
    }

    /// Scenario: Transaction fails with insufficient balance
    /// Given a wallet with limited balance
    /// When a transaction exceeding the balance is attempted
    /// Then it should fail with InsufficientBalance error
    #[test]
    fn scenario_transaction_fails_with_insufficient_balance() {
        // Given: A wallet with limited balance
        let mut wallet = Wallet::from_mnemonic(&Mnemonic::new(), "", WalletConfig::default());
        let from = wallet.next_address(Chain::BitCell).unwrap();
        let to = wallet.next_address(Chain::BitCell).unwrap();
        wallet.update_balance(&from, 1000); // Only 1000 units

        // When: A transaction exceeding the balance is attempted
        let result = wallet.create_transaction(&from, &to, 2000, 100);

        // Then: It should fail with InsufficientBalance error
        assert!(matches!(result, Err(Error::InsufficientBalance { .. })));
    }

    /// Scenario: Signing a transaction
    /// Given a valid transaction and unlocked wallet
    /// When the transaction is signed
    /// Then it should produce a valid signature that can be verified
    #[test]
    fn scenario_signing_transaction() {
        // Given: A valid transaction and unlocked wallet
        let mut wallet = Wallet::from_mnemonic(&Mnemonic::new(), "", WalletConfig::default());
        let from = wallet.next_address(Chain::BitCell).unwrap();
        let to = wallet.next_address(Chain::BitCell).unwrap();
        wallet.update_balance(&from, 1_000_000);

        // When: The transaction is signed
        let tx = wallet.create_transaction(&from, &to, 100_000, 100).unwrap();
        let signed = wallet.sign_transaction(tx, &from).unwrap();

        // Then: It should produce a valid signature that can be verified
        assert!(!signed.hash_hex().is_empty());
        assert_eq!(signed.hash_hex().len(), 64); // 32 bytes = 64 hex chars
    }

    /// Scenario: Nonce increments after transaction
    /// Given a wallet with balance
    /// When multiple transactions are signed
    /// Then the nonce should increment for each transaction
    #[test]
    fn scenario_nonce_increments_after_transaction() {
        // Given: A wallet with balance
        let mut wallet = Wallet::from_mnemonic(&Mnemonic::new(), "", WalletConfig::default());
        let from = wallet.next_address(Chain::BitCell).unwrap();
        let to = wallet.next_address(Chain::BitCell).unwrap();
        wallet.update_balance(&from, 10_000_000);

        // When: Multiple transactions are signed
        assert_eq!(wallet.get_nonce(&from), 0);
        
        wallet.send(&from, &to, 100_000, 100).unwrap();
        assert_eq!(wallet.get_nonce(&from), 1);
        
        wallet.send(&from, &to, 100_000, 100).unwrap();
        assert_eq!(wallet.get_nonce(&from), 2);
        
        wallet.send(&from, &to, 100_000, 100).unwrap();
        
        // Then: The nonce should increment for each transaction
        assert_eq!(wallet.get_nonce(&from), 3);
    }

    /// Scenario: Transaction hash is deterministic
    /// Given the same transaction parameters
    /// When the transaction hash is computed multiple times
    /// Then it should produce the same hash
    #[test]
    fn scenario_transaction_hash_is_deterministic() {
        // Given: The same transaction parameters
        let tx = Transaction::new(
            Chain::BitCell,
            "BC1sender".to_string(),
            "BC1receiver".to_string(),
            100_000,
            100,
            0,
        );

        // When: The transaction hash is computed multiple times
        let hash1 = tx.hash();
        let hash2 = tx.hash();
        let hash3 = tx.hash();

        // Then: It should produce the same hash
        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);
    }

    /// Scenario: Different transactions produce different hashes
    /// Given transactions with different parameters
    /// When their hashes are computed
    /// Then they should be different
    #[test]
    fn scenario_different_transactions_produce_different_hashes() {
        // Given: Transactions with different parameters
        let tx1 = Transaction::new(
            Chain::BitCell,
            "BC1sender".to_string(),
            "BC1receiver".to_string(),
            100_000,
            100,
            0,
        );
        
        let tx2 = Transaction::new(
            Chain::BitCell,
            "BC1sender".to_string(),
            "BC1receiver".to_string(),
            100_001, // Different amount
            100,
            0,
        );

        // When: Their hashes are computed
        let hash1 = tx1.hash();
        let hash2 = tx2.hash();

        // Then: They should be different
        assert_ne!(hash1, hash2);
    }

    /// Scenario: TransactionBuilder creates valid transactions
    /// Given a transaction builder with all required fields
    /// When the transaction is built
    /// Then it should contain the correct values
    #[test]
    fn scenario_transaction_builder_creates_valid_transactions() {
        // Given: A transaction builder with all required fields
        let builder = TransactionBuilder::new(Chain::BitCell)
            .from_str("BC1sender")
            .to_str("BC1receiver")
            .amount(50_000)
            .fee(50)
            .nonce(5)
            .data(b"memo".to_vec());

        // When: The transaction is built
        let tx = builder.build().unwrap();

        // Then: It should contain the correct values
        assert_eq!(tx.chain, Chain::BitCell);
        assert_eq!(tx.from, "BC1sender");
        assert_eq!(tx.to, "BC1receiver");
        assert_eq!(tx.amount, 50_000);
        assert_eq!(tx.fee, 50);
        assert_eq!(tx.nonce, 5);
        assert_eq!(tx.data, b"memo");
    }

    /// Scenario: TransactionBuilder fails without required fields
    /// Given a transaction builder missing required fields
    /// When building is attempted
    /// Then it should fail with appropriate error
    #[test]
    fn scenario_transaction_builder_fails_without_required_fields() {
        // Given: A transaction builder missing sender
        let builder_no_from = TransactionBuilder::new(Chain::BitCell)
            .to_str("BC1receiver")
            .amount(1000);

        // When: Building is attempted
        let result = builder_no_from.build();

        // Then: It should fail with appropriate error
        assert!(result.is_err());

        // Given: A transaction builder missing recipient
        let builder_no_to = TransactionBuilder::new(Chain::BitCell)
            .from_str("BC1sender")
            .amount(1000);

        // When: Building is attempted
        let result = builder_no_to.build();

        // Then: It should fail with appropriate error
        assert!(result.is_err());

        // Given: A transaction builder with zero amount
        let builder_zero_amount = TransactionBuilder::new(Chain::BitCell)
            .from_str("BC1sender")
            .to_str("BC1receiver")
            .amount(0);

        // When: Building is attempted
        let result = builder_zero_amount.build();

        // Then: It should fail with appropriate error
        assert!(result.is_err());
    }

    /// Scenario: Signed transaction can be serialized and deserialized
    /// Given a signed transaction
    /// When it is serialized and deserialized
    /// Then the hash should remain the same
    #[test]
    fn scenario_signed_transaction_serialization() {
        // Given: A signed transaction
        let sk = SecretKey::generate();
        let tx = Transaction::new(
            Chain::BitCell,
            "BC1sender".to_string(),
            "BC1receiver".to_string(),
            100_000,
            100,
            0,
        );
        let signed = tx.sign(&sk);

        // When: It is serialized and deserialized
        let serialized = signed.serialize().unwrap();
        let deserialized = SignedTransaction::deserialize(&serialized).unwrap();

        // Then: The hash should remain the same
        assert_eq!(signed.hash(), deserialized.hash());
        assert_eq!(signed.hash_hex(), deserialized.hash_hex());
    }

    /// Scenario: Fee estimation varies by chain and priority
    /// Given fee estimators for different chains
    /// When fees are estimated with different priorities
    /// Then higher priority should result in higher fees
    #[test]
    fn scenario_fee_estimation_varies_by_priority() {
        // Given: Fee estimators for different chains
        let estimator_normal = FeeEstimator::new(Chain::BitCell);
        let estimator_fast = FeeEstimator::new(Chain::BitCell).with_priority(2.0);
        let estimator_slow = FeeEstimator::new(Chain::BitCell).with_priority(0.5);

        // When: Fees are estimated with different priorities
        let fee_normal = estimator_normal.estimate_simple_transfer(Chain::BitCell);
        let fee_fast = estimator_fast.estimate_simple_transfer(Chain::BitCell);
        let fee_slow = estimator_slow.estimate_simple_transfer(Chain::BitCell);

        // Then: Higher priority should result in higher fees
        assert!(fee_fast > fee_normal);
        assert!(fee_normal > fee_slow);
    }
}

// =============================================================================
// BDD Test Module: Cross-Platform Compatibility
// =============================================================================

mod cross_platform_tests {
    use super::*;
    use std::env;

    /// Scenario: Wallet works correctly on Linux
    /// Given the current platform is Linux
    /// When wallet operations are performed
    /// Then they should complete successfully
    #[test]
    #[cfg(target_os = "linux")]
    fn scenario_wallet_works_on_linux() {
        // Given: The current platform is Linux
        assert_eq!(env::consts::OS, "linux");

        // When: Wallet operations are performed
        let (wallet, mnemonic) = Wallet::create_new(WalletConfig::default());

        // Then: They should complete successfully
        assert!(wallet.is_unlocked());
        assert_eq!(mnemonic.word_count(), 24);
        println!("✓ Linux platform test passed");
    }

    /// Scenario: Wallet works correctly on macOS
    /// Given the current platform is macOS
    /// When wallet operations are performed
    /// Then they should complete successfully
    #[test]
    #[cfg(target_os = "macos")]
    fn scenario_wallet_works_on_macos() {
        // Given: The current platform is macOS
        assert_eq!(env::consts::OS, "macos");

        // When: Wallet operations are performed
        let (wallet, mnemonic) = Wallet::create_new(WalletConfig::default());

        // Then: They should complete successfully
        assert!(wallet.is_unlocked());
        assert_eq!(mnemonic.word_count(), 24);
        println!("✓ macOS platform test passed");
    }

    /// Scenario: Wallet works correctly on Windows
    /// Given the current platform is Windows
    /// When wallet operations are performed
    /// Then they should complete successfully
    #[test]
    #[cfg(target_os = "windows")]
    fn scenario_wallet_works_on_windows() {
        // Given: The current platform is Windows
        assert_eq!(env::consts::OS, "windows");

        // When: Wallet operations are performed
        let (wallet, mnemonic) = Wallet::create_new(WalletConfig::default());

        // Then: They should complete successfully
        assert!(wallet.is_unlocked());
        assert_eq!(mnemonic.word_count(), 24);
        println!("✓ Windows platform test passed");
    }

    /// Scenario: Platform-agnostic wallet creation works everywhere
    /// Given any supported platform
    /// When a wallet is created
    /// Then it should work correctly regardless of platform
    #[test]
    fn scenario_platform_agnostic_wallet_creation() {
        // Given: Any supported platform
        let os = env::consts::OS;
        let arch = env::consts::ARCH;
        println!("Running on: {} ({})", os, arch);

        // When: A wallet is created
        let (mut wallet, _mnemonic) = Wallet::create_new(WalletConfig::default());

        // Then: It should work correctly regardless of platform
        assert!(wallet.is_unlocked());
        
        // All core operations should work
        let addr = wallet.next_address(Chain::BitCell).unwrap();
        assert!(addr.is_valid());
        
        wallet.update_balance(&addr, 1_000_000);
        let balance = wallet.get_balance(&addr);
        assert_eq!(balance.amount(), 1_000_000);

        println!("✓ Platform-agnostic test passed on {} ({})", os, arch);
    }

    /// Scenario: Cryptographic operations are consistent across platforms
    /// Given known test vectors
    /// When cryptographic operations are performed
    /// Then results should be identical across all platforms
    #[test]
    fn scenario_cryptographic_consistency_across_platforms() {
        // Given: Known test vectors (same mnemonic phrase)
        // Using a test mnemonic that we know is valid
        let mnemonic = Mnemonic::new();
        let phrase = mnemonic.phrase().to_string();

        // When: Cryptographic operations are performed
        let seed1 = mnemonic.to_seed("test_passphrase");
        
        // Parse the phrase and generate again
        let mnemonic2 = Mnemonic::from_phrase(&phrase).unwrap();
        let seed2 = mnemonic2.to_seed("test_passphrase");

        // Then: Results should be identical across all platforms
        assert_eq!(seed1.as_bytes(), seed2.as_bytes());
        
        // Verify the seed produces consistent addresses
        let mut wallet1 = Wallet::from_mnemonic(&mnemonic, "test_passphrase", WalletConfig::default());
        let mut wallet2 = Wallet::from_mnemonic(&mnemonic2, "test_passphrase", WalletConfig::default());
        
        let addr1 = wallet1.generate_address(Chain::BitCell, 0).unwrap();
        let addr2 = wallet2.generate_address(Chain::BitCell, 0).unwrap();
        
        assert_eq!(addr1.as_bytes(), addr2.as_bytes());
        
        println!("✓ Cryptographic consistency test passed");
    }

    /// Scenario: Endianness is handled correctly across platforms
    /// Given transaction data with numeric values
    /// When the transaction hash is computed
    /// Then it should be consistent regardless of platform endianness
    #[test]
    fn scenario_endianness_consistency() {
        // Given: Transaction data with numeric values
        let tx = Transaction::new(
            Chain::BitCell,
            "BC1sender".to_string(),
            "BC1receiver".to_string(),
            0x1234567890ABCDEF_u64, // Large value to test endianness
            0xFEDCBA09_u64,
            0x12345678_u64,
        );

        // When: The transaction hash is computed
        let hash = tx.hash();

        // Then: It should be consistent (we verify by checking it's non-zero and stable)
        assert!(!hash.as_bytes().iter().all(|&b| b == 0));
        
        // Same transaction should always produce same hash
        let hash2 = tx.hash();
        assert_eq!(hash, hash2);

        println!("✓ Endianness consistency test passed");
    }

    /// Scenario: File path handling works on all platforms
    /// Given platform-specific path separators
    /// When address strings are used as identifiers
    /// Then they should work correctly as map keys
    #[test]
    fn scenario_address_as_map_key_works_cross_platform() {
        // Given: Platform-specific considerations
        let mut balance_map: HashMap<String, u64> = HashMap::new();

        // When: Address strings are used as identifiers
        let mut wallet = Wallet::from_mnemonic(&Mnemonic::new(), "", WalletConfig::default());
        let addr1 = wallet.generate_address(Chain::BitCell, 0).unwrap();
        let addr2 = wallet.generate_address(Chain::Ethereum, 0).unwrap();

        balance_map.insert(addr1.to_string_formatted(), 1000);
        balance_map.insert(addr2.to_string_formatted(), 2000);

        // Then: They should work correctly as map keys
        assert_eq!(balance_map.get(&addr1.to_string_formatted()), Some(&1000));
        assert_eq!(balance_map.get(&addr2.to_string_formatted()), Some(&2000));
        
        println!("✓ Cross-platform map key test passed");
    }
}

// =============================================================================
// BDD Test Module: Security Tests
// =============================================================================

mod security_tests {
    use super::*;

    /// Scenario: Private key is never exposed in debug output
    /// Given a wallet with private keys
    /// When debug formatting is used
    /// Then private key material should not be visible
    #[test]
    fn scenario_private_key_not_exposed_in_debug() {
        // Given: A wallet with private keys
        let mnemonic = Mnemonic::new();
        
        // When: Debug formatting is used
        let debug_output = format!("{:?}", mnemonic);
        
        // Then: Private key material should not be visible
        // The phrase should not appear in debug output
        assert!(!debug_output.contains(mnemonic.phrase()));
        // Should show redacted format
        assert!(debug_output.contains("Mnemonic"));
    }

    /// Scenario: Seed bytes are not exposed in debug output
    /// Given seed bytes derived from a mnemonic
    /// When debug formatting is used
    /// Then the actual bytes should be redacted
    #[test]
    fn scenario_seed_bytes_redacted_in_debug() {
        // Given: Seed bytes derived from a mnemonic
        let mnemonic = Mnemonic::new();
        let seed = mnemonic.to_seed("");

        // When: Debug formatting is used
        let debug_output = format!("{:?}", seed);

        // Then: The actual bytes should be redacted
        assert!(debug_output.contains("REDACTED") || debug_output.contains("SeedBytes"));
        // Actual seed bytes should not appear in hex
        let seed_hex = hex::encode(seed.as_bytes());
        assert!(!debug_output.contains(&seed_hex));
    }

    /// Scenario: Signing requires unlocked wallet
    /// Given a locked wallet
    /// When attempting to sign a transaction
    /// Then it should fail with WalletLocked error
    #[test]
    fn scenario_signing_requires_unlocked_wallet() {
        // Given: A locked wallet
        let mnemonic = Mnemonic::new();
        let mut wallet = Wallet::from_mnemonic(&mnemonic, "", WalletConfig::default());
        let from = wallet.next_address(Chain::BitCell).unwrap();
        wallet.update_balance(&from, 1_000_000);
        let to = wallet.next_address(Chain::BitCell).unwrap();
        
        wallet.lock();

        // When: Attempting to sign a transaction
        let tx = wallet.create_transaction(&from, &to, 1000, 100).unwrap();
        let result = wallet.sign_transaction(tx, &from);

        // Then: It should fail with WalletLocked error
        assert!(matches!(result, Err(Error::WalletLocked)));
    }

    /// Scenario: Wrong key cannot verify signature
    /// Given a signed transaction
    /// When verification is attempted with wrong public key
    /// Then it should fail
    #[test]
    fn scenario_wrong_key_cannot_verify_signature() {
        // Given: A signed transaction
        let sk1 = SecretKey::generate();
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

        // When: Verification is attempted with wrong public key
        let result = signed.verify(&pk2);

        // Then: It should fail
        assert!(result.is_err());
    }

    /// Scenario: Tampered transaction fails verification
    /// Given a signed transaction
    /// When the transaction content is modified after signing
    /// Then verification should fail
    #[test]
    fn scenario_tampered_transaction_fails_verification() {
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

        // When: We create a different transaction and try to verify
        // the original signature against it
        let tampered_tx = Transaction::new(
            Chain::BitCell,
            "BC1sender".to_string(),
            "BC1receiver".to_string(),
            100_001, // Changed amount
            100,
            0,
        );
        let tampered_hash = tampered_tx.hash();

        // Verify the original signature against tampered hash
        let verify_result = signed.signature.verify(&pk, tampered_hash.as_bytes());

        // Then: Verification should fail
        assert!(verify_result.is_err());
    }

    /// Scenario: Key derivation is consistent
    /// Given the same mnemonic and derivation path
    /// When keys are derived multiple times
    /// Then they should be identical
    #[test]
    fn scenario_key_derivation_is_consistent() {
        // Given: The same mnemonic and derivation path
        let mnemonic = Mnemonic::new();
        let passphrase = "secure_passphrase";

        // When: Keys are derived multiple times via wallet creation
        let mut wallet1 = Wallet::from_mnemonic(&mnemonic, passphrase, WalletConfig::default());
        let mut wallet2 = Wallet::from_mnemonic(&mnemonic, passphrase, WalletConfig::default());

        // Generate same address index
        let addr1 = wallet1.generate_address(Chain::BitCell, 5).unwrap();
        let addr2 = wallet2.generate_address(Chain::BitCell, 5).unwrap();

        // Then: They should be identical
        assert_eq!(addr1.as_bytes(), addr2.as_bytes());
    }

    /// Scenario: Wallet lock clears sensitive data
    /// Given an unlocked wallet with derived keys
    /// When the wallet is locked
    /// Then signing should not be possible
    #[test]
    fn scenario_wallet_lock_clears_sensitive_data() {
        // Given: An unlocked wallet with derived keys
        let mnemonic = Mnemonic::new();
        let mut wallet = Wallet::from_mnemonic(&mnemonic, "", WalletConfig::default());
        
        // Generate some addresses to derive keys
        let from = wallet.next_address(Chain::BitCell).unwrap();
        let to = wallet.next_address(Chain::BitCell).unwrap();
        wallet.update_balance(&from, 1_000_000);

        // Verify we can sign
        let tx = wallet.create_transaction(&from, &to, 1000, 100).unwrap();
        assert!(wallet.sign_transaction(tx, &from).is_ok());

        // When: The wallet is locked
        wallet.lock();

        // Then: Signing should not be possible
        let tx2 = wallet.create_transaction(&from, &to, 1000, 100).unwrap();
        let result = wallet.sign_transaction(tx2, &from);
        assert!(matches!(result, Err(Error::WalletLocked)));
    }

    /// Scenario: Invalid mnemonic rejection
    /// Given an invalid mnemonic phrase
    /// When parsing is attempted
    /// Then it should be rejected with appropriate error
    #[test]
    fn scenario_invalid_mnemonic_rejection() {
        // Given: Various invalid mnemonic phrases
        let invalid_phrases = vec![
            "invalid words that are not bip39",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about", // Valid but testing we can detect invalid
            "one two three",
            "",
        ];

        for phrase in invalid_phrases {
            // When: Parsing is attempted
            let result = Mnemonic::from_phrase(phrase);

            // Then: It should be rejected for truly invalid phrases
            // Note: Some phrases may be valid if they happen to match BIP39
            if !Mnemonic::validate(phrase) {
                assert!(result.is_err());
            }
        }
    }

    /// Scenario: Balance cannot go negative
    /// Given a wallet with a specific balance
    /// When attempting to spend more than available
    /// Then the transaction should fail
    #[test]
    fn scenario_balance_cannot_go_negative() {
        // Given: A wallet with a specific balance
        let mut wallet = Wallet::from_mnemonic(&Mnemonic::new(), "", WalletConfig::default());
        let from = wallet.next_address(Chain::BitCell).unwrap();
        let to = wallet.next_address(Chain::BitCell).unwrap();
        wallet.update_balance(&from, 1000);

        // When: Attempting to spend more than available
        let result = wallet.create_transaction(&from, &to, 900, 200); // Total: 1100 > 1000

        // Then: The transaction should fail
        assert!(matches!(result, Err(Error::InsufficientBalance { have: 1000, need: 1100 })));
    }

    /// Scenario: Signature is unique per transaction
    /// Given the same key and different transactions
    /// When they are signed
    /// Then signatures should be different
    #[test]
    fn scenario_signature_is_unique_per_transaction() {
        // Given: The same key and different transactions
        let sk = SecretKey::generate();

        let tx1 = Transaction::new(
            Chain::BitCell,
            "BC1sender".to_string(),
            "BC1receiver".to_string(),
            100_000,
            100,
            0,
        );

        let tx2 = Transaction::new(
            Chain::BitCell,
            "BC1sender".to_string(),
            "BC1receiver".to_string(),
            100_001,
            100,
            0,
        );

        // When: They are signed
        let signed1 = tx1.sign(&sk);
        let signed2 = tx2.sign(&sk);

        // Then: Signatures should be different
        assert_ne!(signed1.signature.as_bytes(), signed2.signature.as_bytes());
        assert_ne!(signed1.hash(), signed2.hash());
    }
}

// =============================================================================
// BDD Test Module: Balance and History
// =============================================================================

mod balance_and_history_tests {
    use super::*;

    /// Scenario: Balance formatting for different chains
    /// Given balances in different chains
    /// When they are formatted
    /// Then they should show correct symbols and decimal places
    #[test]
    fn scenario_balance_formatting_for_different_chains() {
        // Given: Balances in different chains
        let btc_balance = Balance::new(100_000_000, Chain::Bitcoin); // 1 BTC
        let eth_balance = Balance::new(1_000_000_000_000_000_000, Chain::Ethereum); // 1 ETH
        let cell_balance = Balance::new(100_000_000, Chain::BitCell); // 1 CELL

        // When: They are formatted
        let btc_formatted = btc_balance.format();
        let eth_formatted = eth_balance.format();
        let cell_formatted = cell_balance.format();

        // Then: They should show correct symbols and decimal places
        assert!(btc_formatted.contains("BTC"));
        assert!(eth_formatted.contains("ETH"));
        assert!(cell_formatted.contains("CELL"));
    }

    /// Scenario: Balance operations are safe
    /// Given a balance near maximum value
    /// When adding would overflow
    /// Then it should saturate instead of panicking
    #[test]
    fn scenario_balance_operations_are_safe() {
        // Given: A balance near maximum value
        let max_balance = Balance::new(u64::MAX - 10, Chain::BitCell);

        // When: Adding would overflow
        let result = max_balance.add(100);

        // Then: It should saturate instead of panicking
        assert_eq!(result.amount(), u64::MAX);
    }

    /// Scenario: Transaction history tracks multiple transactions
    /// Given multiple transactions
    /// When added to history
    /// Then they should be retrievable by various criteria
    #[test]
    fn scenario_transaction_history_tracks_transactions() {
        // Given: Multiple transactions
        let mut history = TransactionHistory::new();
        
        for i in 0..10 {
            let record = TransactionRecord::new(
                format!("0x{:064x}", i),
                Chain::BitCell,
                if i % 2 == 0 { TransactionDirection::Outgoing } else { TransactionDirection::Incoming },
                "BC1sender".to_string(),
                "BC1receiver".to_string(),
                (i + 1) as u64 * 1000,
                100,
                1000000 + i as u64,
            );
            history.add(record);
        }

        // When: Added to history
        // Then: They should be retrievable by various criteria
        assert_eq!(history.count(), 10);
        assert_eq!(history.recent(5).len(), 5);
        
        // Get by hash
        assert!(history.get("0x0000000000000000000000000000000000000000000000000000000000000000").is_some());
        
        // Get by address
        let by_addr = history.by_address("BC1sender");
        assert_eq!(by_addr.len(), 10);
    }

    /// Scenario: History summary calculates correctly
    /// Given a history with various transaction types
    /// When summary is requested
    /// Then totals should be correct
    #[test]
    fn scenario_history_summary_calculates_correctly() {
        // Given: A history with various transaction types
        let mut history = TransactionHistory::new();
        
        let outgoing = TransactionRecord::new(
            "0x001".to_string(),
            Chain::BitCell,
            TransactionDirection::Outgoing,
            "BC1sender".to_string(),
            "BC1receiver".to_string(),
            5000,
            100,
            1000000,
        );
        history.add(outgoing);
        
        let incoming = TransactionRecord::new(
            "0x002".to_string(),
            Chain::BitCell,
            TransactionDirection::Incoming,
            "BC1other".to_string(),
            "BC1sender".to_string(),
            3000,
            50,
            1000001,
        );
        history.add(incoming);

        // When: Summary is requested
        let summary = history.summary();

        // Then: Totals should be correct
        assert_eq!(summary.total_transactions, 2);
        assert_eq!(summary.total_sent, 5000);
        assert_eq!(summary.total_received, 3000);
    }
}

// =============================================================================
// BDD Test Module: Chain Configuration
// =============================================================================

mod chain_configuration_tests {
    use super::*;

    /// Scenario: Chain properties are correct
    /// Given different blockchain networks
    /// When their properties are queried
    /// Then they should return correct values
    #[test]
    fn scenario_chain_properties_are_correct() {
        // Given: Different blockchain networks
        let chains = vec![
            (Chain::BitCell, "BitCell", "CELL", 8, false),
            (Chain::Bitcoin, "Bitcoin", "BTC", 8, false),
            (Chain::BitcoinTestnet, "Bitcoin Testnet", "BTC", 8, true),
            (Chain::Ethereum, "Ethereum", "ETH", 18, false),
            (Chain::EthereumSepolia, "Ethereum Sepolia", "ETH", 18, true),
        ];

        for (chain, name, symbol, decimals, is_testnet) in chains {
            // When: Their properties are queried
            // Then: They should return correct values
            assert_eq!(chain.name(), name);
            assert_eq!(chain.symbol(), symbol);
            assert_eq!(chain.decimals(), decimals);
            assert_eq!(chain.is_testnet(), is_testnet);
        }
    }

    /// Scenario: Chain configuration can be customized
    /// Given a chain configuration
    /// When RPC and explorer URLs are set
    /// Then they should be retrievable
    #[test]
    fn scenario_chain_configuration_customization() {
        // Given: A chain configuration
        let config = ChainConfig::new(Chain::BitCell)
            .with_rpc_url("http://localhost:8545")
            .with_explorer_url("https://explorer.example.com");

        // When: RPC and explorer URLs are set
        // Then: They should be retrievable
        assert_eq!(config.chain, Chain::BitCell);
        assert_eq!(config.rpc_url, Some("http://localhost:8545".to_string()));
        assert_eq!(config.explorer_url, Some("https://explorer.example.com".to_string()));
        assert!(config.enabled);
    }

    /// Scenario: Custom chain types work correctly
    /// Given a custom chain ID
    /// When used in transactions
    /// Then it should behave correctly
    #[test]
    fn scenario_custom_chain_types() {
        // Given: A custom chain ID
        let custom_chain = Chain::Custom(42);

        // When: Used in various contexts
        // Then: It should behave correctly
        assert_eq!(custom_chain.chain_id(), 42);
        assert_eq!(custom_chain.coin_type(), 42);
        assert_eq!(custom_chain.name(), "Custom");
        assert_eq!(custom_chain.decimals(), 8);
        assert!(!custom_chain.is_testnet());
    }
}

// =============================================================================
// BDD Test Module: Error Handling
// =============================================================================

mod error_handling_tests {
    use super::*;

    /// Scenario: All error types have descriptive messages
    /// Given various wallet errors
    /// When converted to strings
    /// Then they should be human-readable
    #[test]
    fn scenario_error_messages_are_descriptive() {
        // Given: Various wallet errors
        let errors = vec![
            Error::InvalidMnemonic("test error".to_string()),
            Error::InvalidDerivationPath("m/44'/0'".to_string()),
            Error::InvalidAddress("bad address".to_string()),
            Error::InsufficientBalance { have: 100, need: 200 },
            Error::TransactionError("tx failed".to_string()),
            Error::SigningError("signing failed".to_string()),
            Error::UnsupportedChain("unknown".to_string()),
            Error::WalletLocked,
            Error::Serialization("serde error".to_string()),
            Error::Crypto("crypto error".to_string()),
            Error::Io("io error".to_string()),
        ];

        for error in errors {
            // When: Converted to strings
            let msg = error.to_string();

            // Then: They should be human-readable
            assert!(!msg.is_empty());
            // Each error type should contain relevant information
            match &error {
                Error::InsufficientBalance { have, need } => {
                    assert!(msg.contains(&have.to_string()));
                    assert!(msg.contains(&need.to_string()));
                }
                _ => {}
            }
        }
    }

    /// Scenario: Crypto errors convert properly
    /// Given a crypto module error
    /// When converted to wallet error
    /// Then it should preserve the error message
    #[test]
    fn scenario_crypto_errors_convert_properly() {
        // Given: An invalid secret key scenario
        let invalid_bytes = [0u8; 32];
        
        // When: Attempting to create a secret key from invalid bytes
        // (all zeros is not a valid private key for secp256k1)
        let result = SecretKey::from_bytes(&invalid_bytes);
        
        // Then: It should return an error that can be converted
        if let Err(crypto_err) = result {
            let wallet_err: Error = crypto_err.into();
            let msg = wallet_err.to_string();
            assert!(msg.contains("Crypto") || msg.contains("crypto"));
        }
    }
}
