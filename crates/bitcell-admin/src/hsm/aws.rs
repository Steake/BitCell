//! AWS CloudHSM / KMS Backend
//!
//! This module provides integration with AWS Key Management Service (KMS)
//! for secure key management and cryptographic operations.
//!
//! # Features
//! - Key generation in AWS KMS
//! - ECDSA signing using secp256k1 keys
//! - Multi-AZ support
//! - CloudTrail audit logging
//!
//! # Example
//! ```ignore
//! use bitcell_admin::hsm::{HsmConfig, HsmClient};
//!
//! let config = HsmConfig::aws(
//!     "kms.us-east-1.amazonaws.com",
//!     "AKIAIOSFODNN7EXAMPLE",
//!     "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
//!     "bitcell-key"
//! );
//! let hsm = HsmClient::connect(config).await?;
//! let signature = hsm.sign(&hash).await?;
//! ```

use async_trait::async_trait;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_kms::types::{KeySpec, KeyUsageType, MessageType, SigningAlgorithmSpec};
use bitcell_crypto::{Hash256, PublicKey, Signature};
use std::sync::Arc;

use crate::hsm::{HsmBackend, HsmConfig, HsmError, HsmProvider, HsmResult};

/// AWS CloudHSM / KMS backend
pub struct AwsHsmBackend {
    client: Arc<aws_sdk_kms::Client>,
    region: String,
    key_ids: Arc<tokio::sync::RwLock<std::collections::HashMap<String, String>>>,
}

impl AwsHsmBackend {
    /// Connect to AWS KMS
    pub async fn connect(config: &HsmConfig) -> HsmResult<Self> {
        let access_key = config
            .credentials
            .access_key
            .as_ref()
            .ok_or_else(|| HsmError::InvalidConfig("AWS access key required".into()))?;

        let secret_key = config
            .credentials
            .secret_key
            .as_ref()
            .ok_or_else(|| HsmError::InvalidConfig("AWS secret key required".into()))?;

        // Extract region from endpoint or use default
        let region = Self::extract_region(&config.endpoint).unwrap_or_else(|| "us-east-1".to_string());

        // Create AWS credentials
        let credentials_provider = aws_sdk_kms::config::Credentials::new(
            access_key,
            secret_key,
            None, // session token
            None, // expiry
            "bitcell-admin",
        );

        // Build AWS config
        let aws_config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(region.clone()))
            .credentials_provider(credentials_provider)
            .load()
            .await;

        // Create KMS client
        let kms_client = aws_sdk_kms::Client::new(&aws_config);

        let backend = Self {
            client: Arc::new(kms_client),
            region,
            key_ids: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        };

        // Test connectivity by listing keys (with limit 1)
        if !backend.is_available().await {
            return Err(HsmError::ConnectionFailed(
                "Cannot connect to AWS KMS or insufficient permissions".into(),
            ));
        }

        Ok(backend)
    }

    /// Extract AWS region from endpoint
    fn extract_region(endpoint: &str) -> Option<String> {
        // Parse region from endpoints like "kms.us-east-1.amazonaws.com"
        if let Some(start) = endpoint.find("kms.") {
            if let Some(end) = endpoint[start + 4..].find(".amazonaws.com") {
                return Some(endpoint[start + 4..start + 4 + end].to_string());
            }
        }
        None
    }

    /// Get AWS region
    pub fn region(&self) -> &str {
        &self.region
    }

    /// Find key ID by alias
    async fn find_key_id(&self, key_name: &str) -> HsmResult<String> {
        // Check cache first
        {
            let cache = self.key_ids.read().await;
            if let Some(key_id) = cache.get(key_name) {
                return Ok(key_id.clone());
            }
        }

        // List keys and find by alias
        let alias = format!("alias/{}", key_name);
        
        match self
            .client
            .describe_key()
            .key_id(&alias)
            .send()
            .await
        {
            Ok(response) => {
                if let Some(metadata) = response.key_metadata {
                    if let Some(key_id) = metadata.key_id {
                        // Cache the result
                        self.key_ids.write().await.insert(key_name.to_string(), key_id.clone());
                        return Ok(key_id);
                    }
                }
                Err(HsmError::KeyNotFound(key_name.to_string()))
            }
            Err(e) => {
                if e.to_string().contains("NotFoundException") {
                    Err(HsmError::KeyNotFound(key_name.to_string()))
                } else {
                    Err(HsmError::InternalError(format!("Failed to find key: {}", e)))
                }
            }
        }
    }

    /// Get public key from AWS KMS
    async fn get_aws_public_key(&self, key_name: &str) -> HsmResult<PublicKey> {
        let key_id = self.find_key_id(key_name).await?;

        // Get public key from KMS
        let response = self
            .client
            .get_public_key()
            .key_id(&key_id)
            .send()
            .await
            .map_err(|e| HsmError::InternalError(format!("Failed to get public key: {}", e)))?;

        // Extract public key bytes
        let pubkey_bytes = response
            .public_key
            .ok_or_else(|| HsmError::InternalError("Public key not available".into()))?
            .into_inner();

        // AWS returns DER-encoded public key, we need to extract the raw key
        // For secp256k1, the last 65 bytes (or 33 for compressed) are the actual key
        PublicKey::from_bytes(&pubkey_bytes)
            .or_else(|_| {
                // Try extracting from DER if direct parsing fails
                if pubkey_bytes.len() >= 65 {
                    PublicKey::from_bytes(&pubkey_bytes[pubkey_bytes.len() - 65..])
                } else if pubkey_bytes.len() >= 33 {
                    PublicKey::from_bytes(&pubkey_bytes[pubkey_bytes.len() - 33..])
                } else {
                    Err(bitcell_crypto::CryptoError::InvalidPublicKey)
                }
            })
            .map_err(|e| HsmError::InternalError(format!("Failed to parse public key: {}", e)))
    }

    /// Create a new key in AWS KMS
    async fn create_aws_key(&self, key_name: &str) -> HsmResult<PublicKey> {
        // Create key in KMS
        let create_response = self
            .client
            .create_key()
            .key_spec(KeySpec::EccSecgP256K1) // secp256k1
            .key_usage(KeyUsageType::SignVerify)
            .description(format!("BitCell key: {}", key_name))
            .send()
            .await
            .map_err(|e| HsmError::InternalError(format!("Failed to create key: {}", e)))?;

        let key_id = create_response
            .key_metadata
            .and_then(|m| m.key_id)
            .ok_or_else(|| HsmError::InternalError("Failed to get key ID".into()))?;

        // Create alias for the key
        let alias = format!("alias/{}", key_name);
        self.client
            .create_alias()
            .alias_name(&alias)
            .target_key_id(&key_id)
            .send()
            .await
            .map_err(|e| HsmError::InternalError(format!("Failed to create alias: {}", e)))?;

        // Cache the key ID
        self.key_ids.write().await.insert(key_name.to_string(), key_id);

        // Get and return the public key
        self.get_aws_public_key(key_name).await
    }

    /// Sign data using AWS KMS
    async fn sign_aws(&self, key_name: &str, hash: &Hash256) -> HsmResult<Signature> {
        let key_id = self.find_key_id(key_name).await?;

        // Sign the hash using KMS
        let response = self
            .client
            .sign()
            .key_id(&key_id)
            .message_type(MessageType::Digest) // We're providing a pre-computed hash
            .signing_algorithm(SigningAlgorithmSpec::EcdsaSha256) // secp256k1 with SHA-256
            .message(aws_sdk_kms::primitives::Blob::new(hash.as_bytes()))
            .send()
            .await
            .map_err(|e| HsmError::SigningFailed(format!("AWS KMS signing failed: {}", e)))?;

        // Extract signature bytes
        let sig_bytes = response
            .signature
            .ok_or_else(|| HsmError::SigningFailed("No signature returned".into()))?
            .into_inner();

        // Parse AWS DER-encoded signature to BitCell format
        Signature::from_bytes(&sig_bytes)
            .or_else(|_| {
                // Try extracting from DER if direct parsing fails
                // AWS returns DER-encoded ECDSA signature
                Self::parse_der_signature(&sig_bytes)
            })
            .map_err(|e| HsmError::SigningFailed(format!("Invalid signature: {}", e)))
    }

    /// Parse DER-encoded ECDSA signature
    /// This is a simplified parser for SEQUENCE { INTEGER r, INTEGER s }
    /// 
    /// # Security Note
    /// This is a basic implementation for demonstration. For production use,
    /// consider using a well-tested DER parsing library like:
    /// - `der` crate (part of RustCrypto)
    /// - `simple_asn1` crate
    /// - `yasna` crate
    /// 
    /// These libraries provide proper validation and error handling for
    /// security-critical signature data.
    fn parse_der_signature(der: &[u8]) -> Result<Signature, bitcell_crypto::CryptoError> {
        // Validate minimum length and SEQUENCE tag
        if der.len() < 8 || der[0] != 0x30 {
            return Err(bitcell_crypto::CryptoError::InvalidSignature);
        }

        // Validate sequence length
        let seq_len = der[1] as usize;
        if 2 + seq_len != der.len() {
            return Err(bitcell_crypto::CryptoError::InvalidSignature);
        }

        let mut pos = 2;
        
        // Parse r - INTEGER tag
        if pos >= der.len() || der[pos] != 0x02 {
            return Err(bitcell_crypto::CryptoError::InvalidSignature);
        }
        pos += 1;
        
        // Validate r length is within bounds
        if pos >= der.len() {
            return Err(bitcell_crypto::CryptoError::InvalidSignature);
        }
        let r_len = der[pos] as usize;
        pos += 1;
        
        if r_len == 0 || r_len > 33 || pos + r_len > der.len() {
            return Err(bitcell_crypto::CryptoError::InvalidSignature);
        }
        
        let r_bytes = &der[pos..pos + r_len];
        pos += r_len;
        
        // Parse s - INTEGER tag
        if pos >= der.len() || der[pos] != 0x02 {
            return Err(bitcell_crypto::CryptoError::InvalidSignature);
        }
        pos += 1;
        
        // Validate s length is within bounds
        if pos >= der.len() {
            return Err(bitcell_crypto::CryptoError::InvalidSignature);
        }
        let s_len = der[pos] as usize;
        pos += 1;
        
        if s_len == 0 || s_len > 33 || pos + s_len > der.len() {
            return Err(bitcell_crypto::CryptoError::InvalidSignature);
        }
        
        let s_bytes = &der[pos..pos + s_len];
        
        // Combine r and s into 64-byte signature
        let mut sig = vec![0u8; 64];
        
        // Copy r (skip leading zero byte if present, pad with zeros if needed)
        let r_start = if r_bytes.len() > 32 { r_bytes.len() - 32 } else { 0 };
        let r_pad = if r_bytes.len() < 32 { 32 - r_bytes.len() } else { 0 };
        sig[r_pad..32].copy_from_slice(&r_bytes[r_start..]);
        
        // Copy s (padding with zeros if needed)
        let s_start = if s_bytes.len() > 32 { s_bytes.len() - 32 } else { 0 };
        let s_pad = if s_bytes.len() < 32 { 32 - s_bytes.len() } else { 0 };
        sig[32 + s_pad..64].copy_from_slice(&s_bytes[s_start..]);
        
        Signature::from_bytes(&sig)
    }

    /// List all keys in AWS KMS
    async fn list_aws_keys(&self) -> HsmResult<Vec<String>> {
        let mut key_names = Vec::new();
        let mut marker = None;

        loop {
            let mut request = self.client.list_aliases();
            
            if let Some(m) = marker {
                request = request.marker(m);
            }

            let response = request
                .send()
                .await
                .map_err(|e| HsmError::InternalError(format!("Failed to list keys: {}", e)))?;

            if let Some(aliases) = response.aliases {
                for alias in aliases {
                    if let Some(alias_name) = alias.alias_name {
                        // Remove "alias/" prefix
                        if let Some(name) = alias_name.strip_prefix("alias/") {
                            // Skip AWS managed keys
                            if !name.starts_with("aws/") {
                                key_names.push(name.to_string());
                            }
                        }
                    }
                }
            }

            if response.truncated == Some(true) && response.next_marker.is_some() {
                marker = response.next_marker;
            } else {
                break;
            }
        }

        Ok(key_names)
    }
}

#[async_trait]
impl HsmBackend for AwsHsmBackend {
    fn provider(&self) -> HsmProvider {
        HsmProvider::AwsCloudHsm
    }

    async fn is_available(&self) -> bool {
        // Try to list aliases to verify connectivity
        self.client
            .list_aliases()
            .limit(1)
            .send()
            .await
            .is_ok()
    }

    async fn get_public_key(&self, key_name: &str) -> HsmResult<PublicKey> {
        self.get_aws_public_key(key_name).await
    }

    async fn sign(&self, key_name: &str, hash: &Hash256) -> HsmResult<Signature> {
        self.sign_aws(key_name, hash).await
    }

    async fn generate_key(&self, key_name: &str) -> HsmResult<PublicKey> {
        // Check if key already exists
        if self.find_key_id(key_name).await.is_ok() {
            return Err(HsmError::InternalError(format!(
                "Key '{}' already exists",
                key_name
            )));
        }

        self.create_aws_key(key_name).await
    }

    async fn list_keys(&self) -> HsmResult<Vec<String>> {
        self.list_aws_keys().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_extraction() {
        assert_eq!(
            AwsHsmBackend::extract_region("kms.us-east-1.amazonaws.com"),
            Some("us-east-1".to_string())
        );
        assert_eq!(
            AwsHsmBackend::extract_region("kms.eu-west-1.amazonaws.com"),
            Some("eu-west-1".to_string())
        );
        assert_eq!(
            AwsHsmBackend::extract_region("invalid-endpoint"),
            None
        );
    }

    #[tokio::test]
    async fn test_aws_config_validation() {
        // Test missing access key
        let mut config = HsmConfig::aws("kms.us-east-1.amazonaws.com", "", "secret", "test-key");
        config.credentials.access_key = None;
        
        let result = AwsHsmBackend::connect(&config).await;
        assert!(matches!(result, Err(HsmError::InvalidConfig(_))));
    }

    #[tokio::test]
    async fn test_aws_config_missing_secret() {
        // Test missing secret key
        let mut config = HsmConfig::aws("kms.us-east-1.amazonaws.com", "access", "", "test-key");
        config.credentials.secret_key = None;
        
        let result = AwsHsmBackend::connect(&config).await;
        assert!(matches!(result, Err(HsmError::InvalidConfig(_))));
    }
}
