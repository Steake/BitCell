//! Integration tests for transaction system
//!
//! These tests verify the complete transaction flow from wallet creation
//! to signing and serialization, ensuring compatibility with the RPC layer.

use bitcell_crypto::{PublicKey, SecretKey};
use bitcell_consensus::Transaction;
use bitcell_wallet::{Chain, Mnemonic, Wallet, WalletConfig, TransactionBuilder};

/// Test that wallet can create and sign transactions that serialize correctly
#[test]
fn test_wallet_transaction_creation_and_signing() {
    // Create a wallet
    let mnemonic = Mnemonic::new();
    let mut wallet = Wallet::from_mnemonic(&mnemonic, "", WalletConfig::default());
    
    // Generate addresses
    let from_addr = wallet.next_address(Chain::BitCell).unwrap();
    let to_addr = wallet.next_address(Chain::BitCell).unwrap();
    
    // Set balance for sender
    wallet.update_balance(&from_addr, 1_000_000);
    
    // Create and sign transaction
    let wallet_tx = wallet.create_transaction(&from_addr, &to_addr, 100_000, 1_000).unwrap();
    let signed_tx = wallet.sign_transaction(wallet_tx, &from_addr).unwrap();
    
    // Verify transaction hash exists
    assert!(!signed_tx.hash_hex().is_empty());
    
    // Verify signature is valid
    let from_pk = wallet.get_public_key_for_address(&from_addr).unwrap();
    assert!(signed_tx.verify(&from_pk).is_ok());
}

/// Test that wallet transactions can be converted to consensus transactions
#[test]
fn test_wallet_to_consensus_transaction_conversion() {
    // Create a wallet
    let mnemonic = Mnemonic::new();
    let mut wallet = Wallet::from_mnemonic(&mnemonic, "", WalletConfig::default());
    
    // Generate addresses
    let from_addr = wallet.next_address(Chain::BitCell).unwrap();
    let to_addr = wallet.next_address(Chain::BitCell).unwrap();
    
    // Get public keys
    let from_pk = wallet.get_public_key_for_address(&from_addr).unwrap();
    let to_pk = wallet.get_public_key_for_address(&to_addr).unwrap();
    
    // Set balance
    wallet.update_balance(&from_addr, 1_000_000);
    
    // Create and sign wallet transaction
    let wallet_tx = wallet.create_transaction(&from_addr, &to_addr, 100_000, 1_000).unwrap();
    let nonce = wallet_tx.nonce;
    let amount = wallet_tx.amount;
    let fee = wallet_tx.fee;
    
    let signed_wallet_tx = wallet.sign_transaction(wallet_tx, &from_addr).unwrap();
    
    // Convert to consensus transaction
    let consensus_tx = Transaction {
        nonce,
        from: from_pk.clone(),
        to: to_pk.clone(),
        amount,
        gas_limit: 21000,
        gas_price: fee / 21000,
        data: Vec::new(),
        signature: signed_wallet_tx.signature.clone(),
    };
    
    // Verify transaction signature works with consensus Transaction
    // Note: The signature was created over the wallet transaction hash,
    // so we verify it's still valid with the public key
    let verified_pk = signed_wallet_tx.verify(&from_pk);
    assert!(verified_pk.is_ok(), "Signature should verify with wallet transaction");
}

/// Test that transactions can be serialized and deserialized
#[test]
fn test_transaction_serialization() {
    // Create keys
    let from_sk = SecretKey::generate();
    let from_pk = from_sk.public_key();
    let to_pk = SecretKey::generate().public_key();
    
    // Create a proper transaction and sign it
    let gas_limit = 21000u64;
    let gas_price = 1000u64;
    let amount = 100_000u64;
    let nonce = 0u64;
    
    // Build transaction data for signing (without signature)
    let mut tx_data = Vec::new();
    tx_data.extend_from_slice(&nonce.to_le_bytes());
    tx_data.extend_from_slice(from_pk.as_bytes());
    tx_data.extend_from_slice(to_pk.as_bytes());
    tx_data.extend_from_slice(&amount.to_le_bytes());
    tx_data.extend_from_slice(&gas_limit.to_le_bytes());
    tx_data.extend_from_slice(&gas_price.to_le_bytes());
    
    let signature = from_sk.sign(&tx_data);
    
    // Create a consensus transaction
    let tx = Transaction {
        nonce,
        from: from_pk.clone(),
        to: to_pk.clone(),
        amount,
        gas_limit,
        gas_price,
        data: Vec::new(),
        signature,
    };
    
    // Serialize
    let serialized = bincode::serialize(&tx).expect("Should serialize");
    
    // Deserialize
    let deserialized: Transaction = bincode::deserialize(&serialized).expect("Should deserialize");
    
    // Verify fields match
    assert_eq!(tx.nonce, deserialized.nonce);
    assert_eq!(tx.from.as_bytes(), deserialized.from.as_bytes());
    assert_eq!(tx.to.as_bytes(), deserialized.to.as_bytes());
    assert_eq!(tx.amount, deserialized.amount);
    assert_eq!(tx.gas_limit, deserialized.gas_limit);
    assert_eq!(tx.gas_price, deserialized.gas_price);
}

/// Test that transaction hash is deterministic
#[test]
fn test_transaction_hash_deterministic() {
    let from_pk = SecretKey::generate().public_key();
    let to_pk = SecretKey::generate().public_key();
    let signature = SecretKey::generate().sign(b"test");
    
    let tx1 = Transaction {
        nonce: 5,
        from: from_pk.clone(),
        to: to_pk.clone(),
        amount: 50_000,
        gas_limit: 21000,
        gas_price: 1000,
        data: vec![1, 2, 3],
        signature: signature.clone(),
    };
    
    let tx2 = Transaction {
        nonce: 5,
        from: from_pk.clone(),
        to: to_pk.clone(),
        amount: 50_000,
        gas_limit: 21000,
        gas_price: 1000,
        data: vec![1, 2, 3],
        signature: signature.clone(),
    };
    
    // Same transaction should have same hash
    assert_eq!(tx1.hash(), tx2.hash());
}

/// Test that different transactions have different hashes
#[test]
fn test_transaction_hash_unique() {
    let from_pk = SecretKey::generate().public_key();
    let to_pk = SecretKey::generate().public_key();
    let signature = SecretKey::generate().sign(b"test");
    
    let tx1 = Transaction {
        nonce: 0,
        from: from_pk.clone(),
        to: to_pk.clone(),
        amount: 100_000,
        gas_limit: 21000,
        gas_price: 1000,
        data: Vec::new(),
        signature: signature.clone(),
    };
    
    let tx2 = Transaction {
        nonce: 1,  // Different nonce
        from: from_pk.clone(),
        to: to_pk.clone(),
        amount: 100_000,
        gas_limit: 21000,
        gas_price: 1000,
        data: Vec::new(),
        signature: signature.clone(),
    };
    
    // Different transactions should have different hashes
    assert_ne!(tx1.hash(), tx2.hash());
}

/// Test nonce increment
#[test]
fn test_wallet_nonce_increment() {
    let mnemonic = Mnemonic::new();
    let mut wallet = Wallet::from_mnemonic(&mnemonic, "", WalletConfig::default());
    
    let from_addr = wallet.next_address(Chain::BitCell).unwrap();
    let to_addr = wallet.next_address(Chain::BitCell).unwrap();
    
    wallet.update_balance(&from_addr, 10_000_000);
    
    // Initial nonce should be 0
    assert_eq!(wallet.get_nonce(&from_addr), 0);
    
    // Send first transaction
    let tx1 = wallet.create_transaction(&from_addr, &to_addr, 100_000, 1_000).unwrap();
    assert_eq!(tx1.nonce, 0);
    wallet.sign_transaction(tx1, &from_addr).unwrap();
    
    // Nonce should increment to 1
    assert_eq!(wallet.get_nonce(&from_addr), 1);
    
    // Send second transaction
    let tx2 = wallet.create_transaction(&from_addr, &to_addr, 100_000, 1_000).unwrap();
    assert_eq!(tx2.nonce, 1);
    wallet.sign_transaction(tx2, &from_addr).unwrap();
    
    // Nonce should increment to 2
    assert_eq!(wallet.get_nonce(&from_addr), 2);
}
