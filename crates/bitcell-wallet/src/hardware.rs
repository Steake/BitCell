//! Hardware Wallet Support
//!
//! This module provides an abstraction layer for hardware wallet integration,
//! supporting Ledger, Trezor, and other hardware security devices.
//!
//! # Security
//! Hardware wallets keep private keys secure by:
//! - Never exposing private keys to the host computer
//! - Requiring physical confirmation for transactions
//! - Using secure elements for cryptographic operations
//!
//! # Usage
//! ```ignore
//! use bitcell_wallet::hardware::{HardwareWallet, HardwareWalletType};
//!
//! // Connect to a Ledger device
//! let hw = HardwareWallet::connect(HardwareWalletType::Ledger)?;
//!
//! // Get public key for derivation path
//! let pubkey = hw.get_public_key("m/44'/0'/0'/0/0")?;
//!
//! // Sign a transaction
//! let signature = hw.sign_transaction(&transaction)?;
//! ```

use crate::{Chain, Error, Result, Transaction, SignedTransaction};
use bitcell_crypto::{Hash256, PublicKey, Signature};
use std::sync::Arc;

/// Type of hardware wallet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareWalletType {
    /// Ledger Nano S/X
    Ledger,
    /// Trezor One/Model T
    Trezor,
    /// Generic hardware signer (HSM, etc.)
    Generic,
    /// Mock device for testing
    #[cfg(test)]
    Mock,
}

/// Hardware wallet connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// Device is connected and ready
    Connected,
    /// Device is not connected
    Disconnected,
    /// Device is connected but locked (needs PIN)
    Locked,
    /// Device is busy (processing another operation)
    Busy,
}

/// Hardware wallet trait for device abstraction
pub trait HardwareWalletDevice: Send + Sync {
    /// Get the device type
    fn device_type(&self) -> HardwareWalletType;
    
    /// Check connection status
    fn status(&self) -> ConnectionStatus;
    
    /// Get public key for a derivation path
    fn get_public_key(&self, derivation_path: &str) -> Result<PublicKey>;
    
    /// Get address for a derivation path  
    fn get_address(&self, derivation_path: &str, chain: Chain) -> Result<String>;
    
    /// Sign a message hash
    fn sign_hash(&self, derivation_path: &str, hash: &Hash256) -> Result<Signature>;
    
    /// Sign a transaction (requires user confirmation on device)
    fn sign_transaction(&self, derivation_path: &str, tx: &Transaction) -> Result<Signature>;
}

/// Hardware wallet manager
#[derive(Clone)]
pub struct HardwareWallet {
    device: Arc<dyn HardwareWalletDevice>,
    derivation_path: String,
}

impl HardwareWallet {
    /// Connect to a hardware wallet
    pub fn connect(wallet_type: HardwareWalletType) -> Result<Self> {
        let device: Arc<dyn HardwareWalletDevice> = match wallet_type {
            HardwareWalletType::Ledger => {
                #[cfg(feature = "ledger")]
                {
                    return Ok(Self {
                        device: Arc::new(LedgerDevice::connect()?),
                        derivation_path: "m/44'/60'/0'/0/0".to_string(),
                    });
                }
                #[cfg(not(feature = "ledger"))]
                {
                    return Err(Error::HardwareWallet("Ledger support not compiled in. Enable the 'ledger' feature.".into()));
                }
            }
            HardwareWalletType::Trezor => {
                #[cfg(feature = "trezor")]
                {
                    return Ok(Self {
                        device: Arc::new(TrezorDevice::connect()?),
                        derivation_path: "m/44'/60'/0'/0/0".to_string(),
                    });
                }
                #[cfg(not(feature = "trezor"))]
                {
                    return Err(Error::HardwareWallet("Trezor support not compiled in. Enable the 'trezor' feature.".into()));
                }
            }
            HardwareWalletType::Generic => {
                return Err(Error::HardwareWallet("Generic hardware wallet is not yet implemented".into()));
            }
            #[cfg(test)]
            HardwareWalletType::Mock => {
                Arc::new(MockHardwareWallet::new())
            }
        };
        
        Ok(Self {
            device,
            derivation_path: "m/44'/60'/0'/0/0".to_string(), // Default ETH-like path
        })
    }
    
    /// Set the derivation path
    pub fn with_derivation_path(mut self, path: &str) -> Self {
        self.derivation_path = path.to_string();
        self
    }
    
    /// Get derivation path for a specific chain
    pub fn derivation_path_for_chain(chain: Chain, account: u32, index: u32) -> String {
        let coin_type = match chain {
            Chain::BitCell => 9999, // Custom coin type for BitCell
            Chain::Bitcoin => 0,
            Chain::BitcoinTestnet => 1,
            Chain::Ethereum | Chain::EthereumSepolia => 60,
            Chain::Custom(id) => id,
        };
        
        format!("m/44'/{}'/{}'/{}/{}", coin_type, account, 0, index)
    }
    
    /// Check if device is connected
    pub fn is_connected(&self) -> bool {
        self.device.status() == ConnectionStatus::Connected
    }
    
    /// Get the device status
    pub fn status(&self) -> ConnectionStatus {
        self.device.status()
    }
    
    /// Get the device type
    pub fn device_type(&self) -> HardwareWalletType {
        self.device.device_type()
    }
    
    /// Get public key for current derivation path
    pub fn get_public_key(&self) -> Result<PublicKey> {
        self.device.get_public_key(&self.derivation_path)
    }
    
    /// Get address for current derivation path and chain
    pub fn get_address(&self, chain: Chain) -> Result<String> {
        self.device.get_address(&self.derivation_path, chain)
    }
    
    /// Sign a transaction
    /// 
    /// This will prompt the user for confirmation on the hardware device.
    pub fn sign_transaction(&self, tx: &Transaction) -> Result<SignedTransaction> {
        // Sign the transaction hash
        let hash = tx.hash();
        let signature = self.device.sign_transaction(&self.derivation_path, tx)?;
        
        Ok(SignedTransaction {
            transaction: tx.clone(),
            signature,
            tx_hash: hash,
        })
    }
    
    /// Sign a message hash directly
    pub fn sign_hash(&self, hash: &Hash256) -> Result<Signature> {
        self.device.sign_hash(&self.derivation_path, hash)
    }
}

/// Signing method abstraction
/// 
/// Allows code to work with both software and hardware signing
pub enum SigningMethod {
    /// Software signing with in-memory private key
    Software(bitcell_crypto::SecretKey),
    /// Hardware wallet signing
    Hardware(HardwareWallet),
}

impl SigningMethod {
    /// Sign a transaction using the configured method
    pub fn sign(&self, tx: &Transaction) -> Result<SignedTransaction> {
        match self {
            SigningMethod::Software(sk) => Ok(tx.sign(sk)),
            SigningMethod::Hardware(hw) => hw.sign_transaction(tx),
        }
    }
    
    /// Get the public key
    pub fn public_key(&self) -> Result<PublicKey> {
        match self {
            SigningMethod::Software(sk) => Ok(sk.public_key()),
            SigningMethod::Hardware(hw) => hw.get_public_key(),
        }
    }
    
    /// Check if this is a hardware signing method
    pub fn is_hardware(&self) -> bool {
        matches!(self, SigningMethod::Hardware(_))
    }
}

/// Mock hardware wallet for testing
#[cfg(test)]
pub struct MockHardwareWallet {
    secret_key: bitcell_crypto::SecretKey,
    connected: bool,
}

#[cfg(test)]
impl MockHardwareWallet {
    pub fn new() -> Self {
        Self {
            secret_key: bitcell_crypto::SecretKey::generate(),
            connected: true,
        }
    }
}

#[cfg(test)]
impl HardwareWalletDevice for MockHardwareWallet {
    fn device_type(&self) -> HardwareWalletType {
        HardwareWalletType::Mock
    }
    
    fn status(&self) -> ConnectionStatus {
        if self.connected {
            ConnectionStatus::Connected
        } else {
            ConnectionStatus::Disconnected
        }
    }
    
    fn get_public_key(&self, _derivation_path: &str) -> Result<PublicKey> {
        Ok(self.secret_key.public_key())
    }
    
    fn get_address(&self, derivation_path: &str, chain: Chain) -> Result<String> {
        let pk = self.get_public_key(derivation_path)?;
        // Simple address derivation for testing
        let hash = Hash256::hash(pk.as_bytes());
        let prefix = match chain {
            Chain::BitCell => "BC1",
            Chain::Bitcoin | Chain::BitcoinTestnet => "bc1",
            Chain::Ethereum | Chain::EthereumSepolia => "0x",
            Chain::Custom(_) => "CUST",
        };
        Ok(format!("{}{}", prefix, hex::encode(&hash.as_bytes()[..20])))
    }
    
    fn sign_hash(&self, _derivation_path: &str, hash: &Hash256) -> Result<Signature> {
        Ok(self.secret_key.sign(hash.as_bytes()))
    }
    
    fn sign_transaction(&self, derivation_path: &str, tx: &Transaction) -> Result<Signature> {
        self.sign_hash(derivation_path, &tx.hash())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Chain;
    
    #[test]
    fn test_mock_hardware_wallet() {
        let hw = HardwareWallet::connect(HardwareWalletType::Mock).unwrap();
        
        assert!(hw.is_connected());
        assert_eq!(hw.device_type(), HardwareWalletType::Mock);
    }
    
    #[test]
    fn test_hardware_wallet_get_public_key() {
        let hw = HardwareWallet::connect(HardwareWalletType::Mock).unwrap();
        let pk = hw.get_public_key().unwrap();
        
        assert_eq!(pk.as_bytes().len(), 33); // Compressed public key
    }
    
    #[test]
    fn test_hardware_wallet_get_address() {
        let hw = HardwareWallet::connect(HardwareWalletType::Mock).unwrap();
        let address = hw.get_address(Chain::BitCell).unwrap();
        
        assert!(address.starts_with("BC1"));
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
        
        // Verify signature
        let pk = hw.get_public_key().unwrap();
        assert!(signed.verify(&pk).is_ok());
    }
    
    #[test]
    fn test_signing_method_software() {
        let sk = bitcell_crypto::SecretKey::generate();
        let pk = sk.public_key();
        let method = SigningMethod::Software(sk);
        
        let tx = Transaction::new(
            Chain::BitCell,
            "BC1sender".to_string(),
            "BC1recipient".to_string(),
            1000,
            10,
            0,
        );
        
        let signed = method.sign(&tx).unwrap();
        
        assert!(signed.verify(&pk).is_ok());
        assert!(!method.is_hardware());
    }
    
    #[test]
    fn test_signing_method_hardware() {
        let hw = HardwareWallet::connect(HardwareWalletType::Mock).unwrap();
        let method = SigningMethod::Hardware(hw);
        
        let tx = Transaction::new(
            Chain::BitCell,
            "BC1sender".to_string(),
            "BC1recipient".to_string(),
            1000,
            10,
            0,
        );
        
        let signed = method.sign(&tx).unwrap();
        let pk = method.public_key().unwrap();
        
        assert!(signed.verify(&pk).is_ok());
        assert!(method.is_hardware());
    }
    
    #[test]
    fn test_derivation_path_for_chain() {
        assert_eq!(
            HardwareWallet::derivation_path_for_chain(Chain::Bitcoin, 0, 0),
            "m/44'/0'/0'/0/0"
        );
        assert_eq!(
            HardwareWallet::derivation_path_for_chain(Chain::Ethereum, 0, 5),
            "m/44'/60'/0'/0/5"
        );
        assert_eq!(
            HardwareWallet::derivation_path_for_chain(Chain::BitCell, 1, 3),
            "m/44'/9999'/1'/0/3"
        );
    }
}
