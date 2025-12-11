//! Trezor hardware wallet integration
//!
//! This module provides support for Trezor Model One and Model T devices.
//! 
//! # Device Requirements
//! - Trezor device (Model One or Model T)
//! - USB connection via HID
//! - Device unlocked (PIN entered)
//!
//! # Implementation Note
//! This is a custom implementation using USB HID protocol.
//! For production, consider using Trezor Connect or official libraries.
//!
//! # Security
//! - All signing operations require physical confirmation on device
//! - Private keys never leave the device
//! - Derivation paths and transaction details shown on device screen
//! - Supports passphrase for additional security

use crate::{Chain, Error, Result, Transaction};
use bitcell_crypto::{Hash256, PublicKey, Signature};
use super::{ConnectionStatus, HardwareWalletDevice, HardwareWalletType};

#[cfg(feature = "trezor")]
use hidapi::{HidApi, HidDevice};

/// Trezor USB vendor and product IDs
#[cfg(feature = "trezor")]
const TREZOR_VENDOR_ID: u16 = 0x534c; // SatoshiLabs
#[cfg(feature = "trezor")]
const TREZOR_ONE_PRODUCT_ID: u16 = 0x0001;
#[cfg(feature = "trezor")]
const TREZOR_T_PRODUCT_ID: u16 = 0x0002;

/// Trezor device implementation
pub struct TrezorDevice {
    #[cfg(feature = "trezor")]
    device: HidDevice,
    connected: bool,
    passphrase: Option<String>,
}

impl TrezorDevice {
    /// Connect to a Trezor device
    pub fn connect() -> Result<Self> {
        #[cfg(feature = "trezor")]
        {
            let api = HidApi::new()
                .map_err(|e| Error::HardwareWallet(format!("Failed to initialize HID API: {}", e)))?;
            
            // Try to connect to Trezor One first
            let device = api.open(TREZOR_VENDOR_ID, TREZOR_ONE_PRODUCT_ID)
                .or_else(|_| {
                    // If not found, try Trezor Model T
                    api.open(TREZOR_VENDOR_ID, TREZOR_T_PRODUCT_ID)
                })
                .map_err(|e| Error::HardwareWallet(format!(
                    "Failed to connect to Trezor device: {}. Is the device connected and unlocked?", e
                )))?;
            
            Ok(Self {
                device,
                connected: true,
                passphrase: None,
            })
        }
        
        #[cfg(not(feature = "trezor"))]
        {
            Err(Error::HardwareWallet(
                "Trezor support not compiled in. Enable the 'trezor' feature.".into()
            ))
        }
    }
    
    /// Set passphrase for additional security
    /// 
    /// The passphrase provides an additional layer of security by deriving
    /// different keys based on the passphrase. This means the same device
    /// with different passphrases will generate different keys.
    pub fn with_passphrase(mut self, passphrase: String) -> Self {
        self.passphrase = Some(passphrase);
        self
    }
    
    /// Parse BIP44 derivation path into address_n array
    fn parse_path(path: &str) -> Result<Vec<u32>> {
        // Parse "m/44'/9999'/0'/0/0" format
        let parts: Vec<&str> = path.trim_start_matches("m/").split('/').collect();
        let mut address_n = Vec::with_capacity(parts.len());
        
        for part in parts {
            let hardened = part.ends_with('\'');
            let num_str = part.trim_end_matches('\'');
            let mut num: u32 = num_str.parse()
                .map_err(|_| Error::InvalidDerivationPath(format!("Invalid number in path: {}", num_str)))?;
            
            if hardened {
                num |= 0x8000_0000;
            }
            
            address_n.push(num);
        }
        
        Ok(address_n)
    }
    
    /// Send a command to the device
    #[cfg(feature = "trezor")]
    #[allow(dead_code)] // Reserved for future full protocol implementation
    fn send_command(&self, _command: &[u8]) -> Result<Vec<u8>> {
        // This is a simplified placeholder
        // Real implementation would use Trezor's protobuf protocol
        Err(Error::HardwareWallet(
            "Trezor protocol implementation is a placeholder. Use mock device for testing.".into()
        ))
    }
    
    /// Get public key from device at derivation path
    #[cfg(feature = "trezor")]
    fn get_pubkey_from_device(&self, path: &str) -> Result<Vec<u8>> {
        #[allow(unused_variables)] // Path validation for future implementation
        let address_n = Self::parse_path(path)?;
        
        // Real implementation would:
        // 1. Construct GetPublicKey protobuf message
        // 2. Send via USB HID
        // 3. Parse PublicKey response
        
        Err(Error::HardwareWallet(
            "Trezor GetPublicKey not fully implemented. Use mock device for testing.".into()
        ))
    }
    
    /// Sign a message with the device
    #[cfg(feature = "trezor")]
    fn sign_message_with_device(&self, path: &str, _message: &[u8]) -> Result<Vec<u8>> {
        #[allow(unused_variables)] // Path validation for future implementation
        let address_n = Self::parse_path(path)?;
        
        // Real implementation would:
        // 1. Construct SignMessage protobuf message
        // 2. Handle passphrase if set
        // 3. Send via USB HID
        // 4. Wait for user confirmation on device
        // 5. Parse MessageSignature response
        
        Err(Error::HardwareWallet(
            "Trezor SignMessage not fully implemented. Use mock device for testing.".into()
        ))
    }
}

impl HardwareWalletDevice for TrezorDevice {
    fn device_type(&self) -> HardwareWalletType {
        HardwareWalletType::Trezor
    }
    
    fn status(&self) -> ConnectionStatus {
        if self.connected {
            ConnectionStatus::Connected
        } else {
            ConnectionStatus::Disconnected
        }
    }
    
    fn get_public_key(&self, derivation_path: &str) -> Result<PublicKey> {
        #[cfg(feature = "trezor")]
        {
            let pubkey_bytes = self.get_pubkey_from_device(derivation_path)?;
            PublicKey::from_bytes(&pubkey_bytes)
                .map_err(|e| Error::Crypto(format!("Invalid public key from device: {}", e)))
        }
        
        #[cfg(not(feature = "trezor"))]
        {
            let _ = derivation_path;
            Err(Error::HardwareWallet("Trezor support not compiled in".into()))
        }
    }
    
    fn get_address(&self, derivation_path: &str, chain: Chain) -> Result<String> {
        let pubkey = self.get_public_key(derivation_path)?;
        
        // Derive address from public key based on chain
        let hash = Hash256::hash(pubkey.as_bytes());
        let prefix = match chain {
            Chain::BitCell => "BC1",
            Chain::Bitcoin | Chain::BitcoinTestnet => "bc1",
            Chain::Ethereum | Chain::EthereumSepolia => "0x",
            Chain::Custom(_) => "CUST",
        };
        
        Ok(format!("{}{}", prefix, hex::encode(&hash.as_bytes()[..20])))
    }
    
    fn sign_hash(&self, derivation_path: &str, hash: &Hash256) -> Result<Signature> {
        #[cfg(feature = "trezor")]
        {
            let sig_bytes = self.sign_message_with_device(derivation_path, hash.as_bytes())?;
            Signature::from_bytes(&sig_bytes)
                .map_err(|e| Error::Crypto(format!("Invalid signature from device: {}", e)))
        }
        
        #[cfg(not(feature = "trezor"))]
        {
            let _ = (derivation_path, hash);
            Err(Error::HardwareWallet("Trezor support not compiled in".into()))
        }
    }
    
    fn sign_transaction(&self, derivation_path: &str, tx: &Transaction) -> Result<Signature> {
        // For transactions, we sign the transaction hash
        let hash = tx.hash();
        self.sign_hash(derivation_path, &hash)
    }
}

#[cfg(all(test, feature = "trezor"))]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_path() {
        // Test normal path
        let path = "m/44'/9999'/0'/0/0";
        let result = TrezorDevice::parse_path(path).unwrap();
        
        assert_eq!(result.len(), 5);
        assert_eq!(result[0], 44 | 0x8000_0000); // 44' (hardened)
        assert_eq!(result[1], 9999 | 0x8000_0000); // 9999' (hardened)
        assert_eq!(result[2], 0 | 0x8000_0000); // 0' (hardened)
        assert_eq!(result[3], 0); // 0 (not hardened)
        assert_eq!(result[4], 0); // 0 (not hardened)
    }
    
    #[test]
    fn test_parse_path_ethereum() {
        let path = "m/44'/60'/0'/0/5";
        let result = TrezorDevice::parse_path(path).unwrap();
        
        assert_eq!(result.len(), 5);
        assert_eq!(result[4], 5); // Last index is 5
    }
    
    #[test]
    fn test_invalid_path() {
        let path = "m/invalid/path";
        assert!(TrezorDevice::parse_path(path).is_err());
    }
}

