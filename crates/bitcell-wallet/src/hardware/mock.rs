//! Mock hardware wallet for testing

use crate::{Chain, Error, Result, Transaction};
use bitcell_crypto::{Hash256, PublicKey, Signature};
use super::{ConnectionStatus, HardwareWalletDevice, HardwareWalletType};

/// Mock hardware wallet for testing
pub struct MockHardwareWallet {
    secret_key: bitcell_crypto::SecretKey,
    connected: bool,
}

impl MockHardwareWallet {
    pub fn new() -> Self {
        Self {
            secret_key: bitcell_crypto::SecretKey::generate(),
            connected: true,
        }
    }
}

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
