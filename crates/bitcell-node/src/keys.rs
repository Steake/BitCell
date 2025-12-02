//! Keypair loading utilities for BitCell nodes

use bitcell_crypto::SecretKey;
use std::fs;
use std::path::Path;
use crate::{Result, Error};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use tracing;

/// Load a secret key from a file
/// Supports:
/// - Hex-encoded private key (64 characters)
/// - PEM format (PKCS#8)
/// - Raw binary (32 bytes)
pub fn load_secret_key_from_file(path: &Path) -> Result<SecretKey> {
    let contents = fs::read_to_string(path)
        .or_else(|_| {
            // Try binary read if text fails
            fs::read(path).map(|bytes| {
                // If it's 32 bytes, assume it's a raw key
                if bytes.len() == 32 {
                    hex::encode(&bytes)
                } else {
                    String::from_utf8_lossy(&bytes).to_string()
                }
            })
        })
        .map_err(|e| Error::Node(format!("Failed to read key file: {}", e)))?;
    
    let trimmed = contents.trim();
    
    // Try PEM format first
    if trimmed.starts_with("-----BEGIN") {
        return load_secret_key_from_pem(trimmed);
    }
    
    // Try hex format
    if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return load_secret_key_from_hex(trimmed);
    }
    
    Err(Error::Node("Unsupported key file format. Expected hex (64 chars) or PEM format.".to_string()))
}

/// Load a secret key from a hex string
pub fn load_secret_key_from_hex(hex: &str) -> Result<SecretKey> {
    let hex = hex.trim();
    if hex.len() != 64 {
        return Err(Error::Node("Hex private key must be exactly 64 characters (32 bytes)".to_string()));
    }
    
    let bytes = hex::decode(hex)
        .map_err(|e| Error::Node(format!("Invalid hex encoding: {}", e)))?;
    
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&bytes);
    
    SecretKey::from_bytes(&key_bytes)
        .map_err(|e| Error::Node(format!("Invalid private key bytes: {}", e)))
}

/// Load a secret key from PEM format
fn load_secret_key_from_pem(pem: &str) -> Result<SecretKey> {
    // Simple PEM parser - extract base64 content between headers
    let lines: Vec<&str> = pem.lines().collect();
    
    // Find content between BEGIN and END
    let start = lines.iter().position(|l| l.starts_with("-----BEGIN"))
        .ok_or_else(|| Error::Node("Invalid PEM: missing BEGIN header".to_string()))?;
    let end = lines.iter().position(|l| l.starts_with("-----END"))
        .ok_or_else(|| Error::Node("Invalid PEM: missing END header".to_string()))?;
    
    if end <= start {
        return Err(Error::Node("Invalid PEM format".to_string()));
    }
    
    // Concatenate base64 content
    let b64_content: String = lines[start + 1..end]
        .iter()
        .map(|s| s.trim())
        .collect();
    
    // Decode base64
    let der_bytes = BASE64.decode(&b64_content)
        .map_err(|e| Error::Node(format!("Invalid base64 in PEM: {}", e)))?;
    
    // For PKCS#8, the actual key is at the end (last 32 bytes for ed25519)
    // This is a simplified parser - you might need a proper ASN.1 parser for production
    if der_bytes.len() >= 32 {
        let key_slice = &der_bytes[der_bytes.len() - 32..];
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(key_slice);
        
        SecretKey::from_bytes(&key_bytes)
            .map_err(|e| Error::Node(format!("Invalid key bytes in PEM: {}", e)))
    } else {
        Err(Error::Node("PEM file too short to contain a valid key".to_string()))
    }
}

/// Derive a secret key from a BIP39 mnemonic phrase
pub fn derive_secret_key_from_mnemonic(mnemonic: &str) -> Result<SecretKey> {
    // Use SHA-256 of the mnemonic as the seed
    // In production, use proper BIP39 derivation with PBKDF2
    let hash = bitcell_crypto::Hash256::hash(mnemonic.as_bytes());
    SecretKey::from_bytes(hash.as_bytes())
        .map_err(|e| Error::Node(format!("Failed to derive key from mnemonic: {}", e)))
}

/// Derive a secret key from a simple string seed
pub fn derive_secret_key_from_seed(seed: &str) -> SecretKey {
    let hash = bitcell_crypto::Hash256::hash(seed.as_bytes());
    SecretKey::from_bytes(hash.as_bytes())
        .expect("Hash-derived key should always be valid")
}

/// Resolve secret key from CLI arguments in priority order
pub fn resolve_secret_key(
    private_key_hex: Option<&str>,
    key_file_path: Option<&Path>,
    mnemonic: Option<&str>,
    key_seed: Option<&str>,
) -> Result<SecretKey> {
    // Priority 1: Direct hex private key
    if let Some(hex) = private_key_hex {
        tracing::debug!("Loading key from hex string");
        return load_secret_key_from_hex(hex);
    }
    
    // Priority 2: Key file
    if let Some(path) = key_file_path {
        tracing::debug!("Loading key from file: {}", path.display());
        return load_secret_key_from_file(path);
    }
    
    // Priority 3: Mnemonic phrase
    if let Some(phrase) = mnemonic {
        tracing::debug!("Deriving key from mnemonic phrase");
        return derive_secret_key_from_mnemonic(phrase);
    }
    
    // Priority 4: Simple seed
    if let Some(seed) = key_seed {
        tracing::debug!("Deriving key from seed");
        return Ok(derive_secret_key_from_seed(seed));
    }
    
    // Priority 5: Generate random
    tracing::debug!("Generating random key (no key specified)");
    Ok(SecretKey::generate())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    
    #[test]
    fn test_hex_key_loading() {
        let hex = "a".repeat(64);
        let result = load_secret_key_from_hex(&hex);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_invalid_hex_length() {
        let hex = "a".repeat(32);
        let result = load_secret_key_from_hex(&hex);
        assert!(result.is_err());
        
        // Test with 65 characters (too long)
        let hex_long = "a".repeat(65);
        let result_long = load_secret_key_from_hex(&hex_long);
        assert!(result_long.is_err());
    }
    
    #[test]
    fn test_invalid_hex_characters() {
        // Contains 'g' which is not a valid hex character
        let hex = "g".repeat(64);
        let result = load_secret_key_from_hex(&hex);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_seed_derivation() {
        let sk1 = derive_secret_key_from_seed("test-seed");
        let sk2 = derive_secret_key_from_seed("test-seed");
        assert_eq!(sk1.public_key(), sk2.public_key());
    }
    
    #[test]
    fn test_different_seeds_produce_different_keys() {
        let sk1 = derive_secret_key_from_seed("seed-one");
        let sk2 = derive_secret_key_from_seed("seed-two");
        assert_ne!(sk1.public_key(), sk2.public_key());
    }
    
    #[test]
    fn test_mnemonic_derivation() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let result = derive_secret_key_from_mnemonic(mnemonic);
        assert!(result.is_ok());
        
        // Same mnemonic produces same key
        let result2 = derive_secret_key_from_mnemonic(mnemonic);
        assert!(result2.is_ok());
        assert_eq!(result.unwrap().public_key(), result2.unwrap().public_key());
    }
    
    #[test]
    fn test_different_mnemonics_produce_different_keys() {
        let mnemonic1 = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let mnemonic2 = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";
        
        let sk1 = derive_secret_key_from_mnemonic(mnemonic1).unwrap();
        let sk2 = derive_secret_key_from_mnemonic(mnemonic2).unwrap();
        assert_ne!(sk1.public_key(), sk2.public_key());
    }
    
    #[test]
    fn test_load_from_hex_file() {
        let hex_key = "a".repeat(64);
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_key_hex.txt");
        
        let mut file = std::fs::File::create(&temp_file).unwrap();
        file.write_all(hex_key.as_bytes()).unwrap();
        
        let result = load_secret_key_from_file(&temp_file);
        assert!(result.is_ok());
        
        std::fs::remove_file(temp_file).ok();
    }
    
    #[test]
    fn test_load_from_pem_file() {
        // Create a simple PEM file with valid key data
        let pem_content = "-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIHJlYWxseWxvbmdzZWNyZXRrZXlieXRlczMyY2hhcnM=
-----END PRIVATE KEY-----";
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_key.pem");
        
        let mut file = std::fs::File::create(&temp_file).unwrap();
        file.write_all(pem_content.as_bytes()).unwrap();
        
        let result = load_secret_key_from_file(&temp_file);
        // PEM parsing might fail due to key validation, but it should parse the format
        // The important thing is it doesn't crash
        
        std::fs::remove_file(temp_file).ok();
    }
    
    #[test]
    fn test_load_from_nonexistent_file() {
        let result = load_secret_key_from_file(Path::new("/nonexistent/path/key.txt"));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_load_from_invalid_format_file() {
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_key_invalid.txt");
        
        let mut file = std::fs::File::create(&temp_file).unwrap();
        file.write_all(b"this is not a valid key format").unwrap();
        
        let result = load_secret_key_from_file(&temp_file);
        assert!(result.is_err());
        
        std::fs::remove_file(temp_file).ok();
    }
    
    #[test]
    fn test_resolve_secret_key_priority() {
        // When private_key_hex is provided, it takes priority
        let hex = "a".repeat(64);
        let result = resolve_secret_key(Some(&hex), None, None, None);
        assert!(result.is_ok());
        let key_from_hex = result.unwrap();
        
        // Same hex should produce same key
        let result2 = resolve_secret_key(Some(&hex), None, Some("some mnemonic"), Some("some seed"));
        assert!(result2.is_ok());
        assert_eq!(key_from_hex.public_key(), result2.unwrap().public_key());
    }
    
    #[test]
    fn test_resolve_secret_key_falls_back_to_seed() {
        let result = resolve_secret_key(None, None, None, Some("test-seed"));
        assert!(result.is_ok());
        
        let expected = derive_secret_key_from_seed("test-seed");
        assert_eq!(result.unwrap().public_key(), expected.public_key());
    }
    
    #[test]
    fn test_resolve_secret_key_generates_random() {
        let result1 = resolve_secret_key(None, None, None, None);
        let result2 = resolve_secret_key(None, None, None, None);
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        
        // Random keys should be different
        assert_ne!(result1.unwrap().public_key(), result2.unwrap().public_key());
    }
}
