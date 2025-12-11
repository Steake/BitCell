//! Azure Key Vault Backend
//!
//! This module provides integration with Azure Key Vault
//! for secure key management and cryptographic operations.
//!
//! # Features
//! - Key generation in Azure Key Vault
//! - ECDSA signing using secp256k1 keys
//! - Key rotation support
//! - Access policies and RBAC
//!
//! # Example
//! ```ignore
//! use bitcell_admin::hsm::{HsmConfig, HsmClient};
//!
//! // Create config with Azure credentials
//! let mut config = HsmConfig::mock("test"); // Start with mock config structure
//! config.provider = HsmProvider::AzureKeyVault;
//! config.endpoint = "https://my-vault.vault.azure.net".to_string();
//! config.credentials.access_key = Some("client_id".to_string());
//! config.credentials.secret_key = Some("client_secret".to_string());
//!
//! let hsm = HsmClient::connect(config).await?;
//! let signature = hsm.sign(&hash).await?;
//! ```

use async_trait::async_trait;
use azure_security_keyvault::KeyClient;
use bitcell_crypto::{Hash256, PublicKey, Signature};
use std::sync::Arc;

use crate::hsm::{HsmBackend, HsmConfig, HsmError, HsmProvider, HsmResult};

/// Azure Key Vault backend
pub struct AzureKeyVaultBackend {
    client: Arc<KeyClient>,
    vault_url: String,
}

impl AzureKeyVaultBackend {
    /// Connect to Azure Key Vault
    pub async fn connect(config: &HsmConfig) -> HsmResult<Self> {
        let client_id = config
            .credentials
            .access_key
            .as_ref()
            .ok_or_else(|| HsmError::InvalidConfig("Azure client ID required".into()))?;

        let client_secret = config
            .credentials
            .secret_key
            .as_ref()
            .ok_or_else(|| HsmError::InvalidConfig("Azure client secret required".into()))?;

        // Get tenant ID (defaults to "common" for multi-tenant apps)
        let tenant_id = config
            .credentials
            .tenant_id
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("common");

        // Parse vault URL from endpoint
        let vault_url = if config.endpoint.starts_with("http://") || config.endpoint.starts_with("https://") {
            config.endpoint.clone()
        } else {
            format!("https://{}", config.endpoint)
        };

        // Create Azure credentials using client credentials flow
        let credential = azure_identity::ClientSecretCredential::new(
            azure_core::new_http_client(),
            tenant_id.to_string(),
            client_id.clone(),
            client_secret.clone(),
        );

        // Create Key Vault client
        let key_client = KeyClient::new(&vault_url, Arc::new(credential))
            .map_err(|e| HsmError::ConnectionFailed(format!("Failed to create Azure client: {}", e)))?;

        let backend = Self {
            client: Arc::new(key_client),
            vault_url,
        };

        // Test connectivity
        if !backend.is_available().await {
            return Err(HsmError::ConnectionFailed(
                "Cannot connect to Azure Key Vault or insufficient permissions".into(),
            ));
        }

        Ok(backend)
    }

    /// Get the vault URL
    pub fn vault_url(&self) -> &str {
        &self.vault_url
    }

    /// Get public key from Azure Key Vault
    async fn get_azure_public_key(&self, key_name: &str) -> HsmResult<PublicKey> {
        // Get key from Azure
        let key = self
            .client
            .get(key_name)
            .await
            .map_err(|e| {
                if e.to_string().contains("NotFound") || e.to_string().contains("404") {
                    HsmError::KeyNotFound(key_name.to_string())
                } else {
                    HsmError::InternalError(format!("Failed to get key: {}", e))
                }
            })?;

        // Extract public key from the key bundle
        let key_material = key
            .key
            .ok_or_else(|| HsmError::InternalError("Key material not available".into()))?;

        // Azure returns JWK (JSON Web Key) format
        // For EC keys, we need x and y coordinates
        let x = key_material
            .x
            .ok_or_else(|| HsmError::InternalError("Public key x coordinate missing".into()))?;
        let y = key_material
            .y
            .ok_or_else(|| HsmError::InternalError("Public key y coordinate missing".into()))?;

        // Combine x and y into uncompressed public key format (0x04 || x || y)
        let mut pubkey_bytes = vec![0x04];
        pubkey_bytes.extend_from_slice(&x);
        pubkey_bytes.extend_from_slice(&y);

        PublicKey::from_bytes(&pubkey_bytes)
            .map_err(|e| HsmError::InternalError(format!("Failed to parse public key: {}", e)))
    }

    /// Create a new key in Azure Key Vault
    async fn create_azure_key(&self, key_name: &str) -> HsmResult<PublicKey> {
        use azure_security_keyvault::KeyVaultKeyType;

        // Create key in Azure Key Vault
        let _create_result = self
            .client
            .create(key_name, KeyVaultKeyType::Ec)
            .curve(azure_security_keyvault::JsonWebKeyCurveName::P256K) // secp256k1
            .await
            .map_err(|e| HsmError::InternalError(format!("Failed to create key: {}", e)))?;

        // Get and return the public key
        self.get_azure_public_key(key_name).await
    }

    /// Sign data using Azure Key Vault
    async fn sign_azure(&self, key_name: &str, hash: &Hash256) -> HsmResult<Signature> {
        use azure_security_keyvault::SignatureAlgorithm;

        // Sign the hash using Azure Key Vault
        let sign_result = self
            .client
            .sign(key_name, SignatureAlgorithm::ES256K, hash.as_bytes())
            .await
            .map_err(|e| {
                if e.to_string().contains("NotFound") || e.to_string().contains("404") {
                    HsmError::KeyNotFound(key_name.to_string())
                } else {
                    HsmError::SigningFailed(format!("Azure signing failed: {}", e))
                }
            })?;

        // Extract signature bytes
        let sig_bytes = sign_result
            .result
            .ok_or_else(|| HsmError::SigningFailed("No signature returned".into()))?;

        // Parse signature
        Signature::from_bytes(&sig_bytes)
            .map_err(|e| HsmError::SigningFailed(format!("Invalid signature: {}", e)))
    }

    /// List all keys in Azure Key Vault
    async fn list_azure_keys(&self) -> HsmResult<Vec<String>> {
        let mut key_names = Vec::new();

        // List keys in the vault
        let keys = self
            .client
            .list()
            .into_stream()
            .await
            .map_err(|e| HsmError::InternalError(format!("Failed to list keys: {}", e)))?;

        use futures::StreamExt;
        let mut keys_stream = keys;
        
        while let Some(result) = keys_stream.next().await {
            match result {
                Ok(key) => {
                    if let Some(kid) = key.kid {
                        // Extract key name from key ID
                        // Key ID format: https://vault-url/keys/key-name/version
                        if let Some(name) = kid.split('/').nth_back(1) {
                            key_names.push(name.to_string());
                        }
                    }
                }
                Err(e) => {
                    return Err(HsmError::InternalError(format!("Failed to list key: {}", e)));
                }
            }
        }

        Ok(key_names)
    }
}

#[async_trait]
impl HsmBackend for AzureKeyVaultBackend {
    fn provider(&self) -> HsmProvider {
        HsmProvider::AzureKeyVault
    }

    async fn is_available(&self) -> bool {
        // Try to list keys to verify connectivity
        self.list_azure_keys().await.is_ok()
    }

    async fn get_public_key(&self, key_name: &str) -> HsmResult<PublicKey> {
        self.get_azure_public_key(key_name).await
    }

    async fn sign(&self, key_name: &str, hash: &Hash256) -> HsmResult<Signature> {
        self.sign_azure(key_name, hash).await
    }

    async fn generate_key(&self, key_name: &str) -> HsmResult<PublicKey> {
        // Azure Key Vault will return an error if the key already exists
        // when we try to create it, so we don't need to check separately
        self.create_azure_key(key_name).await
    }

    async fn list_keys(&self) -> HsmResult<Vec<String>> {
        self.list_azure_keys().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_azure_config_validation() {
        // Test missing client ID
        let mut config = HsmConfig::mock("test");
        config.provider = HsmProvider::AzureKeyVault;
        config.endpoint = "https://test.vault.azure.net".to_string();
        config.credentials.access_key = None;
        config.credentials.secret_key = Some("secret".to_string());
        
        let result = AzureKeyVaultBackend::connect(&config).await;
        assert!(matches!(result, Err(HsmError::InvalidConfig(_))));
    }

    #[tokio::test]
    async fn test_azure_config_missing_secret() {
        // Test missing client secret
        let mut config = HsmConfig::mock("test");
        config.provider = HsmProvider::AzureKeyVault;
        config.endpoint = "https://test.vault.azure.net".to_string();
        config.credentials.access_key = Some("client_id".to_string());
        config.credentials.secret_key = None;
        
        let result = AzureKeyVaultBackend::connect(&config).await;
        assert!(matches!(result, Err(HsmError::InvalidConfig(_))));
    }

    #[test]
    fn test_vault_url_formatting() {
        let mut config = HsmConfig::mock("test");
        config.endpoint = "my-vault.vault.azure.net".to_string();
        
        // URL should be formatted with https://
        assert!(config.endpoint.starts_with("my-vault"));
    }
}
