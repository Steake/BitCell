# BitCell Wallet Requirements Specification

**Version**: 1.0  
**Status**: RC2 - Wallet & Security Infrastructure  
**Last Updated**: 2025-12-06

## Executive Summary

This document defines the requirements for the BitCell Wallet application, a modular, high-performance, cross-platform wallet built in Rust using the Slint UI framework. The wallet aims to provide a minimal memory footprint while supporting multiple blockchain networks.

## 1. Functional Requirements

### 1.1 Core Wallet Functionality

#### FR-1.1.1: Wallet Creation and Recovery
- **Priority**: CRITICAL
- **Status**: ✅ IMPLEMENTED
- The wallet SHALL support creation of new wallets using BIP39 mnemonic phrases
- The wallet SHALL support 12, 18, and 24-word mnemonic phrases
- The wallet SHALL allow wallet recovery from mnemonic phrases
- The wallet SHALL support optional passphrase protection (BIP39)
- **Implementation**: `crates/bitcell-wallet/src/mnemonic.rs`
- **Tests**: 11 tests passing in mnemonic module

#### FR-1.1.2: Key Management
- **Priority**: CRITICAL
- **Status**: ✅ IMPLEMENTED
- The wallet SHALL implement hierarchical deterministic (HD) key derivation
- The wallet SHALL follow BIP44 derivation path structure: `m/44'/coin_type'/account'/change/index`
- The wallet SHALL securely store private keys in memory only when unlocked
- The wallet SHALL implement secure memory zeroing on wallet lock
- **Implementation**: `crates/bitcell-wallet/src/wallet.rs`
- **Security Note**: Simplified key derivation currently used; full BIP32 compatibility recommended for external wallet interoperability

#### FR-1.1.3: Multi-Chain Support
- **Priority**: HIGH
- **Status**: ✅ IMPLEMENTED
- The wallet SHALL support BitCell native blockchain (coin_type: 9999)
- The wallet SHALL support Bitcoin (coin_type: 0)
- The wallet SHALL support Ethereum (coin_type: 60)
- The wallet SHALL support testnet variants (Bitcoin Testnet, Ethereum Sepolia)
- The wallet SHALL allow custom chain configuration
- **Implementation**: `crates/bitcell-wallet/src/chain.rs`

#### FR-1.1.4: Address Management
- **Priority**: HIGH
- **Status**: ✅ IMPLEMENTED
- The wallet SHALL generate unique addresses for each supported chain
- The wallet SHALL maintain address derivation indices per chain
- The wallet SHALL support address lookahead (pre-generation)
- The wallet SHALL display addresses in chain-specific formats
- **Implementation**: `crates/bitcell-wallet/src/address.rs`
- **Tests**: Address generation, deterministic derivation verified

### 1.2 Transaction Functionality

#### FR-1.2.1: Transaction Creation
- **Priority**: CRITICAL
- **Status**: ✅ IMPLEMENTED (Core) / 🟡 PARTIAL (GUI)
- The wallet SHALL create valid transactions for supported chains
- The wallet SHALL validate sufficient balance before transaction creation
- The wallet SHALL calculate appropriate transaction fees
- The wallet SHALL maintain accurate nonce tracking per address
- **Implementation**: `crates/bitcell-wallet/src/transaction.rs`
- **Gap**: GUI transaction building needs completion (see FR-1.3.2)

#### FR-1.2.2: Transaction Signing
- **Priority**: CRITICAL
- **Status**: ✅ IMPLEMENTED
- The wallet SHALL sign transactions using appropriate private keys
- The wallet SHALL only allow signing when wallet is unlocked
- The wallet SHALL increment nonce after successful signing
- The wallet SHALL generate transaction hashes for tracking
- **Implementation**: `crates/bitcell-wallet/src/wallet.rs::sign_transaction()`
- **Tests**: 5 transaction signing tests passing

#### FR-1.2.3: Transaction Broadcasting
- **Priority**: HIGH
- **Status**: 🔴 NOT IMPLEMENTED
- The wallet SHALL broadcast signed transactions to the network via RPC
- The wallet SHALL retry failed broadcasts with configurable policy
- The wallet SHALL track transaction status (pending, confirmed, failed)
- **Implementation Gap**: Needs RPC client integration in GUI
- **Related**: See AGENT_PLAN.md Phase 1.1

#### FR-1.2.4: Transaction History
- **Priority**: HIGH
- **Status**: ✅ IMPLEMENTED
- The wallet SHALL maintain transaction history per address
- The wallet SHALL track transaction confirmations
- The wallet SHALL support transaction memos/notes
- The wallet SHALL allow export of transaction history
- **Implementation**: `crates/bitcell-wallet/src/history.rs`
- **Tests**: 7 history tests passing

### 1.3 User Interface Requirements

#### FR-1.3.1: GUI Framework
- **Priority**: CRITICAL
- **Status**: ✅ IMPLEMENTED
- The wallet SHALL use Slint UI framework for native rendering
- The wallet SHALL support macOS, Linux, and Windows platforms
- The wallet SHALL NOT use WebView or Electron
- The wallet SHALL target 60fps for smooth interactions
- The wallet SHALL support accessibility features
- **Implementation**: `crates/bitcell-wallet-gui/` with Slint 1.9+

#### FR-1.3.2: Transaction Interface
- **Priority**: HIGH
- **Status**: 🟡 PARTIAL
- The wallet SHALL provide a form for transaction creation
- The wallet SHALL display real-time balance updates
- The wallet SHALL show estimated transaction fees
- The wallet SHALL confirm transactions before broadcasting
- **Implementation Gap**: Transaction building in GUI prepares real transactions (fetches nonce, gas price, calculates fee) but hardware wallet signing and broadcasting are not yet implemented
- **Location**: `crates/bitcell-wallet-gui/src/main.rs:388-510`
- **Action Required**: Implement hardware wallet signing and transaction broadcast functionality

#### FR-1.3.3: Balance Display
- **Priority**: HIGH
- **Status**: 🟡 PARTIAL
- The wallet SHALL display balances for all managed addresses
- The wallet SHALL show per-chain and total balances
- The wallet SHALL update balances via RPC polling
- **Implementation**: Balance tracking exists, RPC integration needs completion

#### FR-1.3.4: Address Management UI
- **Priority**: MEDIUM
- **Status**: ✅ IMPLEMENTED
- The wallet SHALL display generated addresses
- The wallet SHALL allow copying addresses to clipboard
- The wallet SHALL generate QR codes for addresses
- **Implementation**: QR code generation available in `qrcode.rs`

### 1.4 Security Requirements

#### FR-1.4.1: Secure Key Storage
- **Priority**: CRITICAL
- **Status**: ✅ IMPLEMENTED
- The wallet SHALL NEVER persist private keys to disk
- The wallet SHALL clear sensitive data from memory on lock/close
- The wallet SHALL implement Drop trait for secure cleanup
- **Implementation**: `crates/bitcell-wallet/src/wallet.rs::Drop`
- **Verified**: Memory zeroing on wallet lock

#### FR-1.4.2: Wallet Locking
- **Priority**: CRITICAL
- **Status**: ✅ IMPLEMENTED
- The wallet SHALL support manual locking
- The wallet SHALL auto-lock after configurable timeout (future)
- The wallet SHALL prevent operations requiring keys when locked
- **Tests**: Locked wallet operations verified

#### FR-1.4.3: Hardware Wallet Support
- **Priority**: MEDIUM
- **Status**: 🔴 NOT IMPLEMENTED
- The wallet SHOULD support Ledger hardware wallets
- The wallet SHOULD support Trezor hardware wallets
- The wallet SHALL gracefully handle missing hardware wallet support
- **Implementation**: Structure exists in `hardware.rs`, needs actual device integration
- **Note**: Currently returns `UnsupportedChain` error (should use specific error type)

### 1.5 Network Integration

#### FR-1.5.1: RPC Communication
- **Priority**: HIGH
- **Status**: 🟡 PARTIAL
- The wallet SHALL communicate with BitCell node via JSON-RPC
- The wallet SHALL handle RPC connection failures gracefully
- The wallet SHALL poll for balance updates
- The wallet SHALL poll for transaction confirmations
- **Implementation**: `crates/bitcell-wallet-gui/src/rpc_client.rs`
- **Gap**: Transaction submission methods exist but unused

#### FR-1.5.2: Node Connection Status
- **Priority**: MEDIUM
- **Status**: ✅ IMPLEMENTED
- The wallet SHALL display RPC connection status
- The wallet SHALL indicate when node is unreachable
- The wallet SHALL allow node endpoint configuration
- **Implementation**: Connection polling in GUI main loop

## 2. Non-Functional Requirements

### NFR-2.1: Performance
- **Priority**: HIGH
- The wallet SHALL start within 2 seconds on modern hardware
- The wallet SHALL maintain < 100MB memory footprint when idle
- The wallet SHALL handle 1000+ addresses without performance degradation
- The wallet UI SHALL maintain 60fps during interactions

### NFR-2.2: Reliability
- **Priority**: HIGH
- The wallet SHALL recover gracefully from crashes
- The wallet SHALL never corrupt wallet data
- The wallet SHALL validate all user inputs
- The wallet SHALL have comprehensive error messages

### NFR-2.3: Usability
- **Priority**: MEDIUM
- The wallet SHALL provide clear error messages
- The wallet SHALL guide users through wallet creation
- The wallet SHALL warn users about insecure operations
- The wallet SHALL support keyboard navigation

### NFR-2.4: Portability
- **Priority**: HIGH
- The wallet SHALL compile on macOS, Linux, Windows
- The wallet SHALL use platform-appropriate UI conventions
- The wallet SHALL support HiDPI/Retina displays
- The wallet SHALL work on systems without GPU acceleration

## 3. Testing Requirements

### TR-3.1: Unit Testing
- **Status**: ✅ COMPREHENSIVE
- **Coverage**: 87 unit tests passing
- All core wallet functionality has unit tests
- Mnemonic generation and validation tested
- Transaction creation and signing tested
- Address generation and determinism verified

### TR-3.2: Integration Testing
- **Status**: 🔴 NEEDED
- End-to-end transaction flow testing required
- RPC integration testing required
- Multi-chain transaction testing required

### TR-3.3: Security Testing
- **Status**: 🟡 PARTIAL
- Memory zeroing verified
- Locked wallet operations tested
- Full security audit pending

### TR-3.4: GUI Testing
- **Status**: 🔴 NEEDED
- UI interaction testing required
- Visual regression testing recommended
- Accessibility testing required

## 4. Documentation Requirements

### DR-4.1: User Documentation
- **Status**: 🔴 NEEDED
- User guide for wallet setup and usage
- Multi-chain usage examples
- Security best practices guide
- Recovery procedures documentation

### DR-4.2: Developer Documentation
- **Status**: 🟡 PARTIAL
- API documentation in code (rustdoc)
- Architecture documentation needed
- Integration guide needed
- Custom chain configuration guide needed

## 5. Implementation Status Summary

### Completed Components ✅
1. **Core Wallet Library** (`bitcell-wallet`)
   - Mnemonic generation and recovery (BIP39)
   - HD key derivation (simplified BIP44)
   - Multi-chain address generation
   - Transaction creation and signing
   - Balance tracking
   - Transaction history
   - Wallet lock/unlock mechanism
   - Secure memory handling

2. **GUI Application** (`bitcell-wallet-gui`)
   - Slint UI framework integration
   - Basic wallet interface
   - RPC client structure
   - QR code generation
   - Connection status monitoring

### Partial Implementation 🟡
1. **Transaction Flow**
   - Core: Complete
   - GUI: Needs real transaction building
   - Broadcasting: Structure exists, needs usage

2. **RPC Integration**
   - Client methods implemented
   - Polling for balances needed
   - Transaction submission integration needed

3. **Hardware Wallet Support**
   - Interface defined
   - Device integration pending

### Not Implemented 🔴
1. **Complete Transaction Broadcasting**
2. **Hardware Wallet Device Integration** (Ledger/Trezor)
3. **Comprehensive Integration Tests**
4. **User Documentation**
5. **Auto-lock Timeout Feature**

## 6. Dependencies and Constraints

### Technical Dependencies
- Rust 1.82+
- Slint 1.9+ UI framework
- tokio async runtime
- BitCell node with JSON-RPC API
- Platform-specific UI libraries

### Constraints
- No network access without node
- Limited by RPC API capabilities
- Platform-specific build requirements for Slint
- Hardware wallet support requires device libraries

## 7. Risks and Mitigations

### Risk 1: Key Compatibility
- **Risk**: Simplified key derivation may not be compatible with other BIP32 wallets
- **Mitigation**: Document limitation; plan full BIP32 implementation for v1.0
- **Priority**: MEDIUM

### Risk 2: RPC Reliability
- **Risk**: Wallet dependent on node availability
- **Mitigation**: Implement robust retry logic; offline mode future feature
- **Priority**: LOW

### Risk 3: Hardware Wallet Complexity
- **Risk**: Hardware wallet integration is complex and error-prone
- **Mitigation**: Start with software wallet only; add hardware support incrementally
- **Priority**: LOW

## 8. Acceptance Criteria

### For RC2 Completion
- [ ] All core wallet tests passing (✅ Done: 87/87)
- [ ] GUI builds on all platforms (✅ Done: Linux verified)
- [ ] Transaction creation works end-to-end (🟡 Core done, GUI partial)
- [ ] Balance updates via RPC (🔴 To do)
- [ ] Transaction broadcasting functional (🔴 To do)
- [ ] Security audit recommendations addressed (🔴 To do)
- [ ] Basic user documentation available (🔴 To do)

### For v1.0 Mainnet
- [ ] Full BIP32 key derivation
- [ ] Hardware wallet support (Ledger, Trezor)
- [ ] Comprehensive integration tests
- [ ] Professional security audit
- [ ] Complete user and developer documentation
- [ ] Mobile wallet variants

## 9. Future Enhancements

### Post-RC2 Features
1. Auto-lock timeout configuration
2. Multiple wallet file support
3. Address book / contacts
4. Transaction templates
5. Advanced fee estimation
6. Multi-signature support
7. Staking interface
8. DApp browser integration

### Long-term Vision
1. Mobile wallet (iOS/Android)
2. Browser extension
3. Light client mode
4. Cold storage support
5. Recovery social schemes
6. Hardware security module (HSM) integration

## 10. References

- **Implementation Plan**: `AGENT_PLAN.md`
- **Current Status**: `todo_now.md`
- **API Specification**: `docs/RPC_API_Spec.md`
- **BIP39 Standard**: https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki
- **BIP44 Standard**: https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki
- **Slint Documentation**: https://slint.dev/

---

**Document Owner**: BitCell Development Team  
**Review Cycle**: After each major milestone  
**Next Review**: Post-RC2 release
