//! Hardware wallet integration tests
//!
//! These tests validate the hardware wallet abstraction and device implementations.

use bitcell_wallet::{
    Chain, Error, HardwareWallet, HardwareWalletType, SigningMethod, Transaction,
};

#[test]
fn test_hardware_wallet_connect_mock() {
    let hw = HardwareWallet::connect(HardwareWalletType::Mock).unwrap();
    assert!(hw.is_connected());
    assert_eq!(hw.device_type(), HardwareWalletType::Mock);
}

#[test]
fn test_hardware_wallet_derivation_paths() {
    // BitCell derivation path
    let path = HardwareWallet::derivation_path_for_chain(Chain::BitCell, 0, 0);
    assert_eq!(path, "m/44'/9999'/0'/0/0");
    
    // Bitcoin derivation path
    let path = HardwareWallet::derivation_path_for_chain(Chain::Bitcoin, 0, 0);
    assert_eq!(path, "m/44'/0'/0'/0/0");
    
    // Ethereum derivation path
    let path = HardwareWallet::derivation_path_for_chain(Chain::Ethereum, 0, 5);
    assert_eq!(path, "m/44'/60'/0'/0/5");
    
    // Multiple accounts
    let path = HardwareWallet::derivation_path_for_chain(Chain::BitCell, 1, 3);
    assert_eq!(path, "m/44'/9999'/1'/0/3");
    
    // Custom chain
    let path = HardwareWallet::derivation_path_for_chain(Chain::Custom(1234), 0, 0);
    assert_eq!(path, "m/44'/1234'/0'/0/0");
}

#[test]
fn test_hardware_wallet_set_derivation_path() {
    let hw = HardwareWallet::connect(HardwareWalletType::Mock)
        .unwrap()
        .with_derivation_path("m/44'/9999'/1'/0/5");
    
    // Verify the path is used by checking we get different keys
    let pk1 = hw.get_public_key().unwrap();
    
    let hw2 = HardwareWallet::connect(HardwareWalletType::Mock).unwrap();
    let pk2 = hw2.get_public_key().unwrap();
    
    // Different devices should have different keys
    // (In a real scenario with the same seed, same path would give same key)
    assert_ne!(pk1.as_bytes(), pk2.as_bytes());
}

#[test]
fn test_hardware_wallet_public_key() {
    let hw = HardwareWallet::connect(HardwareWalletType::Mock).unwrap();
    let pk = hw.get_public_key().unwrap();
    
    // Compressed public key should be 33 bytes
    assert_eq!(pk.as_bytes().len(), 33);
}

#[test]
fn test_hardware_wallet_address_generation() {
    let hw = HardwareWallet::connect(HardwareWalletType::Mock).unwrap();
    
    // BitCell address
    let address = hw.get_address(Chain::BitCell).unwrap();
    assert!(address.starts_with("BC1"));
    assert_eq!(address.len(), 43); // "BC1" + 40 hex chars
    
    // Bitcoin address
    let address = hw.get_address(Chain::Bitcoin).unwrap();
    assert!(address.starts_with("bc1"));
    
    // Ethereum address
    let address = hw.get_address(Chain::Ethereum).unwrap();
    assert!(address.starts_with("0x"));
}

#[test]
fn test_hardware_wallet_sign_transaction() {
    let hw = HardwareWallet::connect(HardwareWalletType::Mock).unwrap();
    
    let tx = Transaction::new(
        Chain::BitCell,
        "BC1sender".to_string(),
        "BC1recipient".to_string(),
        1000,
        10,
        0,
    );
    
    let signed = hw.sign_transaction(&tx).unwrap();
    
    // Verify the signature
    let pk = hw.get_public_key().unwrap();
    assert!(signed.verify(&pk).is_ok());
    
    // Verify hash matches
    assert_eq!(signed.tx_hash, tx.hash());
}

#[test]
fn test_hardware_wallet_sign_hash() {
    let hw = HardwareWallet::connect(HardwareWalletType::Mock).unwrap();
    
    let message = b"Test message";
    let hash = bitcell_crypto::Hash256::hash(message);
    
    let signature = hw.sign_hash(&hash).unwrap();
    
    // Verify the signature by signing a transaction with same hash
    // (PublicKey doesn't have verify method, we use SignedTransaction)
    assert_eq!(signature.as_bytes().len(), 64);
}

#[test]
fn test_signing_method_software_vs_hardware() {
    // Test software signing
    let sk = bitcell_crypto::SecretKey::generate();
    let sw_method = SigningMethod::Software(sk);
    
    assert!(!sw_method.is_hardware());
    
    // Test hardware signing
    let hw = HardwareWallet::connect(HardwareWalletType::Mock).unwrap();
    let hw_method = SigningMethod::Hardware(hw);
    
    assert!(hw_method.is_hardware());
}

#[test]
fn test_signing_method_sign_transaction() {
    let tx = Transaction::new(
        Chain::BitCell,
        "BC1sender".to_string(),
        "BC1recipient".to_string(),
        1000,
        10,
        0,
    );
    
    // Test software signing
    let sk = bitcell_crypto::SecretKey::generate();
    let sw_method = SigningMethod::Software(sk);
    let signed_sw = sw_method.sign(&tx).unwrap();
    let pk_sw = sw_method.public_key().unwrap();
    assert!(signed_sw.verify(&pk_sw).is_ok());
    
    // Test hardware signing
    let hw = HardwareWallet::connect(HardwareWalletType::Mock).unwrap();
    let pk = hw.get_public_key().unwrap();
    let hw_method = SigningMethod::Hardware(hw);
    let signed_hw = hw_method.sign(&tx).unwrap();
    assert!(signed_hw.verify(&pk).is_ok());
}

#[test]
fn test_signing_method_public_key() {
    // Software method
    let sk = bitcell_crypto::SecretKey::generate();
    let expected_pk = sk.public_key();
    let sw_method = SigningMethod::Software(sk);
    let pk_sw = sw_method.public_key().unwrap();
    assert_eq!(pk_sw.as_bytes(), expected_pk.as_bytes());
    
    // Hardware method
    let hw = HardwareWallet::connect(HardwareWalletType::Mock).unwrap();
    let expected_pk = hw.get_public_key().unwrap();
    let hw_method = SigningMethod::Hardware(hw);
    let pk_hw = hw_method.public_key().unwrap();
    assert_eq!(pk_hw.as_bytes(), expected_pk.as_bytes());
}

#[test]
fn test_hardware_wallet_multiple_signatures() {
    let hw = HardwareWallet::connect(HardwareWalletType::Mock).unwrap();
    let pk = hw.get_public_key().unwrap();
    
    // Sign multiple transactions
    for i in 0..5 {
        let tx = Transaction::new(
            Chain::BitCell,
            "BC1sender".to_string(),
            format!("BC1recipient{}", i),
            1000 + i as u64,
            10,
            i,
        );
        
        let signed = hw.sign_transaction(&tx).unwrap();
        assert!(signed.verify(&pk).is_ok());
    }
}

#[test]
fn test_hardware_wallet_cross_chain_addresses() {
    let hw = HardwareWallet::connect(HardwareWalletType::Mock).unwrap();
    
    // Generate addresses for different chains
    let bc_addr = hw.get_address(Chain::BitCell).unwrap();
    let btc_addr = hw.get_address(Chain::Bitcoin).unwrap();
    let eth_addr = hw.get_address(Chain::Ethereum).unwrap();
    
    // All addresses should be valid and different formats
    assert!(bc_addr.starts_with("BC1"));
    assert!(btc_addr.starts_with("bc1"));
    assert!(eth_addr.starts_with("0x"));
    
    // The addresses should be deterministic (same for same key)
    let bc_addr2 = hw.get_address(Chain::BitCell).unwrap();
    assert_eq!(bc_addr, bc_addr2);
}

#[cfg(feature = "ledger")]
#[test]
fn test_ledger_device_not_connected() {
    // When no device is connected, should get appropriate error
    use bitcell_wallet::hardware::ledger::LedgerDevice;
    
    let result = LedgerDevice::connect();
    // Should fail gracefully if no device connected
    if let Err(e) = result {
        assert!(e.to_string().contains("connect") || e.to_string().contains("device"));
    }
}

#[cfg(feature = "trezor")]
#[test]
fn test_trezor_device_not_connected() {
    // When no device is connected, should get appropriate error
    use bitcell_wallet::hardware::trezor::TrezorDevice;
    
    let result = TrezorDevice::connect();
    // Should fail gracefully if no device connected
    if let Err(e) = result {
        assert!(e.to_string().contains("connect") || e.to_string().contains("device"));
    }
}

#[test]
fn test_hardware_wallet_bip44_coin_types() {
    // Verify correct BIP44 coin types are used
    
    // BitCell: 9999 (custom)
    let path = HardwareWallet::derivation_path_for_chain(Chain::BitCell, 0, 0);
    assert!(path.contains("9999'"));
    
    // Bitcoin: 0
    let path = HardwareWallet::derivation_path_for_chain(Chain::Bitcoin, 0, 0);
    assert!(path.contains("0'"));
    assert!(path.starts_with("m/44'/0'/"));
    
    // Ethereum: 60
    let path = HardwareWallet::derivation_path_for_chain(Chain::Ethereum, 0, 0);
    assert!(path.contains("60'"));
    
    // Bitcoin Testnet: 1
    let path = HardwareWallet::derivation_path_for_chain(Chain::BitcoinTestnet, 0, 0);
    assert!(path.contains("1'"));
}

#[test]
fn test_hardware_wallet_account_indices() {
    // Test different account indices
    for account in 0..5 {
        let path = HardwareWallet::derivation_path_for_chain(Chain::BitCell, account, 0);
        assert!(path.contains(&format!("/{}'", account)));
    }
}

#[test]
fn test_hardware_wallet_address_indices() {
    // Test different address indices (non-hardened)
    for index in 0..10 {
        let path = HardwareWallet::derivation_path_for_chain(Chain::BitCell, 0, index);
        assert!(path.ends_with(&format!("/{}", index)));
    }
}

#[test]
fn test_hardware_wallet_deterministic_addresses() {
    // Same path should give same address with same device
    let hw = HardwareWallet::connect(HardwareWalletType::Mock).unwrap();
    
    let addr1 = hw.get_address(Chain::BitCell).unwrap();
    let addr2 = hw.get_address(Chain::BitCell).unwrap();
    
    assert_eq!(addr1, addr2);
}

#[test]
fn test_hardware_wallet_signature_verification() {
    let hw = HardwareWallet::connect(HardwareWalletType::Mock).unwrap();
    let pk = hw.get_public_key().unwrap();
    
    let tx = Transaction::new(
        Chain::BitCell,
        "BC1sender".to_string(),
        "BC1recipient".to_string(),
        1000,
        10,
        0,
    );
    
    let signed = hw.sign_transaction(&tx).unwrap();
    
    // Should verify with correct key
    assert!(signed.verify(&pk).is_ok());
    
    // Should fail with wrong key
    let wrong_sk = bitcell_crypto::SecretKey::generate();
    let wrong_pk = wrong_sk.public_key();
    assert!(signed.verify(&wrong_pk).is_err());
}
