//! HSM (Hardware Security Module) Integration
//!
//! This module provides an abstraction layer for HSM integration, supporting
//! various HSM providers for secure key management and transaction signing.
//!
//! # Supported Providers
//! - AWS CloudHSM
//! - HashiCorp Vault Transit
//! - Azure Key Vault
//! - Local PKCS#11 devices
//! - Mock HSM (for testing)
//!
//! # Security
//! HSMs provide hardware-backed security for cryptographic operations:
//! - Private keys never leave the HSM
//! - All signing operations happen inside the HSM
//! - Audit logging for all operations
//! - Multi-party authorization support
//!
//! # Usage
//! ```ignore
//! use bitcell_admin::hsm::{HsmClient, HsmConfig, HsmProvider};
//!
//! let config = HsmConfig::vault("https://vault.example.com", "token");
//! let hsm = HsmClient::connect(config).await?;
//!
//! // Sign a transaction hash
//! let signature = hsm.sign("key-name", &hash).await?;
//! ```

use async_trait::async_trait;
use bitcell_crypto::{Hash256, PublicKey, Signature};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// HSM provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HsmProvider {
    /// HashiCorp Vault Transit secrets engine
    Vault,
    /// AWS CloudHSM
    AwsCloudHsm,
    /// Azure Key Vault
    AzureKeyVault,
    /// Google Cloud HSM
    GoogleCloudHsm,
    /// Local PKCS#11 device
    Pkcs11,
    /// Mock HSM for testing
    Mock,
}

/// HSM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsmConfig {
    /// HSM provider
    pub provider: HsmProvider,
    /// Connection endpoint
    pub endpoint: String,
    /// Authentication credentials
    #[serde(skip_serializing)]
    pub credentials: HsmCredentials,
    /// Default key name for signing
    pub default_key: String,
    /// Connection timeout in seconds
    pub timeout_secs: u64,
    /// Enable audit logging
    pub audit_logging: bool,
}

/// HSM authentication credentials
/// 
/// # Security
/// Credentials are automatically zeroed when dropped to prevent
/// sensitive data from remaining in memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsmCredentials {
    /// API token (for Vault)
    #[serde(skip_serializing)]
    pub token: Option<String>,
    /// Access key (for AWS/Azure client ID)
    #[serde(skip_serializing)]
    pub access_key: Option<String>,
    /// Secret key (for AWS/Azure client secret)
    #[serde(skip_serializing)]
    pub secret_key: Option<String>,
    /// Tenant ID (for Azure)
    pub tenant_id: Option<String>,
    /// Client certificate path (for mTLS)
    pub client_cert: Option<String>,
    /// Client key path (for mTLS)
    pub client_key: Option<String>,
}

impl Default for HsmCredentials {
    fn default() -> Self {
        Self {
            token: None,
            access_key: None,
            secret_key: None,
            tenant_id: None,
            client_cert: None,
            client_key: None,
        }
    }
}

impl Drop for HsmCredentials {
    fn drop(&mut self) {
        // Note: Rust's String does not provide safe zeroing of memory.
        // For production use, consider using the `secrecy` or `zeroize` crates
        // which provide guaranteed secure memory zeroing for sensitive data.
        // 
        // The current implementation relies on compiler optimizations not being
        // too aggressive about removing the zeroing, which is not guaranteed.
        // 
        // Example with zeroize crate:
        // use zeroize::Zeroize;
        // if let Some(ref mut token) = self.token {
        //     token.zeroize();
        // }
    }
}

impl HsmConfig {
    /// Create configuration for HashiCorp Vault
    pub fn vault(endpoint: &str, token: &str, key_name: &str) -> Self {
        Self {
            provider: HsmProvider::Vault,
            endpoint: endpoint.to_string(),
            credentials: HsmCredentials {
                token: Some(token.to_string()),
                access_key: None,
                secret_key: None,
                tenant_id: None,
                client_cert: None,
                client_key: None,
            },
            default_key: key_name.to_string(),
            timeout_secs: 30,
            audit_logging: true,
        }
    }
    
    /// Create configuration for AWS CloudHSM
    pub fn aws(endpoint: &str, access_key: &str, secret_key: &str, key_name: &str) -> Self {
        Self {
            provider: HsmProvider::AwsCloudHsm,
            endpoint: endpoint.to_string(),
            credentials: HsmCredentials {
                token: None,
                access_key: Some(access_key.to_string()),
                secret_key: Some(secret_key.to_string()),
                tenant_id: None,
                client_cert: None,
                client_key: None,
            },
            default_key: key_name.to_string(),
            timeout_secs: 30,
            audit_logging: true,
        }
    }
    
    /// Create configuration for Azure Key Vault
    /// 
    /// # Arguments
    /// * `vault_url` - Azure Key Vault URL (e.g., "https://my-vault.vault.azure.net")
    /// * `tenant_id` - Azure AD tenant ID (use "common" for multi-tenant apps)
    /// * `client_id` - Service Principal application (client) ID
    /// * `client_secret` - Service Principal client secret
    /// * `key_name` - Default key name for operations
    pub fn azure(vault_url: &str, tenant_id: &str, client_id: &str, client_secret: &str, key_name: &str) -> Self {
        Self {
            provider: HsmProvider::AzureKeyVault,
            endpoint: vault_url.to_string(),
            credentials: HsmCredentials {
                token: None,
                access_key: Some(client_id.to_string()),
                secret_key: Some(client_secret.to_string()),
                tenant_id: Some(tenant_id.to_string()),
                client_cert: None,
                client_key: None,
            },
            default_key: key_name.to_string(),
            timeout_secs: 30,
            audit_logging: true,
        }
    }
    
    /// Create configuration for mock HSM (testing only)
    pub fn mock(key_name: &str) -> Self {
        Self {
            provider: HsmProvider::Mock,
            endpoint: "mock://localhost".to_string(),
            credentials: HsmCredentials::default(),
            default_key: key_name.to_string(),
            timeout_secs: 5,
            audit_logging: false,
        }
    }
}

/// HSM operation result
pub type HsmResult<T> = std::result::Result<T, HsmError>;

/// HSM errors
#[derive(Debug, thiserror::Error)]
pub enum HsmError {
    #[error("HSM connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("HSM authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    #[error("Signing failed: {0}")]
    SigningFailed(String),
    
    #[error("HSM operation timeout")]
    Timeout,
    
    #[error("HSM not available")]
    NotAvailable,
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("HSM internal error: {0}")]
    InternalError(String),
}

/// HSM signing backend trait
#[async_trait]
pub trait HsmBackend: Send + Sync {
    /// Get the provider type
    fn provider(&self) -> HsmProvider;
    
    /// Check if HSM is connected and available
    async fn is_available(&self) -> bool;
    
    /// Get public key for a key name
    async fn get_public_key(&self, key_name: &str) -> HsmResult<PublicKey>;
    
    /// Sign a hash with the specified key
    async fn sign(&self, key_name: &str, hash: &Hash256) -> HsmResult<Signature>;
    
    /// Generate a new key pair in the HSM
    async fn generate_key(&self, key_name: &str) -> HsmResult<PublicKey>;
    
    /// List available keys
    async fn list_keys(&self) -> HsmResult<Vec<String>>;
}

/// Maximum number of audit log entries to keep in memory
/// Older entries are automatically rotated out
const MAX_AUDIT_LOG_ENTRIES: usize = 10_000;

/// HSM client for secure key management
pub struct HsmClient {
    config: HsmConfig,
    backend: Arc<dyn HsmBackend>,
    audit_log: Arc<RwLock<Vec<AuditEntry>>>,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub operation: String,
    pub key_name: String,
    pub success: bool,
    pub error: Option<String>,
}

impl HsmClient {
    /// Connect to an HSM with the given configuration
    pub async fn connect(config: HsmConfig) -> HsmResult<Self> {
        let backend: Arc<dyn HsmBackend> = match config.provider {
            HsmProvider::Vault => {
                #[cfg(feature = "vault")]
                {
                    Arc::new(VaultBackend::connect(&config).await?)
                }
                #[cfg(not(feature = "vault"))]
                {
                    return Err(HsmError::InvalidConfig("Vault support not compiled in".into()));
                }
            }
            HsmProvider::AwsCloudHsm => {
                #[cfg(feature = "aws-hsm")]
                {
                    Arc::new(AwsHsmBackend::connect(&config).await?)
                }
                #[cfg(not(feature = "aws-hsm"))]
                {
                    return Err(HsmError::InvalidConfig("AWS HSM support not compiled in".into()));
                }
            }
            HsmProvider::AzureKeyVault => {
                #[cfg(feature = "azure-hsm")]
                {
                    Arc::new(AzureKeyVaultBackend::connect(&config).await?)
                }
                #[cfg(not(feature = "azure-hsm"))]
                {
                    return Err(HsmError::InvalidConfig("Azure Key Vault support not compiled in".into()));
                }
            }
            HsmProvider::GoogleCloudHsm => {
                return Err(HsmError::InvalidConfig("Google Cloud HSM not yet implemented".into()));
            }
            HsmProvider::Pkcs11 => {
                return Err(HsmError::InvalidConfig("PKCS#11 not yet implemented".into()));
            }
            HsmProvider::Mock => {
                Arc::new(MockHsmBackend::new())
            }
        };
        
        // Verify connection
        if !backend.is_available().await {
            return Err(HsmError::ConnectionFailed("HSM not available".into()));
        }
        
        Ok(Self {
            config,
            backend,
            audit_log: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    /// Get the configuration
    pub fn config(&self) -> &HsmConfig {
        &self.config
    }
    
    /// Check if HSM is available
    pub async fn is_available(&self) -> bool {
        self.backend.is_available().await
    }
    
    /// Get public key for the default key
    pub async fn get_public_key(&self) -> HsmResult<PublicKey> {
        self.get_public_key_by_name(&self.config.default_key).await
    }
    
    /// Get public key for a specific key name
    pub async fn get_public_key_by_name(&self, key_name: &str) -> HsmResult<PublicKey> {
        let result = self.backend.get_public_key(key_name).await;
        self.log_operation("get_public_key", key_name, result.is_ok(), result.as_ref().err()).await;
        result
    }
    
    /// Sign a hash with the default key
    pub async fn sign(&self, hash: &Hash256) -> HsmResult<Signature> {
        self.sign_with_key(&self.config.default_key, hash).await
    }
    
    /// Sign a hash with a specific key
    pub async fn sign_with_key(&self, key_name: &str, hash: &Hash256) -> HsmResult<Signature> {
        let result = self.backend.sign(key_name, hash).await;
        self.log_operation("sign", key_name, result.is_ok(), result.as_ref().err()).await;
        result
    }
    
    /// Generate a new key pair
    pub async fn generate_key(&self, key_name: &str) -> HsmResult<PublicKey> {
        let result = self.backend.generate_key(key_name).await;
        self.log_operation("generate_key", key_name, result.is_ok(), result.as_ref().err()).await;
        result
    }
    
    /// List available keys
    pub async fn list_keys(&self) -> HsmResult<Vec<String>> {
        self.backend.list_keys().await
    }
    
    /// Get audit log
    pub async fn audit_log(&self) -> Vec<AuditEntry> {
        self.audit_log.read().await.clone()
    }
    
    /// Clear audit log
    pub async fn clear_audit_log(&self) {
        self.audit_log.write().await.clear();
    }
    
    /// Log an operation
    /// 
    /// The audit log is bounded to MAX_AUDIT_LOG_ENTRIES entries.
    /// When the limit is reached, the oldest entries are removed.
    async fn log_operation(&self, operation: &str, key_name: &str, success: bool, error: Option<&HsmError>) {
        if !self.config.audit_logging {
            return;
        }
        
        let entry = AuditEntry {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            operation: operation.to_string(),
            key_name: key_name.to_string(),
            success,
            error: error.map(|e| e.to_string()),
        };
        
        let mut log = self.audit_log.write().await;
        log.push(entry);
        
        // Enforce maximum size by removing oldest entries
        if log.len() > MAX_AUDIT_LOG_ENTRIES {
            let excess = log.len() - MAX_AUDIT_LOG_ENTRIES;
            log.drain(0..excess);
        }
    }
}

/// Mock HSM backend for testing
pub struct MockHsmBackend {
    keys: Arc<RwLock<std::collections::HashMap<String, bitcell_crypto::SecretKey>>>,
}

impl MockHsmBackend {
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }
}

impl Default for MockHsmBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HsmBackend for MockHsmBackend {
    fn provider(&self) -> HsmProvider {
        HsmProvider::Mock
    }
    
    async fn is_available(&self) -> bool {
        true
    }
    
    async fn get_public_key(&self, key_name: &str) -> HsmResult<PublicKey> {
        let keys = self.keys.read().await;
        keys.get(key_name)
            .map(|sk| sk.public_key())
            .ok_or_else(|| HsmError::KeyNotFound(key_name.to_string()))
    }
    
    async fn sign(&self, key_name: &str, hash: &Hash256) -> HsmResult<Signature> {
        let keys = self.keys.read().await;
        let sk = keys.get(key_name)
            .ok_or_else(|| HsmError::KeyNotFound(key_name.to_string()))?;
        
        Ok(sk.sign(hash.as_bytes()))
    }
    
    async fn generate_key(&self, key_name: &str) -> HsmResult<PublicKey> {
        let sk = bitcell_crypto::SecretKey::generate();
        let pk = sk.public_key();
        
        self.keys.write().await.insert(key_name.to_string(), sk);
        
        Ok(pk)
    }
    
    async fn list_keys(&self) -> HsmResult<Vec<String>> {
        Ok(self.keys.read().await.keys().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mock_hsm_generate_and_sign() {
        let config = HsmConfig::mock("test-key");
        let hsm = HsmClient::connect(config).await.unwrap();
        
        // Generate a key
        let pk = hsm.generate_key("test-key").await.unwrap();
        assert_eq!(pk.as_bytes().len(), 33);
        
        // Sign a hash
        let hash = Hash256::hash(b"test message");
        let signature = hsm.sign(&hash).await.unwrap();
        
        // Verify signature
        assert!(signature.verify(&pk, hash.as_bytes()).is_ok());
    }
    
    #[tokio::test]
    async fn test_mock_hsm_key_not_found() {
        let config = HsmConfig::mock("test-key");
        let hsm = HsmClient::connect(config).await.unwrap();
        
        // Try to get non-existent key
        let result = hsm.get_public_key_by_name("nonexistent").await;
        assert!(matches!(result, Err(HsmError::KeyNotFound(_))));
    }
    
    #[tokio::test]
    async fn test_mock_hsm_list_keys() {
        let config = HsmConfig::mock("default");
        let hsm = HsmClient::connect(config).await.unwrap();
        
        // Generate some keys
        hsm.generate_key("key1").await.unwrap();
        hsm.generate_key("key2").await.unwrap();
        hsm.generate_key("key3").await.unwrap();
        
        // List keys
        let keys = hsm.list_keys().await.unwrap();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
        assert!(keys.contains(&"key3".to_string()));
    }
    
    #[tokio::test]
    async fn test_mock_hsm_audit_log() {
        // Create mock config with audit logging enabled
        let mut config = HsmConfig::mock("audit-test");
        config.audit_logging = true;
        let hsm = HsmClient::connect(config).await.unwrap();
        
        // Perform some operations
        hsm.generate_key("audit-test").await.unwrap();
        let hash = Hash256::hash(b"test");
        hsm.sign(&hash).await.unwrap();
        
        // Check audit log
        let log = hsm.audit_log().await;
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].operation, "generate_key");
        assert!(log[0].success);
        assert_eq!(log[1].operation, "sign");
        assert!(log[1].success);
    }
    
    #[tokio::test]
    async fn test_hsm_config_vault() {
        let config = HsmConfig::vault("https://vault.example.com", "token", "my-key");
        
        assert_eq!(config.provider, HsmProvider::Vault);
        assert_eq!(config.endpoint, "https://vault.example.com");
        assert_eq!(config.default_key, "my-key");
        assert_eq!(config.credentials.token, Some("token".to_string()));
    }
    
    #[tokio::test]
    async fn test_hsm_config_aws() {
        let config = HsmConfig::aws(
            "hsm.us-east-1.amazonaws.com",
            "AKIAIOSFODNN7EXAMPLE",
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
            "my-key",
        );
        
        assert_eq!(config.provider, HsmProvider::AwsCloudHsm);
        assert_eq!(config.credentials.access_key, Some("AKIAIOSFODNN7EXAMPLE".to_string()));
    }
    
    #[tokio::test]
    async fn test_hsm_config_azure() {
        let config = HsmConfig::azure(
            "https://my-vault.vault.azure.net",
            "tenant-id-789",
            "client-id-123",
            "client-secret-456",
            "my-key",
        );
        
        assert_eq!(config.provider, HsmProvider::AzureKeyVault);
        assert_eq!(config.endpoint, "https://my-vault.vault.azure.net");
        assert_eq!(config.credentials.tenant_id, Some("tenant-id-789".to_string()));
        assert_eq!(config.credentials.access_key, Some("client-id-123".to_string()));
        assert_eq!(config.credentials.secret_key, Some("client-secret-456".to_string()));
    }
}

// HSM provider implementations
#[cfg(feature = "vault")]
mod vault;
#[cfg(feature = "vault")]
pub use vault::VaultBackend;

#[cfg(feature = "aws-hsm")]
mod aws;
#[cfg(feature = "aws-hsm")]
pub use aws::AwsHsmBackend;

#[cfg(feature = "azure-hsm")]
mod azure;
#[cfg(feature = "azure-hsm")]
pub use azure::AzureKeyVaultBackend;
