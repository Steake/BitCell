//! BIP39 Mnemonic Seed Phrase Management
//!
//! Provides secure generation and handling of BIP39 mnemonic phrases
//! for wallet creation and recovery.

use crate::{Error, Result};
use bip39::{Language, Mnemonic as Bip39Mnemonic, MnemonicType, Seed};
use zeroize::Zeroize;

/// Mnemonic word count options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordCount {
    /// 12 words (128 bits entropy)
    Words12,
    /// 18 words (192 bits entropy)
    Words18,
    /// 24 words (256 bits entropy)
    Words24,
}

impl WordCount {
    fn to_mnemonic_type(self) -> MnemonicType {
        match self {
            WordCount::Words12 => MnemonicType::Words12,
            WordCount::Words18 => MnemonicType::Words18,
            WordCount::Words24 => MnemonicType::Words24,
        }
    }
}

impl Default for WordCount {
    fn default() -> Self {
        WordCount::Words24
    }
}

/// BIP39 mnemonic phrase for wallet generation
#[derive(Clone)]
pub struct Mnemonic {
    inner: Bip39Mnemonic,
}

impl Mnemonic {
    /// Generate a new random mnemonic with the specified word count
    pub fn generate(word_count: WordCount) -> Self {
        let mnemonic = Bip39Mnemonic::new(word_count.to_mnemonic_type(), Language::English);
        Self { inner: mnemonic }
    }

    /// Generate a new mnemonic with default word count (24 words)
    pub fn new() -> Self {
        Self::generate(WordCount::default())
    }

    /// Parse a mnemonic from a phrase string
    pub fn from_phrase(phrase: &str) -> Result<Self> {
        let mnemonic = Bip39Mnemonic::from_phrase(phrase, Language::English)
            .map_err(|e| Error::InvalidMnemonic(e.to_string()))?;
        Ok(Self { inner: mnemonic })
    }

    /// Get the mnemonic phrase as a string
    pub fn phrase(&self) -> &str {
        self.inner.phrase()
    }

    /// Get words as a vector
    pub fn words(&self) -> Vec<&str> {
        self.inner.phrase().split_whitespace().collect()
    }

    /// Get the number of words in the mnemonic
    pub fn word_count(&self) -> usize {
        self.words().len()
    }

    /// Derive a seed from the mnemonic with optional passphrase
    pub fn to_seed(&self, passphrase: &str) -> SeedBytes {
        let seed = Seed::new(&self.inner, passphrase);
        SeedBytes::new(seed.as_bytes().try_into().expect("Seed should be 64 bytes"))
    }

    /// Validate the mnemonic phrase
    pub fn validate(phrase: &str) -> bool {
        Bip39Mnemonic::from_phrase(phrase, Language::English).is_ok()
    }
}

impl Default for Mnemonic {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Don't expose the actual phrase in debug output
        write!(f, "Mnemonic({})", self.word_count())
    }
}

/// Secure container for seed bytes that gets zeroed on drop
#[derive(Clone)]
pub struct SeedBytes {
    bytes: [u8; 64],
}

impl SeedBytes {
    /// Create new seed bytes from an array
    pub fn new(bytes: [u8; 64]) -> Self {
        Self { bytes }
    }

    /// Get the seed bytes
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.bytes
    }

    /// Get the first 32 bytes (for key derivation)
    pub fn master_key_bytes(&self) -> &[u8] {
        &self.bytes[..32]
    }

    /// Get the last 32 bytes (for chain code)
    pub fn chain_code_bytes(&self) -> &[u8] {
        &self.bytes[32..]
    }
}

impl Drop for SeedBytes {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

impl Zeroize for SeedBytes {
    fn zeroize(&mut self) {
        self.bytes.zeroize();
    }
}

impl std::fmt::Debug for SeedBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SeedBytes([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mnemonic_12_words() {
        let mnemonic = Mnemonic::generate(WordCount::Words12);
        assert_eq!(mnemonic.word_count(), 12);
    }

    #[test]
    fn test_generate_mnemonic_18_words() {
        let mnemonic = Mnemonic::generate(WordCount::Words18);
        assert_eq!(mnemonic.word_count(), 18);
    }

    #[test]
    fn test_generate_mnemonic_24_words() {
        let mnemonic = Mnemonic::generate(WordCount::Words24);
        assert_eq!(mnemonic.word_count(), 24);
    }

    #[test]
    fn test_default_mnemonic() {
        let mnemonic = Mnemonic::new();
        assert_eq!(mnemonic.word_count(), 24);
    }

    #[test]
    fn test_mnemonic_from_phrase() {
        let mnemonic1 = Mnemonic::new();
        let phrase = mnemonic1.phrase().to_string();
        let mnemonic2 = Mnemonic::from_phrase(&phrase).unwrap();
        assert_eq!(mnemonic1.phrase(), mnemonic2.phrase());
    }

    #[test]
    fn test_invalid_mnemonic_phrase() {
        let result = Mnemonic::from_phrase("invalid mnemonic phrase");
        assert!(result.is_err());
    }

    #[test]
    fn test_mnemonic_validation() {
        let mnemonic = Mnemonic::new();
        assert!(Mnemonic::validate(mnemonic.phrase()));
        assert!(!Mnemonic::validate("invalid phrase here"));
    }

    #[test]
    fn test_seed_derivation() {
        let mnemonic = Mnemonic::new();
        let seed1 = mnemonic.to_seed("");
        let seed2 = mnemonic.to_seed("");
        // Same mnemonic and passphrase should produce same seed
        assert_eq!(seed1.as_bytes(), seed2.as_bytes());
    }

    #[test]
    fn test_seed_with_passphrase() {
        let mnemonic = Mnemonic::new();
        let seed1 = mnemonic.to_seed("");
        let seed2 = mnemonic.to_seed("password");
        // Different passphrase should produce different seed
        assert_ne!(seed1.as_bytes(), seed2.as_bytes());
    }

    #[test]
    fn test_seed_bytes_length() {
        let mnemonic = Mnemonic::new();
        let seed = mnemonic.to_seed("");
        assert_eq!(seed.as_bytes().len(), 64);
        assert_eq!(seed.master_key_bytes().len(), 32);
        assert_eq!(seed.chain_code_bytes().len(), 32);
    }

    #[test]
    fn test_mnemonic_words() {
        let mnemonic = Mnemonic::generate(WordCount::Words12);
        let words = mnemonic.words();
        assert_eq!(words.len(), 12);
        for word in words {
            assert!(!word.is_empty());
        }
    }
}
