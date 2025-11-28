//! Address Generation and Management
//!
//! Provides address generation, formatting, and validation for multiple chains.

use crate::{Chain, Error, Result};
use bitcell_crypto::PublicKey;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Address type for different blockchain formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AddressType {
    /// BitCell native address (Base58 encoded)
    BitCell,
    /// Bitcoin P2PKH (Pay to Public Key Hash)
    BitcoinP2PKH,
    /// Bitcoin P2WPKH (Pay to Witness Public Key Hash - SegWit)
    BitcoinP2WPKH,
    /// Ethereum address (hex encoded with checksum)
    Ethereum,
}

/// Blockchain address
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address {
    /// Raw address bytes
    bytes: Vec<u8>,
    /// Address type
    address_type: AddressType,
    /// Associated chain
    chain: Chain,
    /// Derivation index (for HD wallets)
    index: u32,
}

impl Address {
    /// Create a new address from raw bytes
    pub fn new(bytes: Vec<u8>, address_type: AddressType, chain: Chain, index: u32) -> Self {
        Self {
            bytes,
            address_type,
            chain,
            index,
        }
    }

    /// Generate a BitCell address from a public key
    /// 
    /// Uses double SHA256 and takes the first 20 bytes as the address.
    /// This is a simplified scheme for BitCell's native addresses.
    pub fn from_public_key_bitcell(public_key: &PublicKey, index: u32) -> Self {
        let pubkey_bytes = public_key.as_bytes();
        let hash1 = Sha256::digest(pubkey_bytes);
        let hash2 = Sha256::digest(hash1);
        // Take first 20 bytes as address
        let address_bytes = hash2[..20].to_vec();
        
        Self::new(address_bytes, AddressType::BitCell, Chain::BitCell, index)
    }

    /// Generate a Bitcoin P2PKH address from a public key
    /// 
    /// Note: This is a simplified implementation using double SHA256.
    /// For full Bitcoin compatibility, use RIPEMD160(SHA256(pubkey)).
    /// Addresses generated here are for internal use and may not be
    /// compatible with external Bitcoin wallets.
    pub fn from_public_key_bitcoin(public_key: &PublicKey, testnet: bool, index: u32) -> Self {
        let pubkey_bytes = public_key.as_bytes();
        // Simplified: using double SHA256 and taking 20 bytes
        // For full compatibility, implement RIPEMD160(SHA256(pubkey))
        let hash1 = Sha256::digest(pubkey_bytes);
        let hash2 = Sha256::digest(hash1);
        let address_bytes = hash2[..20].to_vec();
        
        let chain = if testnet { Chain::BitcoinTestnet } else { Chain::Bitcoin };
        Self::new(address_bytes, AddressType::BitcoinP2PKH, chain, index)
    }

    /// Generate an Ethereum address from a public key
    /// 
    /// Note: This is a simplified implementation using SHA256.
    /// For full Ethereum compatibility, use Keccak256 on the uncompressed
    /// public key (excluding the 0x04 prefix) and take the last 20 bytes.
    /// Addresses generated here are for internal use and may not be
    /// compatible with external Ethereum wallets.
    pub fn from_public_key_ethereum(public_key: &PublicKey, testnet: bool, index: u32) -> Self {
        let pubkey_bytes = public_key.as_bytes();
        // Simplified: using SHA256 instead of Keccak256
        // For full compatibility, implement Keccak256(uncompressed_pubkey[1:])
        let hash = Sha256::digest(pubkey_bytes);
        let address_bytes = hash[12..].to_vec(); // Last 20 bytes
        
        let chain = if testnet { Chain::EthereumSepolia } else { Chain::Ethereum };
        Self::new(address_bytes, AddressType::Ethereum, chain, index)
    }

    /// Get the raw address bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Get the address type
    pub fn address_type(&self) -> AddressType {
        self.address_type
    }

    /// Get the associated chain
    pub fn chain(&self) -> Chain {
        self.chain
    }

    /// Get the derivation index
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Format the address as a string according to its type
    pub fn to_string_formatted(&self) -> String {
        match self.address_type {
            AddressType::BitCell => {
                // Format: BC1 + base58 encoded
                format!("BC1{}", bs58::encode(&self.bytes).into_string())
            }
            AddressType::BitcoinP2PKH => {
                // Version byte + address + checksum
                let version = if self.chain == Chain::BitcoinTestnet { 0x6f } else { 0x00 };
                let mut data = vec![version];
                data.extend_from_slice(&self.bytes);
                // Checksum
                let checksum = Sha256::digest(Sha256::digest(&data));
                data.extend_from_slice(&checksum[..4]);
                bs58::encode(&data).into_string()
            }
            AddressType::BitcoinP2WPKH => {
                // Bech32 encoding (simplified)
                format!("bc1q{}", hex::encode(&self.bytes))
            }
            AddressType::Ethereum => {
                // Hex encoding with 0x prefix
                format!("0x{}", hex::encode(&self.bytes))
            }
        }
    }

    /// Parse an address from a string
    pub fn from_string(s: &str, chain: Chain) -> Result<Self> {
        match chain {
            Chain::BitCell => {
                if !s.starts_with("BC1") {
                    return Err(Error::InvalidAddress("BitCell address must start with BC1".into()));
                }
                let bytes = bs58::decode(&s[3..])
                    .into_vec()
                    .map_err(|e| Error::InvalidAddress(e.to_string()))?;
                Ok(Self::new(bytes, AddressType::BitCell, chain, 0))
            }
            Chain::Bitcoin | Chain::BitcoinTestnet => {
                let bytes = bs58::decode(s)
                    .into_vec()
                    .map_err(|e| Error::InvalidAddress(e.to_string()))?;
                if bytes.len() < 25 {
                    return Err(Error::InvalidAddress("Address too short".into()));
                }
                // Verify checksum
                let payload = &bytes[..bytes.len() - 4];
                let checksum = &bytes[bytes.len() - 4..];
                let computed_checksum = Sha256::digest(Sha256::digest(payload));
                if &computed_checksum[..4] != checksum {
                    return Err(Error::InvalidAddress("Invalid checksum".into()));
                }
                Ok(Self::new(payload[1..].to_vec(), AddressType::BitcoinP2PKH, chain, 0))
            }
            Chain::Ethereum | Chain::EthereumSepolia => {
                let s = s.strip_prefix("0x").unwrap_or(s);
                if s.len() != 40 {
                    return Err(Error::InvalidAddress("Ethereum address must be 40 hex chars".into()));
                }
                let bytes = hex::decode(s)
                    .map_err(|e| Error::InvalidAddress(e.to_string()))?;
                Ok(Self::new(bytes, AddressType::Ethereum, chain, 0))
            }
            Chain::Custom(_) => {
                Err(Error::InvalidAddress("Custom chain parsing not implemented".into()))
            }
        }
    }

    /// Validate that the address is well-formed
    pub fn is_valid(&self) -> bool {
        match self.address_type {
            AddressType::BitCell => self.bytes.len() == 20,
            AddressType::BitcoinP2PKH => self.bytes.len() == 20,
            AddressType::BitcoinP2WPKH => self.bytes.len() == 20,
            AddressType::Ethereum => self.bytes.len() == 20,
        }
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_formatted())
    }
}

/// Address manager for tracking multiple addresses
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AddressManager {
    /// All generated addresses
    addresses: Vec<Address>,
    /// Next index for each chain
    next_index: std::collections::HashMap<Chain, u32>,
}

impl AddressManager {
    /// Create a new address manager
    pub fn new() -> Self {
        Self {
            addresses: Vec::new(),
            next_index: std::collections::HashMap::new(),
        }
    }

    /// Add an address
    pub fn add_address(&mut self, address: Address) {
        let chain = address.chain();
        let index = address.index();
        self.addresses.push(address);
        // Update next index if needed
        let current = self.next_index.entry(chain).or_insert(0);
        if index >= *current {
            *current = index + 1;
        }
    }

    /// Get all addresses for a chain
    pub fn get_addresses(&self, chain: Chain) -> Vec<&Address> {
        self.addresses.iter().filter(|a| a.chain() == chain).collect()
    }

    /// Get the next derivation index for a chain
    pub fn next_index(&self, chain: Chain) -> u32 {
        *self.next_index.get(&chain).unwrap_or(&0)
    }

    /// Get all addresses
    pub fn all_addresses(&self) -> &[Address] {
        &self.addresses
    }

    /// Count addresses for a chain
    pub fn count(&self, chain: Chain) -> usize {
        self.addresses.iter().filter(|a| a.chain() == chain).count()
    }

    /// Find address by string representation
    pub fn find_by_string(&self, address_str: &str) -> Option<&Address> {
        self.addresses.iter().find(|a| a.to_string_formatted() == address_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcell_crypto::SecretKey;

    fn test_keypair() -> (SecretKey, PublicKey) {
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        (sk, pk)
    }

    #[test]
    fn test_bitcell_address_generation() {
        let (_, pk) = test_keypair();
        let address = Address::from_public_key_bitcell(&pk, 0);
        
        assert_eq!(address.chain(), Chain::BitCell);
        assert_eq!(address.address_type(), AddressType::BitCell);
        assert!(address.is_valid());
        assert!(address.to_string_formatted().starts_with("BC1"));
    }

    #[test]
    fn test_bitcoin_address_generation() {
        let (_, pk) = test_keypair();
        let address = Address::from_public_key_bitcoin(&pk, false, 0);
        
        assert_eq!(address.chain(), Chain::Bitcoin);
        assert_eq!(address.address_type(), AddressType::BitcoinP2PKH);
        assert!(address.is_valid());
    }

    #[test]
    fn test_ethereum_address_generation() {
        let (_, pk) = test_keypair();
        let address = Address::from_public_key_ethereum(&pk, false, 0);
        
        assert_eq!(address.chain(), Chain::Ethereum);
        assert_eq!(address.address_type(), AddressType::Ethereum);
        assert!(address.is_valid());
        assert!(address.to_string_formatted().starts_with("0x"));
    }

    #[test]
    fn test_address_deterministic() {
        let (_, pk) = test_keypair();
        let addr1 = Address::from_public_key_bitcell(&pk, 0);
        let addr2 = Address::from_public_key_bitcell(&pk, 0);
        
        assert_eq!(addr1.as_bytes(), addr2.as_bytes());
        assert_eq!(addr1.to_string_formatted(), addr2.to_string_formatted());
    }

    #[test]
    fn test_address_manager() {
        let (_, pk) = test_keypair();
        let mut manager = AddressManager::new();
        
        let addr1 = Address::from_public_key_bitcell(&pk, 0);
        let addr2 = Address::from_public_key_bitcell(&pk, 1);
        
        manager.add_address(addr1);
        manager.add_address(addr2);
        
        assert_eq!(manager.count(Chain::BitCell), 2);
        assert_eq!(manager.next_index(Chain::BitCell), 2);
    }

    #[test]
    fn test_address_parsing() {
        let (_, pk) = test_keypair();
        let original = Address::from_public_key_bitcell(&pk, 0);
        let formatted = original.to_string_formatted();
        
        let parsed = Address::from_string(&formatted, Chain::BitCell).unwrap();
        assert_eq!(original.as_bytes(), parsed.as_bytes());
    }

    #[test]
    fn test_ethereum_address_format() {
        let (_, pk) = test_keypair();
        let address = Address::from_public_key_ethereum(&pk, false, 0);
        let formatted = address.to_string_formatted();
        
        assert!(formatted.starts_with("0x"));
        assert_eq!(formatted.len(), 42); // 0x + 40 hex chars
    }

    #[test]
    fn test_testnet_addresses() {
        let (_, pk) = test_keypair();
        
        let btc_mainnet = Address::from_public_key_bitcoin(&pk, false, 0);
        let btc_testnet = Address::from_public_key_bitcoin(&pk, true, 0);
        
        assert_eq!(btc_mainnet.chain(), Chain::Bitcoin);
        assert_eq!(btc_testnet.chain(), Chain::BitcoinTestnet);
    }
}
