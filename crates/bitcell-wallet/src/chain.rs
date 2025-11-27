//! Multi-chain Support
//!
//! Defines supported blockchains and their configurations.

use serde::{Deserialize, Serialize};

/// Supported blockchain networks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Chain {
    /// BitCell native blockchain
    #[default]
    BitCell,
    /// Bitcoin mainnet
    Bitcoin,
    /// Bitcoin testnet
    BitcoinTestnet,
    /// Ethereum mainnet
    Ethereum,
    /// Ethereum testnet (Sepolia)
    EthereumSepolia,
    /// Custom blockchain
    Custom(u32),
}

impl Chain {
    /// Get the chain ID
    pub fn chain_id(&self) -> u32 {
        match self {
            Chain::BitCell => 1,
            Chain::Bitcoin => 0,
            Chain::BitcoinTestnet => 1,
            Chain::Ethereum => 1,
            Chain::EthereumSepolia => 11155111,
            Chain::Custom(id) => *id,
        }
    }

    /// Get the BIP44 coin type for this chain
    pub fn coin_type(&self) -> u32 {
        match self {
            Chain::BitCell => 9999, // Custom coin type for BitCell
            Chain::Bitcoin => 0,
            Chain::BitcoinTestnet => 1,
            Chain::Ethereum | Chain::EthereumSepolia => 60,
            Chain::Custom(id) => *id,
        }
    }

    /// Get the chain name
    pub fn name(&self) -> &'static str {
        match self {
            Chain::BitCell => "BitCell",
            Chain::Bitcoin => "Bitcoin",
            Chain::BitcoinTestnet => "Bitcoin Testnet",
            Chain::Ethereum => "Ethereum",
            Chain::EthereumSepolia => "Ethereum Sepolia",
            Chain::Custom(_) => "Custom",
        }
    }

    /// Get the chain symbol
    pub fn symbol(&self) -> &'static str {
        match self {
            Chain::BitCell => "CELL",
            Chain::Bitcoin | Chain::BitcoinTestnet => "BTC",
            Chain::Ethereum | Chain::EthereumSepolia => "ETH",
            Chain::Custom(_) => "???",
        }
    }

    /// Get the number of decimal places for the native token
    pub fn decimals(&self) -> u8 {
        match self {
            Chain::BitCell => 8,
            Chain::Bitcoin | Chain::BitcoinTestnet => 8,
            Chain::Ethereum | Chain::EthereumSepolia => 18,
            Chain::Custom(_) => 8,
        }
    }

    /// Check if this is a testnet
    pub fn is_testnet(&self) -> bool {
        matches!(self, Chain::BitcoinTestnet | Chain::EthereumSepolia)
    }

    /// Get address prefix/version byte
    pub fn address_prefix(&self) -> &[u8] {
        match self {
            Chain::BitCell => b"BC",
            Chain::Bitcoin => &[0x00],
            Chain::BitcoinTestnet => &[0x6f],
            Chain::Ethereum | Chain::EthereumSepolia => b"0x",
            Chain::Custom(_) => b"CX",
        }
    }
}

/// Chain-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// The chain type
    pub chain: Chain,
    /// RPC endpoint URL (optional)
    pub rpc_url: Option<String>,
    /// Explorer URL (optional)
    pub explorer_url: Option<String>,
    /// Whether this chain is enabled
    pub enabled: bool,
}

impl ChainConfig {
    /// Create a new chain configuration
    pub fn new(chain: Chain) -> Self {
        Self {
            chain,
            rpc_url: None,
            explorer_url: None,
            enabled: true,
        }
    }

    /// Create with RPC URL
    pub fn with_rpc_url(mut self, url: &str) -> Self {
        self.rpc_url = Some(url.to_string());
        self
    }

    /// Create with explorer URL
    pub fn with_explorer_url(mut self, url: &str) -> Self {
        self.explorer_url = Some(url.to_string());
        self
    }

    /// Enable or disable the chain
    pub fn set_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Get default configuration for BitCell
    pub fn bitcell_default() -> Self {
        Self::new(Chain::BitCell)
            .with_rpc_url("http://localhost:8545")
            .with_explorer_url("https://explorer.bitcell.network")
    }
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self::new(Chain::BitCell)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_properties() {
        assert_eq!(Chain::BitCell.name(), "BitCell");
        assert_eq!(Chain::BitCell.symbol(), "CELL");
        assert_eq!(Chain::BitCell.decimals(), 8);
        assert!(!Chain::BitCell.is_testnet());
    }

    #[test]
    fn test_bitcoin_properties() {
        assert_eq!(Chain::Bitcoin.name(), "Bitcoin");
        assert_eq!(Chain::Bitcoin.symbol(), "BTC");
        assert_eq!(Chain::Bitcoin.decimals(), 8);
        assert!(!Chain::Bitcoin.is_testnet());
        assert_eq!(Chain::Bitcoin.coin_type(), 0);
    }

    #[test]
    fn test_ethereum_properties() {
        assert_eq!(Chain::Ethereum.name(), "Ethereum");
        assert_eq!(Chain::Ethereum.symbol(), "ETH");
        assert_eq!(Chain::Ethereum.decimals(), 18);
        assert!(!Chain::Ethereum.is_testnet());
        assert_eq!(Chain::Ethereum.coin_type(), 60);
    }

    #[test]
    fn test_testnet_detection() {
        assert!(!Chain::Bitcoin.is_testnet());
        assert!(Chain::BitcoinTestnet.is_testnet());
        assert!(!Chain::Ethereum.is_testnet());
        assert!(Chain::EthereumSepolia.is_testnet());
    }

    #[test]
    fn test_chain_config() {
        let config = ChainConfig::new(Chain::BitCell)
            .with_rpc_url("http://localhost:8545")
            .with_explorer_url("https://explorer.example.com");
        
        assert_eq!(config.chain, Chain::BitCell);
        assert_eq!(config.rpc_url, Some("http://localhost:8545".to_string()));
        assert!(config.enabled);
    }

    #[test]
    fn test_custom_chain() {
        let chain = Chain::Custom(12345);
        assert_eq!(chain.chain_id(), 12345);
        assert_eq!(chain.coin_type(), 12345);
        assert_eq!(chain.name(), "Custom");
    }

    #[test]
    fn test_default_chain() {
        let chain = Chain::default();
        assert_eq!(chain, Chain::BitCell);
    }
}
