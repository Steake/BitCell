# Mobile Wallet SDK - Known Issues and Future Work

## Critical Security Issues (Documented)

### Password Verification
**Status:** NOT IMPLEMENTED - Clearly documented with warnings

**Affected Methods:**
- `unlock()` - Accepts any password
- `export_mnemonic()` - No password verification before export
- `change_password()` - Doesn't verify old password

**Required for Production:**
- Implement password-based key derivation (PBKDF2 with 100,000+ iterations or Argon2)
- Store password hash/verification data
- Verify password before allowing access to sensitive operations

### Backup Encryption
**Status:** NOT IMPLEMENTED - Only hex encoding used

**Current State:**
- Backups are NOT encrypted
- Simple hex encoding provides no security
- **DO NOT use for production or real secrets**

**Required for Production:**
- Implement AES-256-GCM encryption
- Use PBKDF2 for password-derived key (100,000+ iterations)
- Generate random salt and nonce per backup
- Properly test encryption/decryption round-trips

### Mnemonic Storage
**Status:** INSECURE - Stored as plaintext

**Current Issue:**
- Mnemonic phrase stored as plaintext in secure storage
- Should derive and store encrypted seed instead

**Required for Production:**
- Convert mnemonic to seed immediately on creation
- Encrypt seed with password-derived key
- Store only encrypted seed
- Never keep plaintext mnemonic in storage

## Race Conditions

### Lock State Management
**Issue:** Multiple RwLocks updated separately (wallet and lock_state)
**Impact:** Potential inconsistent state between updates
**Solution:** Use single lock or atomic update for both fields

## Missing Test Coverage

**Needs Tests:**
- `unlock_with_biometric()` - All result paths
- `create_backup()` and `restore_from_backup()` - Integration tests
- Race condition scenarios
- Error handling paths

## Documentation Issues

### Integration Guide
**Issue:** Example uses `toJson()` but actual method may differ
**Solution:** Verify generated binding names and update examples

## Build Issues (FIXED)

✅ UniFFI build configuration - Fixed in commit 1b0d172
✅ UniFFI scaffolding macro - Fixed in commit 1b0d172
✅ Test expectations - Fixed in commit 1b0d172

## Placeholder Implementations

The following are intentional placeholders documented for future work:

1. **Transaction Signing** - Returns placeholder values
2. **Message Signing** - Returns NotImplemented error
3. **Account Info** - Returns placeholder data
4. **Address/Public Key** - Returns placeholder strings

These require enhancement of the underlying `bitcell-wallet` crate to expose necessary APIs.

## Priority for Next Phase

### High Priority (Security-Critical)
1. Implement password verification
2. Implement backup encryption
3. Secure mnemonic storage
4. Fix race conditions

### Medium Priority (Functionality)
1. Complete transaction signing integration
2. Add comprehensive test coverage
3. Implement platform-specific storage (iOS Keychain, Android Keystore)
4. Implement platform-specific biometrics

### Low Priority (Enhancement)
1. Better error granularity for biometric failures
2. Persist biometric enable/disable setting
3. Add enable_biometric implementation
4. Documentation updates

## Testing Strategy

Before production use, must have:
- [ ] Unit tests for all security-critical paths
- [ ] Integration tests for wallet lifecycle
- [ ] Security audit of cryptographic implementations
- [ ] Platform-specific integration tests (iOS/Android)
- [ ] Race condition tests with concurrent access
- [ ] Fuzzing of input validation
