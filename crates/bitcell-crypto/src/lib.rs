//! BitCell Cryptographic Primitives
//!
//! This crate provides all cryptographic building blocks for the BitCell blockchain:
//! - Hash functions (SHA-256, Blake3, Poseidon)
//! - Digital signatures (ECDSA, Ring signatures)
//! - VRF (Verifiable Random Functions)
//! - Commitments (Pedersen)
//! - Merkle trees

pub mod hash;
pub mod signature;
pub mod vrf;
pub mod commitment;
pub mod merkle;
pub mod ring;

pub use hash::{Hash256, Hashable};
pub use signature::{PublicKey, SecretKey, Signature};
pub use vrf::{VrfProof, VrfOutput};
pub use commitment::PedersenCommitment;
pub use merkle::MerkleTree;

/// Standard result type for cryptographic operations
pub type Result<T> = std::result::Result<T, Error>;

/// Cryptographic errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("Invalid proof")]
    InvalidProof,
    
    #[error("Invalid commitment")]
    InvalidCommitment,
    
    #[error("Invalid VRF output")]
    InvalidVrf,
    
    #[error("Invalid public key")]
    InvalidPublicKey,
    
    #[error("Invalid secret key")]
    InvalidSecretKey,
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Ring signature error: {0}")]
    RingSignature(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_imports() {
        // Smoke test to ensure all modules compile
    }
}
