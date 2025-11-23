//! ECDSA signatures using secp256k1
//!
//! Primary signature scheme for transaction and block signing.

use crate::{Error, Result};
use k256::ecdsa::{
    signature::{Signer, Verifier},
    Signature as K256Signature, SigningKey, VerifyingKey,
};
use rand::rngs::OsRng;

use std::fmt;

/// ECDSA public key (33 bytes compressed)
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PublicKey([u8; 33]);

impl serde::Serialize for PublicKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for PublicKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let bytes = <Vec<u8>>::deserialize(deserializer)?;
        if bytes.len() != 33 {
            return Err(serde::de::Error::custom("Invalid public key length"));
        }
        let mut array = [0u8; 33];
        array.copy_from_slice(&bytes);
        Ok(PublicKey(array))
    }
}

impl PublicKey {
    /// Create from compressed bytes
    pub fn from_bytes(bytes: [u8; 33]) -> Result<Self> {
        // Validate it's a valid point
        VerifyingKey::from_sec1_bytes(&bytes)
            .map_err(|_| Error::InvalidPublicKey)?;
        Ok(Self(bytes))
    }

    /// Get bytes
    pub fn as_bytes(&self) -> &[u8; 33] {
        &self.0
    }

    /// Derive miner ID (hash of public key)
    pub fn miner_id(&self) -> crate::Hash256 {
        crate::Hash256::hash(&self.0)
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PublicKey({})", hex::encode(&self.0[..8]))
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

/// ECDSA secret key
pub struct SecretKey(SigningKey);

impl SecretKey {
    /// Generate a new random key pair
    pub fn generate() -> Self {
        let signing_key = SigningKey::random(&mut OsRng);
        Self(signing_key)
    }

    /// Create from bytes (32 bytes)
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self> {
        SigningKey::from_bytes(bytes.into())
            .map(Self)
            .map_err(|_| Error::InvalidSecretKey)
    }

    /// Get the public key
    pub fn public_key(&self) -> PublicKey {
        let verifying_key = self.0.verifying_key();
        // Safe: compressed encoding always produces 33 bytes for secp256k1
        let bytes: [u8; 33] = verifying_key.to_encoded_point(true).as_bytes()
            .try_into()
            .expect("secp256k1 compressed public key is always 33 bytes");
        PublicKey(bytes)
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Signature {
        let sig: K256Signature = self.0.sign(message);
        Signature(sig.to_bytes().into())
    }

    /// Export as bytes (for storage - handle carefully!)
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes().into()
    }
}

/// ECDSA signature (64 bytes)
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Signature([u8; 64]);

impl serde::Serialize for Signature {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for Signature {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let bytes = <Vec<u8>>::deserialize(deserializer)?;
        if bytes.len() != 64 {
            return Err(serde::de::Error::custom("Invalid signature length"));
        }
        let mut array = [0u8; 64];
        array.copy_from_slice(&bytes);
        Ok(Signature(array))
    }
}

impl Signature {
    /// Create from bytes
    pub fn from_bytes(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    /// Get bytes
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }

    /// Verify signature
    pub fn verify(&self, public_key: &PublicKey, message: &[u8]) -> Result<()> {
        let verifying_key = VerifyingKey::from_sec1_bytes(public_key.as_bytes())
            .map_err(|_| Error::InvalidPublicKey)?;

        let signature = K256Signature::from_bytes(&self.0.into())
            .map_err(|_| Error::InvalidSignature)?;

        verifying_key
            .verify(message, &signature)
            .map_err(|_| Error::InvalidSignature)
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature({})", hex::encode(&self.0[..8]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        
        // Should be able to derive miner ID
        let _miner_id = pk.miner_id();
    }

    #[test]
    fn test_sign_and_verify() {
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        let message = b"test message";

        let sig = sk.sign(message);
        assert!(sig.verify(&pk, message).is_ok());
    }

    #[test]
    fn test_verify_wrong_message() {
        let sk = SecretKey::generate();
        let pk = sk.public_key();

        let sig = sk.sign(b"original");
        assert!(sig.verify(&pk, b"tampered").is_err());
    }

    #[test]
    fn test_verify_wrong_key() {
        let sk1 = SecretKey::generate();
        let sk2 = SecretKey::generate();
        let pk2 = sk2.public_key();

        let sig = sk1.sign(b"message");
        assert!(sig.verify(&pk2, b"message").is_err());
    }

    #[test]
    fn test_key_serialization() {
        let sk = SecretKey::generate();
        let bytes = sk.to_bytes();
        let sk2 = SecretKey::from_bytes(&bytes).unwrap();
        
        assert_eq!(sk.public_key(), sk2.public_key());
    }
}
