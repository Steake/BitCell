//! Key management for Groth16 proving and verification keys
//!
//! This module provides utilities for:
//! - Serializing and deserializing keys
//! - Loading keys from ceremony outputs
//! - Verifying key integrity
//! - Managing key file paths

use ark_bn254::Bn254;
use ark_groth16::{ProvingKey, VerifyingKey};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use sha2::{Digest, Sha256};

use crate::{Error, Result};

/// Key type for different circuits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    /// BattleCircuit keys
    Battle,
    /// StateCircuit keys
    State,
}

impl KeyType {
    /// Get the directory name for this key type
    pub fn dir_name(&self) -> &'static str {
        match self {
            KeyType::Battle => "battle",
            KeyType::State => "state",
        }
    }
}

/// Load proving key from file
///
/// # Arguments
/// * `path` - Path to the proving key file
///
/// # Returns
/// The deserialized proving key
///
/// # Example
/// ```no_run
/// use bitcell_zkp::key_management::{load_proving_key, KeyType};
///
/// let pk = load_proving_key("keys/battle/proving_key.bin")?;
/// # Ok::<(), bitcell_zkp::Error>(())
/// ```
pub fn load_proving_key<P: AsRef<Path>>(path: P) -> Result<ProvingKey<Bn254>> {
    let file = File::open(path.as_ref())
        .map_err(|e| Error::Setup(format!("Failed to open proving key file: {}", e)))?;
    
    let mut reader = BufReader::new(file);
    let pk = ProvingKey::<Bn254>::deserialize_compressed(&mut reader)
        .map_err(|e| Error::Setup(format!("Failed to deserialize proving key: {}", e)))?;
    
    Ok(pk)
}

/// Load verification key from file
///
/// # Arguments
/// * `path` - Path to the verification key file
///
/// # Returns
/// The deserialized verification key
///
/// # Example
/// ```no_run
/// use bitcell_zkp::key_management::{load_verification_key, KeyType};
///
/// let vk = load_verification_key("keys/battle/verification_key.bin")?;
/// # Ok::<(), bitcell_zkp::Error>(())
/// ```
pub fn load_verification_key<P: AsRef<Path>>(path: P) -> Result<VerifyingKey<Bn254>> {
    let file = File::open(path.as_ref())
        .map_err(|e| Error::Setup(format!("Failed to open verification key file: {}", e)))?;
    
    let mut reader = BufReader::new(file);
    let vk = VerifyingKey::<Bn254>::deserialize_compressed(&mut reader)
        .map_err(|e| Error::Setup(format!("Failed to deserialize verification key: {}", e)))?;
    
    Ok(vk)
}

/// Save proving key to file
///
/// # Arguments
/// * `pk` - The proving key to save
/// * `path` - Path where to save the key
pub fn save_proving_key<P: AsRef<Path>>(pk: &ProvingKey<Bn254>, path: P) -> Result<()> {
    let file = File::create(path.as_ref())
        .map_err(|e| Error::Setup(format!("Failed to create proving key file: {}", e)))?;
    
    let mut writer = BufWriter::new(file);
    pk.serialize_compressed(&mut writer)
        .map_err(|e| Error::Setup(format!("Failed to serialize proving key: {}", e)))?;
    
    writer.flush()
        .map_err(|e| Error::Setup(format!("Failed to flush proving key file: {}", e)))?;
    
    Ok(())
}

/// Save verification key to file
///
/// # Arguments
/// * `vk` - The verification key to save
/// * `path` - Path where to save the key
pub fn save_verification_key<P: AsRef<Path>>(vk: &VerifyingKey<Bn254>, path: P) -> Result<()> {
    let file = File::create(path.as_ref())
        .map_err(|e| Error::Setup(format!("Failed to create verification key file: {}", e)))?;
    
    let mut writer = BufWriter::new(file);
    vk.serialize_compressed(&mut writer)
        .map_err(|e| Error::Setup(format!("Failed to serialize verification key: {}", e)))?;
    
    writer.flush()
        .map_err(|e| Error::Setup(format!("Failed to flush verification key file: {}", e)))?;
    
    Ok(())
}

/// Compute SHA256 hash of a file
///
/// # Arguments
/// * `path` - Path to the file to hash
///
/// # Returns
/// Hex-encoded SHA256 hash
pub fn compute_file_hash<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = File::open(path.as_ref())
        .map_err(|e| Error::Setup(format!("Failed to open file for hashing: {}", e)))?;
    
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 8192];
    
    loop {
        let n = file.read(&mut buffer)
            .map_err(|e| Error::Setup(format!("Failed to read file: {}", e)))?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    
    Ok(format!("{:x}", hasher.finalize()))
}

/// Verify proving key integrity against known hash
///
/// # Arguments
/// * `path` - Path to the proving key file
/// * `expected_hash` - Expected SHA256 hash (hex-encoded)
///
/// # Returns
/// `Ok(())` if hash matches, `Err` otherwise
pub fn verify_proving_key_hash<P: AsRef<Path>>(path: P, expected_hash: &str) -> Result<()> {
    let actual_hash = compute_file_hash(path)?;
    
    if actual_hash.to_lowercase() != expected_hash.to_lowercase() {
        return Err(Error::Setup(format!(
            "Proving key hash mismatch. Expected: {}, Got: {}",
            expected_hash, actual_hash
        )));
    }
    
    Ok(())
}

/// Verify verification key integrity against known hash
///
/// # Arguments
/// * `path` - Path to the verification key file
/// * `expected_hash` - Expected SHA256 hash (hex-encoded)
///
/// # Returns
/// `Ok(())` if hash matches, `Err` otherwise
pub fn verify_verification_key_hash<P: AsRef<Path>>(path: P, expected_hash: &str) -> Result<()> {
    let actual_hash = compute_file_hash(path)?;
    
    if actual_hash.to_lowercase() != expected_hash.to_lowercase() {
        return Err(Error::Setup(format!(
            "Verification key hash mismatch. Expected: {}, Got: {}",
            expected_hash, actual_hash
        )));
    }
    
    Ok(())
}

/// Get default key paths for a circuit type
///
/// # Arguments
/// * `key_type` - The type of circuit
///
/// # Returns
/// Tuple of (proving_key_path, verification_key_path)
pub fn default_key_paths(key_type: KeyType) -> (String, String) {
    let dir = key_type.dir_name();
    (
        format!("keys/{}/proving_key.bin", dir),
        format!("keys/{}/verification_key.bin", dir),
    )
}

/// Key metadata for verification
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyMetadata {
    /// Circuit type
    pub circuit: String,
    /// Key version
    pub version: String,
    /// SHA256 hash of proving key
    pub proving_key_hash: String,
    /// SHA256 hash of verification key
    pub verification_key_hash: String,
    /// Size of proving key in bytes
    pub proving_key_size: u64,
    /// Size of verification key in bytes
    pub verification_key_size: u64,
    /// Number of ceremony participants
    pub num_participants: usize,
    /// Ceremony completion date
    pub ceremony_date: String,
    /// IPFS CID for proving key (optional)
    pub ipfs_proving_key: Option<String>,
    /// IPFS CID for verification key (optional)
    pub ipfs_verification_key: Option<String>,
    /// Arweave transaction ID for proving key (optional)
    pub arweave_proving_key: Option<String>,
    /// Arweave transaction ID for verification key (optional)
    pub arweave_verification_key: Option<String>,
    /// Ceremony status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Additional notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Circuit-specific parameters (stored as generic JSON value)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub circuit_parameters: Option<serde_json::Value>,
}

impl KeyMetadata {
    /// Load metadata from JSON file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())
            .map_err(|e| Error::Setup(format!("Failed to open metadata file: {}", e)))?;
        
        let metadata: KeyMetadata = serde_json::from_reader(BufReader::new(file))
            .map_err(|e| Error::Setup(format!("Failed to parse metadata: {}", e)))?;
        
        Ok(metadata)
    }
    
    /// Save metadata to JSON file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file = File::create(path.as_ref())
            .map_err(|e| Error::Setup(format!("Failed to create metadata file: {}", e)))?;
        
        serde_json::to_writer_pretty(BufWriter::new(file), self)
            .map_err(|e| Error::Setup(format!("Failed to write metadata: {}", e)))?;
        
        Ok(())
    }
    
    /// Verify that keys match the metadata hashes
    pub fn verify_keys(&self, pk_path: &str, vk_path: &str) -> Result<()> {
        verify_proving_key_hash(pk_path, &self.proving_key_hash)?;
        verify_verification_key_hash(vk_path, &self.verification_key_hash)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_groth16::Groth16;
    use ark_relations::r1cs::ConstraintSynthesizer;
    use ark_snark::SNARK;
    use std::fs;
    use tempfile::TempDir;
    
    // Simple test circuit for key serialization tests
    #[derive(Clone)]
    struct TestCircuit;
    
    impl ConstraintSynthesizer<ark_bn254::Fr> for TestCircuit {
        fn generate_constraints(
            self,
            _cs: ark_relations::r1cs::ConstraintSystemRef<ark_bn254::Fr>,
        ) -> std::result::Result<(), ark_relations::r1cs::SynthesisError> {
            Ok(())
        }
    }
    
    #[test]
    fn test_save_and_load_keys() {
        let temp_dir = TempDir::new().unwrap();
        let pk_path = temp_dir.path().join("test_pk.bin");
        let vk_path = temp_dir.path().join("test_vk.bin");
        
        // Generate test keys
        let rng = &mut ark_std::rand::thread_rng();
        let circuit = TestCircuit;
        let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(circuit, rng).unwrap();
        
        // Save keys
        save_proving_key(&pk, &pk_path).unwrap();
        save_verification_key(&vk, &vk_path).unwrap();
        
        // Verify files exist
        assert!(pk_path.exists());
        assert!(vk_path.exists());
        
        // Load keys
        let loaded_pk = load_proving_key(&pk_path).unwrap();
        let loaded_vk = load_verification_key(&vk_path).unwrap();
        
        // Verify keys are equivalent (by serializing and comparing)
        let mut pk_bytes = Vec::new();
        pk.serialize_compressed(&mut pk_bytes).unwrap();
        let mut loaded_pk_bytes = Vec::new();
        loaded_pk.serialize_compressed(&mut loaded_pk_bytes).unwrap();
        assert_eq!(pk_bytes, loaded_pk_bytes);
        
        let mut vk_bytes = Vec::new();
        vk.serialize_compressed(&mut vk_bytes).unwrap();
        let mut loaded_vk_bytes = Vec::new();
        loaded_vk.serialize_compressed(&mut loaded_vk_bytes).unwrap();
        assert_eq!(vk_bytes, loaded_vk_bytes);
    }
    
    #[test]
    fn test_compute_file_hash() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        
        // Write test data
        fs::write(&test_file, b"Hello, BitCell!").unwrap();
        
        // Compute hash
        let hash = compute_file_hash(&test_file).unwrap();
        
        // Verify hash is hex string of correct length (64 chars for SHA256)
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
    
    #[test]
    fn test_verify_key_hash() {
        let temp_dir = TempDir::new().unwrap();
        let pk_path = temp_dir.path().join("test_pk.bin");
        
        // Generate and save test key
        let rng = &mut ark_std::rand::thread_rng();
        let circuit = TestCircuit;
        let (pk, _) = Groth16::<Bn254>::circuit_specific_setup(circuit, rng).unwrap();
        save_proving_key(&pk, &pk_path).unwrap();
        
        // Compute expected hash
        let expected_hash = compute_file_hash(&pk_path).unwrap();
        
        // Verify should succeed
        assert!(verify_proving_key_hash(&pk_path, &expected_hash).is_ok());
        
        // Verify with wrong hash should fail
        let wrong_hash = "0000000000000000000000000000000000000000000000000000000000000000";
        assert!(verify_proving_key_hash(&pk_path, wrong_hash).is_err());
    }
    
    #[test]
    fn test_key_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let metadata_path = temp_dir.path().join("metadata.json");
        
        // Create test metadata
        let metadata = KeyMetadata {
            circuit: "TestCircuit".to_string(),
            version: "1.0".to_string(),
            proving_key_hash: "abc123".to_string(),
            verification_key_hash: "def456".to_string(),
            proving_key_size: 1000,
            verification_key_size: 100,
            num_participants: 5,
            ceremony_date: "2026-03-15".to_string(),
            ipfs_proving_key: Some("QmTest123".to_string()),
            ipfs_verification_key: Some("QmTest456".to_string()),
            arweave_proving_key: None,
            arweave_verification_key: None,
            status: Some("complete".to_string()),
            notes: Some("Test metadata".to_string()),
            circuit_parameters: Some(serde_json::json!({
                "test_param": "test_value"
            })),
        };
        
        // Save metadata
        metadata.save(&metadata_path).unwrap();
        
        // Load metadata
        let loaded = KeyMetadata::load(&metadata_path).unwrap();
        
        // Verify fields
        assert_eq!(loaded.circuit, "TestCircuit");
        assert_eq!(loaded.num_participants, 5);
        assert_eq!(loaded.ipfs_proving_key, Some("QmTest123".to_string()));
        assert_eq!(loaded.status, Some("complete".to_string()));
        assert_eq!(loaded.notes, Some("Test metadata".to_string()));
        assert!(loaded.circuit_parameters.is_some());
    }
    
    #[test]
    fn test_default_key_paths() {
        let (pk_path, vk_path) = default_key_paths(KeyType::Battle);
        assert_eq!(pk_path, "keys/battle/proving_key.bin");
        assert_eq!(vk_path, "keys/battle/verification_key.bin");
        
        let (pk_path, vk_path) = default_key_paths(KeyType::State);
        assert_eq!(pk_path, "keys/state/proving_key.bin");
        assert_eq!(vk_path, "keys/state/verification_key.bin");
    }
}
