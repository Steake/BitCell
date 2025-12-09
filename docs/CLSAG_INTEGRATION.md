# CLSAG Integration Guide

## Overview

BitCell implements **CLSAG (Concise Linkable Spontaneous Anonymous Group)** ring signatures to provide tournament participant anonymity while preventing double-signing through key image tracking.

## Key Features

### Ring Signatures
- **Anonymity**: Hides the actual signer among a ring of possible signers
- **Linkability**: Multiple signatures from the same signer produce the same key image
- **Unforgeability**: Only the holder of a secret key in the ring can produce valid signatures

### Configurable Ring Sizes
- **Minimum**: 11 participants (MIN_RING_SIZE)
- **Default**: 16 participants (DEFAULT_RING_SIZE) 
- **Maximum**: 64 participants (MAX_RING_SIZE)

### Double-Spend Prevention
- **Key Images**: Unique identifier per secret key
- **O(1) Lookup**: HashSet-based registry for efficient duplicate detection
- **Thread-Safe**: Arc<Mutex<>> wrapper for concurrent access

## API Usage

### Basic Ring Signature

```rust
use bitcell_crypto::{ClsagSecretKey, ClsagPublicKey, ClsagSignature, DEFAULT_RING_SIZE};

// Generate ring of participants
let secret_keys: Vec<ClsagSecretKey> = (0..DEFAULT_RING_SIZE)
    .map(|_| ClsagSecretKey::generate())
    .collect();

let ring: Vec<ClsagPublicKey> = secret_keys
    .iter()
    .map(|sk| sk.public_key())
    .collect();

// Sign a message
let my_secret_key = &secret_keys[5];
let message = b"tournament entry";

let signature = ClsagSignature::sign(my_secret_key, &ring, message)?;

// Verify the signature
signature.verify(&ring, message)?;
```

### Key Image Extraction

```rust
use bitcell_crypto::KeyImage;

// Get key image from signature
let key_image: &KeyImage = signature.key_image();

// Or directly from secret key
let my_key_image = my_secret_key.key_image();
```

### Double-Spend Prevention

```rust
use bitcell_state::KeyImageRegistry;

// Create registry
let mut registry = KeyImageRegistry::new();

// First signature succeeds
let sig1 = ClsagSignature::sign(&secret_keys[0], &ring, b"entry_1")?;
registry.check_and_mark(*sig1.key_image())?; // OK

// Second signature from same key fails
let sig2 = ClsagSignature::sign(&secret_keys[0], &ring, b"entry_2")?;
let result = registry.check_and_mark(*sig2.key_image());
assert!(result.is_err()); // Double-spend detected!
```

### Concurrent Access

```rust
use std::sync::Arc;
use std::sync::Mutex;

// Create shared registry
let registry = KeyImageRegistry::new_shared();

// Use from multiple threads
let reg_clone = Arc::clone(&registry);
std::thread::spawn(move || {
    let mut reg = reg_clone.lock().unwrap();
    reg.check_and_mark(some_key_image)?;
});
```

## Performance Characteristics

Ring signature operations scale linearly with ring size (O(n)):

| Ring Size | Sign Time | Verify Time | Key Image |
|-----------|-----------|-------------|-----------|
| 11 (min)  | ~3-4 ms   | ~3-4 ms     | <1 ms     |
| 16 (default) | ~4-5 ms | ~4-5 ms     | <1 ms     |
| 32        | ~7-8 ms   | ~7-8 ms     | <1 ms     |
| 64 (max)  | ~12-15 ms | ~12-15 ms   | <1 ms     |

Key image registry operations are O(1):
- `contains()`: <1 μs
- `mark()`: <1 μs  
- `check_and_mark()`: <1 μs

## Tournament Integration

### Registration Phase

```rust
// Each participant creates a signature with ring of eligible keys
let participant_ring = get_eligible_participants(); // Vec<ClsagPublicKey>
let entry_message = format!("tournament_{}", tournament_id);

let signature = ClsagSignature::sign(
    &my_secret_key,
    &participant_ring,
    entry_message.as_bytes()
)?;

// Submit signature with key image
tournament.register(signature)?;
```

### Validation

```rust
// Server validates each entry
fn validate_tournament_entry(
    signature: &ClsagSignature,
    ring: &[ClsagPublicKey],
    message: &[u8],
    registry: &mut KeyImageRegistry,
) -> Result<()> {
    // 1. Verify ring signature
    signature.verify(ring, message)?;
    
    // 2. Check for double-signing
    registry.check_and_mark(*signature.key_image())?;
    
    Ok(())
}
```

## Security Considerations

### Ring Size Selection

**Trade-offs:**
- **Smaller rings** (11-16): Faster, less bandwidth, weaker anonymity
- **Larger rings** (32-64): Slower, more bandwidth, stronger anonymity

**Recommendations:**
- Use DEFAULT_RING_SIZE (16) for most cases
- Increase to 32+ for high-value tournaments
- Never go below MIN_RING_SIZE (11) for security

### Key Image Persistence

The current KeyImageRegistry is **in-memory only**. For production:

```rust
// TODO: Implement persistence layer
impl KeyImageRegistry {
    pub fn load_from_disk(path: &Path) -> Result<Self>;
    pub fn save_to_disk(&self, path: &Path) -> Result<()>;
}
```

RocksDB integration planned for RC2 (see RELEASE_REQUIREMENTS.md).

### Replay Protection

Key images prevent the same key from signing multiple times, but messages should include:
- Tournament ID
- Round number  
- Timestamp/nonce

```rust
let message = format!(
    "tournament_{}_round_{}_{}",
    tournament_id,
    round,
    timestamp
);
```

## Implementation Details

### Cryptographic Primitives

- **Curve**: Ristretto255 (based on Curve25519)
- **Hash-to-Point**: SHA-512 with domain separation
- **Hash-to-Scalar**: SHA-512 for challenge generation

### Ring Equation

CLSAG verifies the equation:
```
c_{i+1} = H(m, P_0, ..., P_n, I, s_i·G + c_i·P_i, s_i·H_p(P_i) + c_i·I)
```

Where:
- `m`: Message
- `P_i`: Public keys in ring
- `I`: Key image
- `s_i`: Response values
- `c_i`: Challenge values
- `H_p()`: Hash-to-point function

### Key Image Generation

```
I = x · H_p(P)
```

Where:
- `x`: Secret key scalar
- `P`: Public key (x·G)
- `H_p()`: Hash-to-point function

This ensures:
1. **Uniqueness**: Each secret key produces a unique key image
2. **Linkability**: Same key always produces same key image
3. **Untraceability**: Cannot determine which key in ring produced it

## Testing

### Unit Tests

Run CLSAG unit tests:
```bash
cargo test -p bitcell-crypto clsag::tests
```

Run KeyImageRegistry tests:
```bash
cargo test -p bitcell-state key_images::tests
```

### Integration Tests

Run full integration suite:
```bash
cargo test --test clsag_integration_test
```

Specific scenarios:
```bash
# Ring size validation
cargo test --test clsag_integration_test ring_size

# Double-spend detection
cargo test --test clsag_integration_test double_spend

# Concurrent access
cargo test --test clsag_integration_test concurrent
```

### Benchmarks

Run performance benchmarks:
```bash
cargo bench -p bitcell-crypto -- clsag
```

This measures:
- Signing time for different ring sizes
- Verification time for different ring sizes
- Key image generation time

## Migration from Mock Ring Signatures

If upgrading from hash-based mock ring signatures:

1. **Update imports:**
```rust
// Old
use bitcell_crypto::ring::{RingSignature, RingPublicKey};

// New
use bitcell_crypto::{ClsagSignature, ClsagPublicKey};
```

2. **Update signing:**
```rust
// Old
let sig = RingSignature::sign_mock(&sk, &ring, message);

// New
let sig = ClsagSignature::sign(&sk, &ring, message)?;
```

3. **Add key image tracking:**
```rust
// New requirement
let mut registry = KeyImageRegistry::new();
registry.check_and_mark(*sig.key_image())?;
```

4. **Update ring size:**
```rust
// Ensure ring meets minimum size
assert!(ring.len() >= MIN_RING_SIZE);
```

## References

- **CLSAG Paper**: "CLSAG: Compact Linkable Spontaneous Anonymous Group Signatures"
  - Authors: Goodell, Noether, RandomRun
  - https://eprint.iacr.org/2019/654

- **Monero Implementation**: 
  - https://github.com/monero-project/monero

- **Curve25519/Ristretto**:
  - https://ristretto.group/

## Future Enhancements

Planned for future releases:

### RC2 (Q1 2026)
- [ ] RocksDB persistence for KeyImageRegistry
- [ ] Merkle commitment for light client key image proofs
- [ ] Batch verification optimization

### RC3 (Q2 2026)
- [ ] Improved ring member selection algorithms
- [ ] Ring signature aggregation
- [ ] Zero-knowledge range proofs for amounts

## Support

For questions or issues:
- GitHub Issues: https://github.com/Steake/BitCell/issues
- Documentation: https://github.com/Steake/BitCell/tree/master/docs
- Security: security@bitcell.org (for vulnerabilities)
