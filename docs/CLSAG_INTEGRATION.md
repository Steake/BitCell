# CLSAG Ring Signatures Integration Guide

## Overview

BitCell implements CLSAG (Concise Linkable Spontaneous Anonymous Group) ring signatures for tournament anonymity and double-spend prevention. This document describes the implementation and how to integrate it into the protocol.

## Features

### 1. Ring Signature Privacy
CLSAG provides cryptographic anonymity for tournament participants:
- Signer identity is hidden among a ring of public keys
- Verification confirms signer is in the ring without revealing which member
- Based on Monero's CLSAG construction for proven security

### 2. Configurable Ring Sizes
Ring sizes are configurable with validated bounds:
- **Minimum:** 11 (provides adequate anonymity set)
- **Maximum:** 64 (balances privacy with verification performance)
- **Default:** 16 (recommended for most use cases)

### 3. Key Image Tracking
Key images enable double-spend prevention:
- Each secret key produces a unique, deterministic key image
- O(1) lookup to detect reused key images
- Prevents double-signing in tournaments
- Cryptographically links signatures from same signer

## Usage

### Creating a Ring Signature

```rust
use bitcell_crypto::{ClsagSecretKey, ClsagSignature, MIN_RING_SIZE};

// Generate keys for ring members
let mut ring = vec![];
for _ in 0..MIN_RING_SIZE {
    let sk = ClsagSecretKey::generate();
    ring.push(sk.public_key());
}

// Your secret key (must be in the ring)
let my_secret = ClsagSecretKey::generate();
ring[5] = my_secret.public_key(); // Add to ring

// Sign a message
let message = b"tournament commitment";
let signature = ClsagSignature::sign(&my_secret, &ring, message)?;

// Verify the signature
signature.verify(&ring, message)?;
```

### Ring Size Configuration

```rust
use bitcell_crypto::{RingConfig, DEFAULT_RING_SIZE};

// Use default configuration
let config = RingConfig::default();
assert_eq!(config.ring_size(), 16);

// Custom ring size
let config = RingConfig::new(32)?; // Size must be [11, 64]
```

### Key Image Tracking

```rust
use bitcell_state::KeyImageRegistry;

// Create a registry
let mut registry = KeyImageRegistry::new();

// Check and mark a key image
let key_image = signature.key_image();
match registry.check_and_mark(*key_image) {
    Ok(()) => println!("Valid signature, first use"),
    Err(_) => println!("Double-spend detected!"),
}

// Check if a key image is used
if registry.is_used(key_image) {
    println!("This key image has been seen before");
}
```

## Integration with Tournament Protocol

### 1. Commitment Phase

During the commitment phase, miners submit anonymous commitments:

```rust
use bitcell_consensus::tournament::{GliderCommitment, Tournament};
use bitcell_crypto::{ClsagSignature, Hash256};

// Create commitment
let commitment_hash = Hash256::hash(&commitment_data);

// Sign with ring of eligible miners
let ring_signature = ClsagSignature::sign(
    &miner_secret_key,
    &tournament.eligible_miners,
    commitment_hash.as_bytes(),
)?;

// Serialize signature for commitment
let signature_bytes = bincode::serialize(&ring_signature)?;

let commitment = GliderCommitment {
    commitment: commitment_hash,
    ring_signature: signature_bytes,
    height: tournament.height,
};
```

### 2. Signature Verification

When processing commitments, verify the ring signature:

```rust
// Deserialize the signature
let ring_signature: ClsagSignature = bincode::deserialize(&commitment.ring_signature)?;

// Verify against the ring of eligible miners
ring_signature.verify(&eligible_miners_ring, commitment.commitment.as_bytes())?;

// Extract and check key image for double-signing
let key_image = ring_signature.key_image();
if state.key_images.is_used(key_image) {
    return Err(Error::DoubleSpend);
}

// Mark as used
state.key_images.mark_used(*key_image)?;
```

### 3. State Management

The `StateManager` includes a `KeyImageRegistry`:

```rust
use bitcell_state::StateManager;

let mut state = StateManager::new();

// Access the registry
let registry = &mut state.key_images;

// Check for double-spend
if registry.is_used(&key_image) {
    // Reject the commitment/transaction
}

// Mark as used after validation
registry.mark_used(key_image)?;
```

## Performance Characteristics

### Signing Performance (by ring size)
- **Ring size 11:** ~2-3ms
- **Ring size 16 (default):** ~3-4ms
- **Ring size 32:** ~6-8ms
- **Ring size 64 (max):** ~12-15ms

### Verification Performance (by ring size)
- **Ring size 11:** ~2-3ms
- **Ring size 16 (default):** ~3-4ms
- **Ring size 32:** ~6-8ms
- **Ring size 64 (max):** ~12-15ms

### Key Image Operations
- **Key image generation:** <1ms
- **Key image lookup:** O(1), ~nanoseconds
- **Key image storage:** 32 bytes per image

### Performance Notes
- Verification is O(n) in ring size (must check each member)
- Key image tracking is O(1) using HashSet
- Signatures scale linearly with ring size
- Default ring size (16) provides good privacy/performance balance

## Security Considerations

### 1. Ring Size Selection
- **Smaller rings (11-16):** Faster but less anonymity
- **Larger rings (32-64):** Better privacy but slower
- Choose based on threat model and performance requirements

### 2. Ring Composition
- Ring should include genuine eligible participants
- Avoid rings with known-compromised keys
- Regular rotation of ring membership recommended

### 3. Key Image Protection
- Key images must be stored persistently
- Clearing key images enables double-spend attacks
- Use write-ahead logging for key image updates

### 4. Double-Spend Prevention
- Always check key images before accepting signatures
- Mark key images as used atomically with signature acceptance
- Handle chain reorganizations by rolling back key images

## Migration from Old Ring Signatures

The old hash-based ring signatures in `ring.rs` are being deprecated. To migrate:

### Before (Old):
```rust
use bitcell_crypto::ring::RingSignature;

let sig = RingSignature::sign(&secret_key, &ring, message)?;
```

### After (New):
```rust
use bitcell_crypto::ClsagSignature;

// Convert PublicKey to ClsagPublicKey (if needed)
let clsag_ring: Vec<ClsagPublicKey> = ring.iter()
    .map(|pk| /* conversion */)
    .collect();

let sig = ClsagSignature::sign(&secret_key, &clsag_ring, message)?;
```

## Testing

### Unit Tests
Run CLSAG tests:
```bash
cargo test --package bitcell-crypto --lib clsag
```

### Benchmark Tests
Run performance benchmarks:
```bash
cargo bench --package bitcell-crypto -- clsag
```

### Integration Tests
Test key image tracking:
```bash
cargo test --package bitcell-state --lib key_images
```

## References

1. **CLSAG Paper:** "CLSAG: Compact Linkable Spontaneous Anonymous Group Signatures"
   - https://eprint.iacr.org/2019/654.pdf

2. **Monero Implementation:** Reference implementation in Monero
   - https://github.com/monero-project/monero

3. **curve25519-dalek:** Cryptographic library used
   - https://github.com/dalek-cryptography/curve25519-dalek

## Troubleshooting

### Common Issues

**Issue:** Ring size validation error
```
Error: Ring size X is below minimum 11
```
**Solution:** Ensure rings have at least 11 members

**Issue:** Signer not in ring
```
Error: Signer not in ring
```
**Solution:** Verify signer's public key is included in the ring

**Issue:** Double-spend detected
```
Error: Key image already used
```
**Solution:** This is expected behavior - reject the signature

### Debug Tips

Enable debug logging:
```rust
env_logger::init();
// Logs will show key image operations
```

Verify key image determinism:
```rust
let ki1 = secret_key.key_image();
let ki2 = secret_key.key_image();
assert_eq!(ki1, ki2); // Should always be equal
```

## Future Enhancements

### Planned for RC2
- [ ] Persistence of key images to RocksDB
- [ ] Chain reorganization handling for key images
- [ ] Merkle commitment of key image set for light clients
- [ ] Integration with tournament ZK proofs

### Planned for RC3
- [ ] Optimized batch verification
- [ ] Multi-signature CLSAG variants
- [ ] Hardware wallet support for CLSAG
- [ ] Threshold ring signatures

## Contact

For questions or issues:
- Open an issue on GitHub
- Consult RELEASE_REQUIREMENTS.md section RC2-003
