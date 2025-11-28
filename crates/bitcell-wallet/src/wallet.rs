//! Wallet Core
//!
//! Main wallet functionality that integrates all components.

use crate::{
    Address, Balance, Chain, ChainConfig, Error, Mnemonic, Result,
    SignedTransaction, Transaction, TransactionBuilder, TransactionHistory,
    address::AddressManager,
    balance::BalanceTracker,
    mnemonic::SeedBytes,
};
use bitcell_crypto::{Hash256, PublicKey, SecretKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Wallet configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    /// Wallet name
    pub name: String,
    /// Enabled chains
    pub chains: Vec<ChainConfig>,
    /// Auto-generate addresses
    pub auto_generate_addresses: bool,
    /// Number of addresses to pre-generate per chain
    pub address_lookahead: u32,
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            name: "Default Wallet".to_string(),
            chains: vec![
                ChainConfig::new(Chain::BitCell),
                ChainConfig::new(Chain::Bitcoin),
                ChainConfig::new(Chain::Ethereum),
            ],
            auto_generate_addresses: true,
            address_lookahead: 5,
        }
    }
}

/// Key derivation path
#[derive(Debug, Clone)]
pub struct DerivationPath {
    /// Purpose (44' for BIP44)
    pub purpose: u32,
    /// Coin type
    pub coin_type: u32,
    /// Account
    pub account: u32,
    /// Change (0 = external, 1 = internal)
    pub change: u32,
    /// Address index
    pub index: u32,
}

impl DerivationPath {
    /// Create a new derivation path for BIP44
    pub fn bip44(coin_type: u32, account: u32, change: u32, index: u32) -> Self {
        Self {
            purpose: 44,
            coin_type,
            account,
            change,
            index,
        }
    }

    /// Create path for a specific chain and index
    pub fn for_chain(chain: Chain, index: u32) -> Self {
        Self::bip44(chain.coin_type(), 0, 0, index)
    }
}

impl std::fmt::Display for DerivationPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "m/{}'/{}'/{}'/{}'/{}",
            self.purpose, self.coin_type, self.account, self.change, self.index
        )
    }
}

/// Derived key pair
struct DerivedKey {
    secret_key: SecretKey,
    public_key: PublicKey,
    #[allow(dead_code)]
    path: DerivationPath,
}

impl DerivedKey {
    fn new(secret_key: SecretKey, path: DerivationPath) -> Self {
        let public_key = secret_key.public_key();
        Self {
            secret_key,
            public_key,
            path,
        }
    }
}

/// Wallet state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalletState {
    /// Wallet is locked
    Locked,
    /// Wallet is unlocked
    Unlocked,
}

/// Main wallet structure
pub struct Wallet {
    /// Wallet configuration
    config: WalletConfig,
    /// Current state
    state: WalletState,
    /// Master seed (only available when unlocked)
    master_seed: Option<SeedBytes>,
    /// Derived keys by path string
    derived_keys: HashMap<String, DerivedKey>,
    /// Address manager
    addresses: AddressManager,
    /// Balance tracker
    balances: BalanceTracker,
    /// Transaction history
    history: TransactionHistory,
    /// Nonce tracker per address
    nonces: HashMap<String, u64>,
}

impl Wallet {
    /// Create a new wallet from a mnemonic
    pub fn from_mnemonic(mnemonic: &Mnemonic, passphrase: &str, config: WalletConfig) -> Self {
        let seed = mnemonic.to_seed(passphrase);
        let mut wallet = Self {
            config,
            state: WalletState::Unlocked,
            master_seed: Some(seed),
            derived_keys: HashMap::new(),
            addresses: AddressManager::new(),
            balances: BalanceTracker::new(),
            history: TransactionHistory::new(),
            nonces: HashMap::new(),
        };
        
        // Pre-generate addresses for enabled chains
        if wallet.config.auto_generate_addresses {
            let chains: Vec<_> = wallet.config.chains.iter()
                .filter(|c| c.enabled)
                .map(|c| (c.chain, wallet.config.address_lookahead))
                .collect();
            
            for (chain, lookahead) in chains {
                for i in 0..lookahead {
                    if let Err(e) = wallet.generate_address(chain, i) {
                        // Log warning but continue - address generation failure shouldn't
                        // prevent wallet creation
                        #[cfg(debug_assertions)]
                        eprintln!("Warning: failed to generate address for chain {:?} at index {}: {}", chain, i, e);
                        let _ = e; // Suppress unused warning in release
                    }
                }
            }
        }
        
        wallet
    }

    /// Create a new wallet with a fresh mnemonic
    pub fn create_new(config: WalletConfig) -> (Self, Mnemonic) {
        let mnemonic = Mnemonic::new();
        let wallet = Self::from_mnemonic(&mnemonic, "", config);
        (wallet, mnemonic)
    }

    /// Get wallet configuration
    pub fn config(&self) -> &WalletConfig {
        &self.config
    }

    /// Get wallet state
    pub fn state(&self) -> WalletState {
        self.state
    }

    /// Check if wallet is unlocked
    pub fn is_unlocked(&self) -> bool {
        self.state == WalletState::Unlocked
    }

    /// Lock the wallet
    pub fn lock(&mut self) {
        self.master_seed = None;
        self.derived_keys.clear();
        self.state = WalletState::Locked;
    }

    /// Unlock the wallet with mnemonic and passphrase
    pub fn unlock(&mut self, mnemonic: &Mnemonic, passphrase: &str) -> Result<()> {
        let seed = mnemonic.to_seed(passphrase);
        self.master_seed = Some(seed);
        self.state = WalletState::Unlocked;
        
        // Re-derive keys for existing addresses
        // Collect address info first to avoid borrow issues
        let address_info: Vec<_> = self.addresses.all_addresses()
            .iter()
            .map(|a| (a.chain(), a.index()))
            .collect();
        
        for (chain, index) in address_info {
            let path = DerivationPath::for_chain(chain, index);
            self.derive_key(&path)?;
        }
        
        Ok(())
    }

    /// Derive a key at a specific path
    /// 
    /// Note: This uses a simplified key derivation scheme for the initial implementation.
    /// For full BIP32 compatibility with external wallets, implement proper HMAC-SHA512
    /// based hierarchical deterministic key derivation. The current implementation
    /// provides deterministic key generation that is secure but may not be compatible
    /// with other BIP32-compliant wallets.
    fn derive_key(&mut self, path: &DerivationPath) -> Result<&DerivedKey> {
        let path_str = path.to_string();
        
        if self.derived_keys.contains_key(&path_str) {
            return Ok(&self.derived_keys[&path_str]);
        }
        
        let seed = self.master_seed.as_ref().ok_or(Error::WalletLocked)?;
        
        // Simplified key derivation using HMAC-like construction
        // For full BIP32 compatibility, use a proper BIP32 library
        let mut derivation_data = Vec::new();
        derivation_data.extend_from_slice(seed.as_bytes());
        derivation_data.extend_from_slice(path_str.as_bytes());
        
        let derived_hash = Hash256::hash(&derivation_data);
        let secret_key = SecretKey::from_bytes(derived_hash.as_bytes())?;
        
        let derived_key = DerivedKey::new(secret_key, path.clone());
        self.derived_keys.insert(path_str.clone(), derived_key);
        
        Ok(&self.derived_keys[&path_str])
    }

    /// Generate a new address for a chain
    pub fn generate_address(&mut self, chain: Chain, index: u32) -> Result<Address> {
        let path = DerivationPath::for_chain(chain, index);
        let key = self.derive_key(&path)?;
        let public_key = &key.public_key;
        
        let address = match chain {
            Chain::BitCell => Address::from_public_key_bitcell(public_key, index),
            Chain::Bitcoin => Address::from_public_key_bitcoin(public_key, false, index),
            Chain::BitcoinTestnet => Address::from_public_key_bitcoin(public_key, true, index),
            Chain::Ethereum => Address::from_public_key_ethereum(public_key, false, index),
            Chain::EthereumSepolia => Address::from_public_key_ethereum(public_key, true, index),
            Chain::Custom(_) => Address::from_public_key_bitcell(public_key, index),
        };
        
        self.addresses.add_address(address.clone());
        
        Ok(address)
    }

    /// Get the next address for a chain
    pub fn next_address(&mut self, chain: Chain) -> Result<Address> {
        let index = self.addresses.next_index(chain);
        self.generate_address(chain, index)
    }

    /// Get all addresses for a chain
    pub fn get_addresses(&self, chain: Chain) -> Vec<&Address> {
        self.addresses.get_addresses(chain)
    }

    /// Get all addresses
    pub fn all_addresses(&self) -> &[Address] {
        self.addresses.all_addresses()
    }

    /// Get balance for an address
    pub fn get_balance(&self, address: &Address) -> Balance {
        self.balances.get_balance(address)
    }

    /// Get total balance for a chain
    pub fn get_total_balance(&self, chain: Chain) -> Balance {
        self.balances.get_total(chain)
    }

    /// Update balance for an address
    pub fn update_balance(&mut self, address: &Address, amount: u64) {
        self.balances.update_balance(address, amount);
    }

    /// Get nonce for an address
    pub fn get_nonce(&self, address: &Address) -> u64 {
        let key = address.to_string_formatted();
        *self.nonces.get(&key).unwrap_or(&0)
    }

    /// Increment nonce for an address
    fn increment_nonce(&mut self, address: &Address) {
        let key = address.to_string_formatted();
        let nonce = self.nonces.entry(key).or_insert(0);
        *nonce += 1;
    }

    /// Create a transaction
    pub fn create_transaction(
        &self,
        from: &Address,
        to: &Address,
        amount: u64,
        fee: u64,
    ) -> Result<Transaction> {
        // Verify sufficient balance
        let balance = self.get_balance(from);
        let total_cost = amount.saturating_add(fee);
        if !balance.is_sufficient(total_cost) {
            return Err(Error::InsufficientBalance {
                have: balance.amount(),
                need: total_cost,
            });
        }
        
        let nonce = self.get_nonce(from);
        
        TransactionBuilder::new(from.chain())
            .from(from)
            .to(to)
            .amount(amount)
            .fee(fee)
            .nonce(nonce)
            .build()
    }

    /// Sign a transaction
    pub fn sign_transaction(&mut self, tx: Transaction, from: &Address) -> Result<SignedTransaction> {
        if !self.is_unlocked() {
            return Err(Error::WalletLocked);
        }
        
        let path = DerivationPath::for_chain(from.chain(), from.index());
        let key = self.derive_key(&path)?;
        
        let signed = tx.sign(&key.secret_key);
        
        // Update nonce
        self.increment_nonce(from);
        
        Ok(signed)
    }

    /// Create and sign a transaction in one step
    pub fn send(
        &mut self,
        from: &Address,
        to: &Address,
        amount: u64,
        fee: u64,
    ) -> Result<SignedTransaction> {
        let tx = self.create_transaction(from, to, amount, fee)?;
        self.sign_transaction(tx, from)
    }

    /// Get transaction history
    pub fn history(&self) -> &TransactionHistory {
        &self.history
    }

    /// Get mutable transaction history
    pub fn history_mut(&mut self) -> &mut TransactionHistory {
        &mut self.history
    }

    /// Export wallet data (excluding keys)
    pub fn export_data(&self) -> WalletExport {
        WalletExport {
            config: self.config.clone(),
            addresses: self.addresses.clone(),
            balances: self.balances.clone(),
            history: self.history.clone(),
            nonces: self.nonces.clone(),
        }
    }

    /// Import wallet data
    pub fn import_data(&mut self, data: WalletExport) {
        self.config = data.config;
        self.addresses = data.addresses;
        self.balances = data.balances;
        self.history = data.history;
        self.nonces = data.nonces;
    }
}

impl Drop for Wallet {
    fn drop(&mut self) {
        // Securely clear sensitive data
        self.master_seed = None;
        self.derived_keys.clear();
    }
}

/// Exportable wallet data
/// 
/// Contains wallet configuration and state that can be safely exported
/// and imported. Does not contain sensitive key material.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletExport {
    config: WalletConfig,
    addresses: AddressManager,
    balances: BalanceTracker,
    history: TransactionHistory,
    nonces: HashMap<String, u64>,
}

impl WalletExport {
    /// Get the wallet configuration
    pub fn config(&self) -> &WalletConfig {
        &self.config
    }
    
    /// Get the address manager
    pub fn addresses(&self) -> &AddressManager {
        &self.addresses
    }
    
    /// Get the balance tracker
    pub fn balances(&self) -> &BalanceTracker {
        &self.balances
    }
    
    /// Get the transaction history
    pub fn history(&self) -> &TransactionHistory {
        &self.history
    }
    
    /// Get the nonces map
    pub fn nonces(&self) -> &HashMap<String, u64> {
        &self.nonces
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_wallet() -> Wallet {
        let mnemonic = Mnemonic::new();
        Wallet::from_mnemonic(&mnemonic, "", WalletConfig::default())
    }

    #[test]
    fn test_wallet_creation() {
        let (wallet, _mnemonic) = Wallet::create_new(WalletConfig::default());
        assert!(wallet.is_unlocked());
    }

    #[test]
    fn test_wallet_from_mnemonic() {
        let mnemonic = Mnemonic::new();
        let wallet = Wallet::from_mnemonic(&mnemonic, "", WalletConfig::default());
        assert!(wallet.is_unlocked());
    }

    #[test]
    fn test_address_generation() {
        let mut wallet = test_wallet();
        
        let addr1 = wallet.generate_address(Chain::BitCell, 0).unwrap();
        let addr2 = wallet.generate_address(Chain::BitCell, 1).unwrap();
        
        assert_ne!(addr1.as_bytes(), addr2.as_bytes());
    }

    #[test]
    fn test_address_deterministic() {
        let mnemonic = Mnemonic::new();
        
        let mut wallet1 = Wallet::from_mnemonic(&mnemonic, "", WalletConfig::default());
        let addr1 = wallet1.generate_address(Chain::BitCell, 10).unwrap();
        
        let mut wallet2 = Wallet::from_mnemonic(&mnemonic, "", WalletConfig::default());
        let addr2 = wallet2.generate_address(Chain::BitCell, 10).unwrap();
        
        assert_eq!(addr1.as_bytes(), addr2.as_bytes());
    }

    #[test]
    fn test_next_address() {
        let mut wallet = test_wallet();
        
        // Default config generates 5 addresses per chain
        let next = wallet.next_address(Chain::BitCell).unwrap();
        assert_eq!(next.index(), 5);
    }

    #[test]
    fn test_wallet_lock_unlock() {
        let mnemonic = Mnemonic::new();
        let mut wallet = Wallet::from_mnemonic(&mnemonic, "", WalletConfig::default());
        
        assert!(wallet.is_unlocked());
        
        wallet.lock();
        assert!(!wallet.is_unlocked());
        assert_eq!(wallet.state(), WalletState::Locked);
        
        wallet.unlock(&mnemonic, "").unwrap();
        assert!(wallet.is_unlocked());
    }

    #[test]
    fn test_create_transaction() {
        let mut wallet = test_wallet();
        
        let from = wallet.next_address(Chain::BitCell).unwrap();
        let to = wallet.next_address(Chain::BitCell).unwrap();
        
        // Set balance
        wallet.update_balance(&from, 1_000_000);
        
        let tx = wallet.create_transaction(&from, &to, 100_000, 100).unwrap();
        
        assert_eq!(tx.amount, 100_000);
        assert_eq!(tx.fee, 100);
    }

    #[test]
    fn test_insufficient_balance() {
        let mut wallet = test_wallet();
        
        let from = wallet.next_address(Chain::BitCell).unwrap();
        let to = wallet.next_address(Chain::BitCell).unwrap();
        
        // No balance set
        let result = wallet.create_transaction(&from, &to, 100_000, 100);
        
        assert!(matches!(result, Err(Error::InsufficientBalance { .. })));
    }

    #[test]
    fn test_sign_transaction() {
        let mut wallet = test_wallet();
        
        let from = wallet.next_address(Chain::BitCell).unwrap();
        let to = wallet.next_address(Chain::BitCell).unwrap();
        
        wallet.update_balance(&from, 1_000_000);
        
        let tx = wallet.create_transaction(&from, &to, 100_000, 100).unwrap();
        let signed = wallet.sign_transaction(tx, &from).unwrap();
        
        assert!(!signed.hash_hex().is_empty());
    }

    #[test]
    fn test_send() {
        let mut wallet = test_wallet();
        
        let from = wallet.next_address(Chain::BitCell).unwrap();
        let to = wallet.next_address(Chain::BitCell).unwrap();
        
        wallet.update_balance(&from, 1_000_000);
        
        let signed = wallet.send(&from, &to, 100_000, 100).unwrap();
        
        assert!(!signed.hash_hex().is_empty());
    }

    #[test]
    fn test_nonce_increment() {
        let mut wallet = test_wallet();
        
        let from = wallet.next_address(Chain::BitCell).unwrap();
        let to = wallet.next_address(Chain::BitCell).unwrap();
        
        wallet.update_balance(&from, 10_000_000);
        
        assert_eq!(wallet.get_nonce(&from), 0);
        
        wallet.send(&from, &to, 100_000, 100).unwrap();
        assert_eq!(wallet.get_nonce(&from), 1);
        
        wallet.send(&from, &to, 100_000, 100).unwrap();
        assert_eq!(wallet.get_nonce(&from), 2);
    }

    #[test]
    fn test_multi_chain_addresses() {
        let mut wallet = test_wallet();
        
        let btc_addr = wallet.next_address(Chain::Bitcoin).unwrap();
        let eth_addr = wallet.next_address(Chain::Ethereum).unwrap();
        let cell_addr = wallet.next_address(Chain::BitCell).unwrap();
        
        assert_eq!(btc_addr.chain(), Chain::Bitcoin);
        assert_eq!(eth_addr.chain(), Chain::Ethereum);
        assert_eq!(cell_addr.chain(), Chain::BitCell);
    }

    #[test]
    fn test_balance_tracking() {
        let mut wallet = test_wallet();
        
        let addr = wallet.next_address(Chain::BitCell).unwrap();
        
        wallet.update_balance(&addr, 500_000);
        
        let balance = wallet.get_balance(&addr);
        assert_eq!(balance.amount(), 500_000);
        
        let total = wallet.get_total_balance(Chain::BitCell);
        assert!(total.amount() >= 500_000);
    }

    #[test]
    fn test_export_import() {
        let mut wallet = test_wallet();
        let addr = wallet.next_address(Chain::BitCell).unwrap();
        wallet.update_balance(&addr, 1_000_000);
        
        let export = wallet.export_data();
        
        // Create new wallet and import
        let mnemonic = Mnemonic::new();
        let mut new_wallet = Wallet::from_mnemonic(&mnemonic, "", WalletConfig::default());
        new_wallet.import_data(export);
        
        // Check that data was imported
        assert_eq!(new_wallet.config.name, "Default Wallet");
    }

    #[test]
    fn test_derivation_path() {
        let path = DerivationPath::bip44(0, 0, 0, 5);
        assert_eq!(path.to_string(), "m/44'/0'/0'/0'/5");
        
        let chain_path = DerivationPath::for_chain(Chain::BitCell, 3);
        assert!(chain_path.to_string().contains("9999")); // BitCell coin type
    }

    #[test]
    fn test_locked_wallet_operations() {
        let mut wallet = test_wallet();
        let addr = wallet.next_address(Chain::BitCell).unwrap();
        wallet.update_balance(&addr, 1_000_000);
        
        wallet.lock();
        
        // Balance queries should still work
        let balance = wallet.get_balance(&addr);
        assert_eq!(balance.amount(), 1_000_000);
        
        // But creating transactions should work (doesn't need signing)
        let to = Address::from_public_key_bitcell(&SecretKey::generate().public_key(), 0);
        let tx = wallet.create_transaction(&addr, &to, 100, 10);
        assert!(tx.is_ok());
        
        // Signing should fail
        let tx = tx.unwrap();
        let result = wallet.sign_transaction(tx, &addr);
        assert!(matches!(result, Err(Error::WalletLocked)));
    }
}
