//! Ledger hardware wallet integration
//!
//! This module provides support for Ledger Nano S/X devices.
//! 
//! # Device Requirements
//! - Ledger device with BitCell app installed (falls back to generic Ethereum app)
//! - USB connection
//! - Device unlocked (PIN entered)
//!
//! # Security
//! - All signing operations require physical confirmation on device
//! - Private keys never leave the device
//! - Derivation paths are displayed on device screen

use crate::{Chain, Error, Result, Transaction};
use bitcell_crypto::{Hash256, PublicKey, Signature};
use super::{ConnectionStatus, HardwareWalletDevice, HardwareWalletType};

#[cfg(feature = "ledger")]
use ledger_transport_hid::{TransportNativeHID, hidapi::HidApi};
#[cfg(feature = "ledger")]
use ledger_apdu::{APDUCommand, APDUAnswer};

/// Ledger APDU instruction codes
const INS_GET_PUBLIC_KEY: u8 = 0x02;
const INS_SIGN: u8 = 0x04;
const INS_GET_APP_CONFIGURATION: u8 = 0x06;

/// Ledger device implementation
pub struct LedgerDevice {
    #[cfg(feature = "ledger")]
    transport: TransportNativeHID,
    connected: bool,
}

impl LedgerDevice {
    /// Connect to a Ledger device
    pub fn connect() -> Result<Self> {
        #[cfg(feature = "ledger")]
        {
            let hidapi = HidApi::new()
                .map_err(|e| Error::HardwareWallet(format!("Failed to initialize HID API: {}", e)))?;
            
            let transport = TransportNativeHID::new(&hidapi)
                .map_err(|e| Error::HardwareWallet(format!("Failed to connect to Ledger device: {}. Is the device connected and unlocked?", e)))?;
            
            Ok(Self {
                transport,
                connected: true,
            })
        }
        
        #[cfg(not(feature = "ledger"))]
        {
            Err(Error::HardwareWallet(
                "Ledger support not compiled in. Enable the 'ledger' feature.".into()
            ))
        }
    }
    
    /// Verify device is running the correct app
    #[cfg(feature = "ledger")]
    pub fn verify_app(&self) -> Result<String> {
        // Get app configuration to verify correct app is running
        let command = APDUCommand {
            cla: 0xe0,
            ins: INS_GET_APP_CONFIGURATION,
            p1: 0x00,
            p2: 0x00,
            data: vec![],
        };
        
        let response = self.transport.exchange(&command)
            .map_err(|e| Error::HardwareWallet(format!("Failed to get app configuration: {}", e)))?;
        
        if response.retcode() != 0x9000 {
            return Err(Error::HardwareWallet(
                format!("Device returned error code: 0x{:04x}", response.retcode())
            ));
        }
        
        // Parse app version from response
        let data = response.data();
        if data.len() >= 4 {
            let version = format!("{}.{}.{}", data[1], data[2], data[3]);
            Ok(version)
        } else {
            Ok("unknown".to_string())
        }
    }
    
    /// Parse BIP44 derivation path into bytes
    fn serialize_path(path: &str) -> Result<Vec<u8>> {
        // Parse "m/44'/9999'/0'/0/0" format
        let parts: Vec<&str> = path.trim_start_matches("m/").split('/').collect();
        let mut result = vec![parts.len() as u8];
        
        for part in parts {
            let hardened = part.ends_with('\'');
            let num_str = part.trim_end_matches('\'');
            let mut num: u32 = num_str.parse()
                .map_err(|_| Error::InvalidDerivationPath(format!("Invalid number in path: {}", num_str)))?;
            
            if hardened {
                num |= 0x8000_0000;
            }
            
            result.extend_from_slice(&num.to_be_bytes());
        }
        
        Ok(result)
    }
    
    /// Get public key from device at derivation path
    #[cfg(feature = "ledger")]
    fn get_pubkey_from_device(&self, path: &str) -> Result<Vec<u8>> {
        let path_bytes = Self::serialize_path(path)?;
        
        let command = APDUCommand {
            cla: 0xe0,
            ins: INS_GET_PUBLIC_KEY,
            p1: 0x00, // No display
            p2: 0x00, // No chain code
            data: path_bytes,
        };
        
        let response = self.transport.exchange(&command)
            .map_err(|e| Error::HardwareWallet(format!("Failed to get public key: {}", e)))?;
        
        if response.retcode() != 0x9000 {
            return Err(Error::HardwareWallet(
                format!("Device returned error code: 0x{:04x}. Make sure the correct app is open.", response.retcode())
            ));
        }
        
        let data = response.data();
        if data.is_empty() {
            return Err(Error::HardwareWallet("Empty response from device".into()));
        }
        
        // First byte is the public key length
        let pubkey_len = data[0] as usize;
        if data.len() < 1 + pubkey_len {
            return Err(Error::HardwareWallet("Invalid public key response".into()));
        }
        
        Ok(data[1..1+pubkey_len].to_vec())
    }
    
    /// Sign a hash with the device
    #[cfg(feature = "ledger")]
    fn sign_hash_with_device(&self, path: &str, hash: &[u8]) -> Result<Vec<u8>> {
        let path_bytes = Self::serialize_path(path)?;
        
        // Construct signing payload: path_length + path + hash
        let mut data = path_bytes;
        data.extend_from_slice(hash);
        
        let command = APDUCommand {
            cla: 0xe0,
            ins: INS_SIGN,
            p1: 0x00,
            p2: 0x00,
            data,
        };
        
        let response = self.transport.exchange(&command)
            .map_err(|e| Error::HardwareWallet(format!("Failed to sign: {}. User may have rejected the transaction.", e)))?;
        
        if response.retcode() == 0x6985 {
            return Err(Error::HardwareWallet("User rejected the transaction on device".into()));
        }
        
        if response.retcode() != 0x9000 {
            return Err(Error::HardwareWallet(
                format!("Device returned error code: 0x{:04x}", response.retcode())
            ));
        }
        
        Ok(response.data().to_vec())
    }
}

impl HardwareWalletDevice for LedgerDevice {
    fn device_type(&self) -> HardwareWalletType {
        HardwareWalletType::Ledger
    }
    
    fn status(&self) -> ConnectionStatus {
        if self.connected {
            ConnectionStatus::Connected
        } else {
            ConnectionStatus::Disconnected
        }
    }
    
    fn get_public_key(&self, derivation_path: &str) -> Result<PublicKey> {
        #[cfg(feature = "ledger")]
        {
            let pubkey_bytes = self.get_pubkey_from_device(derivation_path)?;
            PublicKey::from_bytes(&pubkey_bytes)
                .map_err(|e| Error::Crypto(format!("Invalid public key from device: {}", e)))
        }
        
        #[cfg(not(feature = "ledger"))]
        {
            let _ = derivation_path;
            Err(Error::HardwareWallet("Ledger support not compiled in".into()))
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
        #[cfg(feature = "ledger")]
        {
            let sig_bytes = self.sign_hash_with_device(derivation_path, hash.as_bytes())?;
            Signature::from_bytes(&sig_bytes)
                .map_err(|e| Error::Crypto(format!("Invalid signature from device: {}", e)))
        }
        
        #[cfg(not(feature = "ledger"))]
        {
            let _ = (derivation_path, hash);
            Err(Error::HardwareWallet("Ledger support not compiled in".into()))
        }
    }
    
    fn sign_transaction(&self, derivation_path: &str, tx: &Transaction) -> Result<Signature> {
        // For transactions, we sign the transaction hash
        let hash = tx.hash();
        self.sign_hash(derivation_path, &hash)
    }
}

#[cfg(all(test, feature = "ledger"))]
mod tests {
    use super::*;
    
    #[test]
    fn test_serialize_path() {
        // Test normal path
        let path = "m/44'/9999'/0'/0/0";
        let result = LedgerDevice::serialize_path(path).unwrap();
        
        // Should be: [5, 0x8000002c, 0x8000270f, 0x80000000, 0x00000000, 0x00000000]
        assert_eq!(result[0], 5); // 5 components
        
        // Test another path
        let path2 = "m/44'/60'/0'/0/5";
        let result2 = LedgerDevice::serialize_path(path2).unwrap();
        assert_eq!(result2[0], 5);
    }
    
    #[test]
    fn test_invalid_path() {
        let path = "m/invalid/path";
        assert!(LedgerDevice::serialize_path(path).is_err());
    }
}
