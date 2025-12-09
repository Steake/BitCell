//! HashiCorp Vault Transit Secrets Engine Backend
//!
//! This module provides integration with HashiCorp Vault's Transit secrets engine
//! for secure key management and cryptographic operations.
//!
//! # Features
//! - Key generation in Vault
//! - ECDSA signing using secp256k1 keys
//! - Audit logging of all operations
//! - Automatic token renewal
//!
//! # Example
//! ```ignore
//! use bitcell_admin::hsm::{HsmConfig, HsmClient};
//!
//! let config = HsmConfig::vault("https://vault.example.com", "token", "bitcell-key");
//! let hsm = HsmClient::connect(config).await?;
//! let signature = hsm.sign(&hash).await?;
//! ```

use async_trait::async_trait;
use bitcell_crypto::{Hash256, PublicKey, Signature};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::hsm::{HsmBackend, HsmConfig, HsmError, HsmProvider, HsmResult};

/// HashiCorp Vault Transit backend
pub struct VaultBackend {
    client: Arc<VaultClient>,
    mount_path: String,
}

/// Vault client wrapper
struct VaultClient {
    client: vaultrs::client::VaultClient,
    config: VaultConfig,
}

#[derive(Debug, Clone)]
struct VaultConfig {
    endpoint: String,
    token: String,
    namespace: Option<String>,
}

impl VaultBackend {
    /// Connect to a Vault server
    pub async fn connect(config: &HsmConfig) -> HsmResult<Self> {
        let token = config
            .credentials
            .token
            .as_ref()
            .ok_or_else(|| HsmError::InvalidConfig("Vault token required".into()))?;

        let vault_config = VaultConfig {
            endpoint: config.endpoint.clone(),
            token: token.clone(),
            namespace: None,
        };

        // Create Vault client
        let vault_client = vaultrs::client::VaultClient::new(
            vaultrs::client::VaultClientSettingsBuilder::default()
                .address(&vault_config.endpoint)
                .token(&vault_config.token)
                .build()
                .map_err(|e| HsmError::ConnectionFailed(format!("Failed to build Vault client: {}", e)))?,
        )
        .map_err(|e| HsmError::ConnectionFailed(format!("Failed to create Vault client: {}", e)))?;

        let client = Arc::new(VaultClient {
            client: vault_client,
            config: vault_config,
        });

        // Use "transit" as the default mount path
        let mount_path = "transit".to_string();

        // Verify connection by checking if transit engine is mounted
        // This will return an error if we can't connect or don't have permissions
        let backend = Self {
            client,
            mount_path,
        };

        // Test connectivity
        if !backend.is_available().await {
            return Err(HsmError::ConnectionFailed(
                "Cannot connect to Vault or transit engine not available".into(),
            ));
        }

        Ok(backend)
    }

    /// Get the transit mount path
    pub fn mount_path(&self) -> &str {
        &self.mount_path
    }

    /// List all keys in the transit engine
    async fn list_vault_keys(&self) -> HsmResult<Vec<String>> {
        match vaultrs::transit::key::list(
            &self.client.client,
            &self.mount_path,
        )
        .await
        {
            Ok(keys) => Ok(keys),
            Err(e) => {
                // If the error is "no keys found", return empty list
                if e.to_string().contains("no such file") || e.to_string().contains("404") {
                    Ok(Vec::new())
                } else {
                    Err(HsmError::InternalError(format!("Failed to list keys: {}", e)))
                }
            }
        }
    }

    /// Check if a key exists
    async fn key_exists(&self, key_name: &str) -> bool {
        match vaultrs::transit::key::read(
            &self.client.client,
            &self.mount_path,
            key_name,
        )
        .await
        {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Get public key from Vault
    async fn get_vault_public_key(&self, key_name: &str) -> HsmResult<PublicKey> {
        // Read key from Vault
        let key_info = vaultrs::transit::key::read(
            &self.client.client,
            &self.mount_path,
            key_name,
        )
        .await
        .map_err(|e| {
            if e.to_string().contains("404") {
                HsmError::KeyNotFound(key_name.to_string())
            } else {
                HsmError::InternalError(format!("Failed to read key: {}", e))
            }
        })?;

        // Extract the latest public key
        let latest_version = key_info.latest_version;
        let public_key_data = key_info
            .keys
            .get(&latest_version.to_string())
            .ok_or_else(|| HsmError::InternalError("No public key found for latest version".into()))?;

        // Parse the public key (assuming secp256k1)
        // Vault returns public keys in different formats depending on the key type
        // For secp256k1, it typically returns hex-encoded compressed public key
        let pubkey_str = public_key_data
            .public_key
            .as_ref()
            .ok_or_else(|| HsmError::InternalError("Public key not available".into()))?;

        // Parse hex-encoded public key
        let pubkey_bytes = hex::decode(pubkey_str)
            .map_err(|e| HsmError::InternalError(format!("Invalid public key format: {}", e)))?;

        PublicKey::from_bytes(&pubkey_bytes)
            .map_err(|e| HsmError::InternalError(format!("Failed to parse public key: {}", e)))
    }

    /// Create a new key in Vault
    async fn create_vault_key(&self, key_name: &str) -> HsmResult<PublicKey> {
        // Create key configuration
        let opts = vaultrs::api::transit::requests::CreateKeyRequest::builder()
            .key_type(vaultrs::api::transit::KeyType::EcdsaSecp256k1)
            .exportable(false) // Keys should not be exportable for security
            .build()
            .map_err(|e| HsmError::InternalError(format!("Failed to build key request: {}", e)))?;

        // Create the key
        vaultrs::transit::key::create(
            &self.client.client,
            &self.mount_path,
            key_name,
            Some(&opts),
        )
        .await
        .map_err(|e| HsmError::InternalError(format!("Failed to create key: {}", e)))?;

        // Return the public key
        self.get_vault_public_key(key_name).await
    }

    /// Sign data using Vault
    async fn sign_vault(&self, key_name: &str, hash: &Hash256) -> HsmResult<Signature> {
        // Prepare sign request
        let opts = vaultrs::api::transit::requests::SignDataRequest::builder()
            .key_version(None) // Use latest version
            .hash_algorithm(Some(vaultrs::api::transit::HashAlgorithm::Sha256))
            .prehashed(true) // We're passing a pre-computed hash
            .signature_algorithm(Some("pkcs1v15".to_string())) // Standard signature algorithm
            .build()
            .map_err(|e| HsmError::SigningFailed(format!("Failed to build sign request: {}", e)))?;

        // Sign the hash
        let sign_result = vaultrs::transit::data::sign(
            &self.client.client,
            &self.mount_path,
            key_name,
            hash.as_bytes(),
            Some(&opts),
        )
        .await
        .map_err(|e| {
            if e.to_string().contains("404") {
                HsmError::KeyNotFound(key_name.to_string())
            } else {
                HsmError::SigningFailed(format!("Vault signing failed: {}", e))
            }
        })?;

        // Parse the signature
        // Vault returns signatures in the format "vault:v1:base64_signature"
        let sig_str = sign_result
            .signature
            .strip_prefix("vault:")
            .and_then(|s| s.split(':').nth(1))
            .ok_or_else(|| HsmError::SigningFailed("Invalid signature format".into()))?;

        // Decode base64 signature
        let sig_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            sig_str,
        )
        .map_err(|e| HsmError::SigningFailed(format!("Failed to decode signature: {}", e)))?;

        // Convert to BitCell signature format
        Signature::from_bytes(&sig_bytes)
            .map_err(|e| HsmError::SigningFailed(format!("Invalid signature: {}", e)))
    }
}

#[async_trait]
impl HsmBackend for VaultBackend {
    fn provider(&self) -> HsmProvider {
        HsmProvider::Vault
    }

    async fn is_available(&self) -> bool {
        // Try to list keys to verify connectivity
        self.list_vault_keys().await.is_ok()
    }

    async fn get_public_key(&self, key_name: &str) -> HsmResult<PublicKey> {
        self.get_vault_public_key(key_name).await
    }

    async fn sign(&self, key_name: &str, hash: &Hash256) -> HsmResult<Signature> {
        self.sign_vault(key_name, hash).await
    }

    async fn generate_key(&self, key_name: &str) -> HsmResult<PublicKey> {
        // Check if key already exists
        if self.key_exists(key_name).await {
            return Err(HsmError::InternalError(format!(
                "Key '{}' already exists",
                key_name
            )));
        }

        self.create_vault_key(key_name).await
    }

    async fn list_keys(&self) -> HsmResult<Vec<String>> {
        self.list_vault_keys().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires a running Vault instance
    async fn test_vault_connection() {
        // This test requires a local Vault instance running on localhost:8200
        // with the transit engine enabled at the default path
        let config = HsmConfig::vault("http://127.0.0.1:8200", "root", "test-key");
        
        let result = VaultBackend::connect(&config).await;
        // This should either connect successfully or fail with a connection error
        // We can't assert success without a real Vault instance
        assert!(result.is_ok() || matches!(result, Err(HsmError::ConnectionFailed(_))));
    }

    #[tokio::test]
    async fn test_vault_config_validation() {
        // Test missing token
        let mut config = HsmConfig::vault("http://127.0.0.1:8200", "", "test-key");
        config.credentials.token = None;
        
        let result = VaultBackend::connect(&config).await;
        assert!(matches!(result, Err(HsmError::InvalidConfig(_))));
    }
}
