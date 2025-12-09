# BitCell Wallet Architecture

**Version**: 1.0  
**Status**: Design Document  
**Last Updated**: 2025-12-06

## 1. Overview

The BitCell Wallet is a modular, cross-platform cryptocurrency wallet application built in Rust. It consists of two primary components:

1. **bitcell-wallet**: Core wallet library providing fundamental cryptocurrency wallet functionality
2. **bitcell-wallet-gui**: Native GUI application using Slint UI framework

This architecture emphasizes security, performance, and maintainability through clear separation of concerns and minimal external dependencies.

## 2. High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    GUI Layer (Slint UI)                      │
│  ┌────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │  Wallet    │  │ Transaction  │  │   Settings &     │   │
│  │  Overview  │  │   Interface  │  │   Management     │   │
│  └────────────┘  └──────────────┘  └──────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Application State & Logic Layer                 │
│  ┌────────────────────────────────────────────────────────┐ │
│  │            bitcell-wallet-gui Application             │ │
│  │  • Event Handlers                                      │ │
│  │  • State Management                                    │ │
│  │  • RPC Client                                          │ │
│  │  • UI Updates & Polling                                │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                Core Wallet Library Layer                     │
│  ┌────────────────────────────────────────────────────────┐ │
│  │                  bitcell-wallet Crate                  │ │
│  │                                                         │ │
│  │  ┌─────────────┐  ┌──────────────┐  ┌─────────────┐  │ │
│  │  │  Mnemonic   │  │   Wallet     │  │  Address    │  │ │
│  │  │  Generator  │  │   Manager    │  │  Manager    │  │ │
│  │  └─────────────┘  └──────────────┘  └─────────────┘  │ │
│  │                                                         │ │
│  │  ┌─────────────┐  ┌──────────────┐  ┌─────────────┐  │ │
│  │  │Transaction  │  │   Balance    │  │  History    │  │ │
│  │  │   Builder   │  │   Tracker    │  │  Tracker    │  │ │
│  │  └─────────────┘  └──────────────┘  └─────────────┘  │ │
│  │                                                         │ │
│  │  ┌─────────────┐  ┌──────────────┐                    │ │
│  │  │  Hardware   │  │    Chain     │                    │ │
│  │  │   Wallet    │  │   Support    │                    │ │
│  │  └─────────────┘  └──────────────┘                    │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Cryptographic Primitives Layer                  │
│  ┌────────────────────────────────────────────────────────┐ │
│  │                 bitcell-crypto Crate                   │ │
│  │  • Key Generation (ECDSA, Ed25519)                     │ │
│  │  • Signature Creation & Verification                   │ │
│  │  • Hash Functions (SHA256, Blake3)                     │ │
│  │  • Secure Random Number Generation                     │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    External Services                         │
│  ┌──────────────────┐         ┌─────────────────────────┐  │
│  │  BitCell Node    │         │  Hardware Wallet Device │  │
│  │  (JSON-RPC)      │         │  (Ledger/Trezor)        │  │
│  └──────────────────┘         └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## 3. Component Details

### 3.1 Core Wallet Library (bitcell-wallet)

The core wallet library provides chain-agnostic wallet functionality that can be used by any frontend (GUI, CLI, or programmatic).

#### 3.1.1 Wallet Module (`wallet.rs`)

**Responsibility**: Central wallet management and coordination

**Key Components**:
- `Wallet`: Main wallet structure
- `WalletConfig`: Configuration settings
- `WalletState`: Lock/unlock state management
- `DerivationPath`: BIP44 path management

**Key Operations**:
- `create_new()`: Generate new wallet with mnemonic
- `from_mnemonic()`: Recover wallet from seed phrase
- `lock()` / `unlock()`: Security state management
- `generate_address()`: Create new addresses
- `create_transaction()`: Build unsigned transactions
- `sign_transaction()`: Sign with appropriate key
- `send()`: Combined create + sign operation

**Security Features**:
- Master seed only in memory when unlocked
- Automatic key derivation on demand
- Secure cleanup via Drop trait
- Nonce tracking per address

#### 3.1.2 Mnemonic Module (`mnemonic.rs`)

**Responsibility**: BIP39 seed phrase generation and management

**Key Components**:
- `Mnemonic`: Wrapper around BIP39 phrase
- `SeedBytes`: 64-byte seed derived from mnemonic

**Features**:
- 12, 18, 24-word phrase support
- Passphrase protection
- Deterministic seed derivation using PBKDF2
- Validation of phrase checksums

**Entropy Sources**:
- Uses `rand` crate with secure OS RNG
- 128-bit (12 words), 192-bit (18), 256-bit (24)

#### 3.1.3 Address Module (`address.rs`)

**Responsibility**: Multi-chain address generation and formatting

**Key Components**:
- `Address`: Universal address representation
- `AddressType`: Chain-specific formats
- `AddressManager`: Address collection management

**Supported Formats**:
- BitCell: Custom format with version byte
- Bitcoin: P2PKH (Base58Check)
- Ethereum: Keccak256 + EIP-55 checksum

**Key Operations**:
- `from_public_key_bitcell()`: BitCell address
- `from_public_key_bitcoin()`: BTC address (mainnet/testnet)
- `from_public_key_ethereum()`: ETH address
- `to_string_formatted()`: Chain-appropriate display

#### 3.1.4 Transaction Module (`transaction.rs`)

**Responsibility**: Transaction creation, signing, and serialization

**Key Components**:
- `Transaction`: Unsigned transaction structure
- `SignedTransaction`: Transaction with signature
- `TransactionBuilder`: Fluent API for construction
- `FeeEstimator`: Fee calculation utilities

**Transaction Fields**:
```rust
pub struct Transaction {
    pub chain: Chain,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
    pub data: Vec<u8>,
}
```

**Signature Generation**:
- ECDSA (secp256k1) for Bitcoin/Ethereum
- Ed25519 for BitCell native
- Transaction hash as signature input
- Deterministic signing (RFC 6979)

#### 3.1.5 Balance Module (`balance.rs`)

**Responsibility**: Balance tracking and queries

**Key Components**:
- `Balance`: Amount with chain info
- `BalanceTracker`: Multi-address balance management

**Features**:
- Per-address balance tracking
- Per-chain total calculations
- Sufficient balance validation
- Atomic balance updates

#### 3.1.6 History Module (`history.rs`)

**Responsibility**: Transaction history tracking

**Key Components**:
- `TransactionRecord`: Historical transaction data
- `TransactionHistory`: Collection manager

**Features**:
- Confirmation tracking
- Transaction memos
- Time-based filtering
- Export functionality

#### 3.1.7 Chain Module (`chain.rs`)

**Responsibility**: Multi-chain configuration and constants

**Supported Chains**:
```rust
pub enum Chain {
    BitCell,              // Native chain
    Bitcoin,              // BTC mainnet
    BitcoinTestnet,       // BTC testnet
    Ethereum,             // ETH mainnet
    EthereumSepolia,      // ETH testnet
    Custom(String),       // Extensible
}
```

**Chain Configuration**:
- Coin type (BIP44)
- Network parameters
- Address formats
- Default RPC endpoints

#### 3.1.8 Hardware Wallet Module (`hardware.rs`)

**Responsibility**: Hardware wallet integration interface

**Status**: Interface defined, implementation pending

**Supported Devices**:
- Ledger (planned)
- Trezor (planned)
- Software signing (implemented)

**Key Abstraction**:
```rust
pub trait HardwareWalletDevice {
    fn get_address(&self, path: &str) -> Result<Address>;
    fn sign_transaction(&self, tx: &Transaction, path: &str) -> Result<Signature>;
}
```

### 3.2 GUI Application (bitcell-wallet-gui)

Native cross-platform wallet application using Slint UI.

#### 3.2.1 Application State (`main.rs`)

**Responsibility**: Global application state management

**State Structure**:
```rust
struct AppState {
    wallet: Option<Wallet>,
    mnemonic: Option<Mnemonic>,
    rpc_client: Option<RpcClient>,
}
```

**State Management**:
- Shared via `Rc<RefCell<AppState>>`
- Updates propagate to UI via callbacks
- Atomic state transitions

#### 3.2.2 RPC Client (`rpc_client.rs`)

**Responsibility**: Communication with BitCell node

**Key Methods**:
- `get_node_info()`: Node status
- `get_balance()`: Address balance query
- `send_raw_transaction()`: Broadcast transaction
- `get_block_number()`: Current height

**Connection Management**:
- Configurable endpoint
- Automatic retry logic
- Connection status polling
- Graceful failure handling

#### 3.2.3 UI Components (`main.slint`)

**Main Views**:
1. **Welcome View**: New/restore wallet
2. **Overview View**: Balance dashboard
3. **Send View**: Transaction creation
4. **Receive View**: Address display + QR
5. **History View**: Transaction list
6. **Settings View**: Configuration

**Slint Features Used**:
- Native rendering (no WebView)
- Responsive layouts
- Animations and transitions
- Keyboard navigation
- Theme support

#### 3.2.4 Event Handling

**Callback Pattern**:
```rust
let state = Rc::new(RefCell::new(AppState::new()));

main_window.on_create_wallet({
    let state = state.clone();
    move |name, passphrase| {
        // Wallet creation logic
    }
});
```

**Event Types**:
- Wallet creation/restoration
- Transaction submission
- Address generation
- Settings updates
- Timer-based polling

#### 3.2.5 QR Code Generation (`qrcode.rs`)

**Responsibility**: Generate QR codes for addresses

**Implementation**:
- Uses `qrcodegen` crate
- Base64-encoded PNG output
- Error correction level: Medium
- Optimized size for display

#### 3.2.6 Game Visualization (`game_viz.rs`)

**Responsibility**: Visualize BitCell CA battles (optional feature)

**Purpose**: 
- Educational: Show blockchain consensus mechanism
- Engaging: Make wallet more interesting
- Status: Placeholder for future enhancement

## 4. Security Architecture

### 4.1 Key Management Security

**In-Memory Only**:
- Private keys NEVER written to disk
- Master seed cleared on lock
- Derived keys cleared on lock
- Drop trait ensures cleanup

**Derivation Security**:
- Deterministic key derivation
- No key reuse across chains
- Hardened derivation for accounts
- Non-hardened for addresses

**Signing Security**:
- Keys only accessible when unlocked
- Signature creation in secure memory
- Immediate cleanup after signing
- No key export functionality

### 4.2 Network Security

**RPC Communication**:
- No sensitive data in RPC calls
- Transaction signing client-side only
- Signed transactions transmitted
- No private keys over network

**Future Enhancements**:
- TLS for RPC connections
- Certificate pinning
- Request signing
- Rate limiting

### 4.3 UI Security

**Input Validation**:
- Address format validation
- Amount range checking
- Fee reasonableness checks
- Mnemonic phrase validation

**User Warnings**:
- Confirm before transactions
- Warn on large transfers
- Display fee estimates
- Show transaction details

### 4.4 Threat Model

**Protected Against**:
- Memory dumps (key clearing)
- Malicious transactions (validation)
- Network eavesdropping (no keys sent)
- Clipboard attacks (address validation)

**Not Protected Against** (Future Work):
- Malware with elevated privileges
- Hardware keyloggers
- Screen capture attacks
- Supply chain attacks

## 5. Data Flow Diagrams

### 5.1 Wallet Creation Flow

```
User Input (Mnemonic Choice)
        │
        ▼
Generate Entropy (128/192/256 bits)
        │
        ▼
BIP39 Mnemonic Generation
        │
        ▼
PBKDF2 Seed Derivation (+ optional passphrase)
        │
        ▼
Wallet Initialization
        │
        ├─► Address Pre-generation (Lookahead)
        │        │
        │        └─► BIP44 Derivation per Chain
        │                 │
        │                 └─► Address Creation & Storage
        │
        └─► Store Wallet Config (NO KEYS)
```

### 5.2 Transaction Creation and Broadcasting Flow

```
User: Enter Amount, Recipient, Fee
        │
        ▼
Validate Balance & Inputs
        │
        ▼
Create Transaction Struct
        │
        ▼
Get Nonce from Wallet State
        │
        ▼
User Confirms Transaction
        │
        ▼
Derive Signing Key (requires unlocked wallet)
        │
        ▼
Sign Transaction (ECDSA/Ed25519)
        │
        ▼
Serialize Signed Transaction
        │
        ▼
RPC: send_raw_transaction()
        │
        ▼
Update Nonce & History
        │
        ▼
Poll for Confirmation
```

### 5.3 Balance Update Flow

```
Timer Trigger (every N seconds)
        │
        ▼
For Each Managed Address:
        │
        ├─► RPC: get_balance(address)
        │        │
        │        ▼
        │   Update Balance Tracker
        │        │
        │        ▼
        └────  Update UI Display
```

## 6. Performance Considerations

### 6.1 Memory Management

**Target Footprint**: < 100MB idle

**Optimization Strategies**:
- Lazy key derivation (on-demand only)
- Limited address lookahead (configurable)
- Transaction history pagination
- UI texture caching in Slint

**Memory Clearing**:
- Explicit Drop implementations
- Zeroize sensitive data
- No key serialization

### 6.2 Startup Performance

**Target**: < 2 seconds on modern hardware

**Optimization**:
- Async wallet loading
- Deferred address generation
- Lazy UI component initialization
- Cached RPC responses

### 6.3 UI Rendering

**Target**: 60fps interactions

**Slint Optimizations**:
- Native rendering (OpenGL/Direct3D/Metal)
- Efficient property bindings
- Minimal redraws
- Hardware acceleration

## 7. Extensibility Points

### 7.1 Adding New Chains

**Steps**:
1. Add enum variant to `Chain`
2. Implement address generation in `Address`
3. Add chain-specific signing if needed
4. Update `ChainConfig` with parameters
5. Test deterministic derivation

**Example**:
```rust
// In chain.rs
Chain::Solana => 501, // SOL coin type

// In address.rs
pub fn from_public_key_solana(pubkey: &PublicKey, index: u32) -> Self {
    // Solana address format
}
```

### 7.2 Custom Fee Estimation

**Interface**:
```rust
pub trait FeeEstimator {
    fn estimate_fee(&self, priority: FeePriority) -> Result<u64>;
}
```

**Implementation Options**:
- Static fee (current)
- RPC-based fee estimation
- Historical data analysis
- Third-party API integration

### 7.3 Plugin Architecture (Future)

**Potential Extensions**:
- DApp integrations
- DEX interfaces
- NFT management
- Staking dashboards
- Custom transaction types

## 8. Testing Strategy

### 8.1 Unit Tests

**Coverage**: All core wallet modules

**Test Categories**:
- Mnemonic generation & validation
- Key derivation determinism
- Address generation correctness
- Transaction signing verification
- Balance tracking accuracy
- History management

**Current Status**: 87 tests passing

### 8.2 Integration Tests

**Needed**:
- End-to-end transaction flow
- Multi-chain address generation
- RPC communication scenarios
- Error handling paths
- State persistence

### 8.3 Property-Based Tests

**Using `proptest`**:
- Key derivation properties
- Signature verification
- Amount arithmetic (no overflow)
- Nonce increment correctness

### 8.4 GUI Tests

**Manual Testing**:
- User interaction flows
- Visual regression checks
- Platform-specific behavior
- Accessibility features

**Automated** (Future):
- Slint testing framework
- Screenshot comparisons
- Interaction recording

## 9. Deployment Architecture

### 9.1 Build Targets

**Supported Platforms**:
- Linux (x86_64, aarch64)
- macOS (x86_64, Apple Silicon)
- Windows (x86_64)

**Build Requirements**:
- Rust 1.82+
- Platform-specific UI libraries
- C compiler for native dependencies

### 9.2 Distribution

**Methods**:
- Direct binary downloads
- Package managers (brew, apt, chocolatey)
- App stores (future)

**Update Mechanism** (Future):
- In-app update notifications
- Signature verification
- Rollback capability

### 9.3 Configuration

**User Data Locations**:
- Linux: `~/.config/bitcell-wallet/`
- macOS: `~/Library/Application Support/BitCell Wallet/`
- Windows: `%APPDATA%\BitCell Wallet\`

**Stored Data**:
- Wallet configuration (no keys!)
- Address book (future)
- User preferences
- Transaction history cache

## 10. Future Enhancements

### 10.1 Short-term (RC2 → v1.0)

1. **Complete RPC Integration**
   - Real-time balance updates
   - Transaction broadcasting
   - Confirmation tracking

2. **Hardware Wallet Support**
   - Ledger integration
   - Trezor integration
   - Device detection

3. **Enhanced Security**
   - Auto-lock timeout
   - Biometric unlock (platform-dependent)
   - Secure enclaves (iOS/Android)

4. **Improved UX**
   - Transaction templates
   - Address book
   - Multi-wallet support
   - Fiat conversion display

### 10.2 Long-term (v1.0+)

1. **Mobile Wallets**
   - iOS app (Swift + Rust core)
   - Android app (Kotlin + Rust core)
   - Shared core via FFI

2. **Advanced Features**
   - Multi-signature wallets
   - Time-locked transactions
   - Contract interaction
   - Staking interface

3. **Integration**
   - Browser extension
   - WalletConnect protocol
   - DApp browser
   - Cross-chain bridges

4. **Enterprise**
   - HSM integration
   - Audit logging
   - Permission system
   - Batch operations

## 11. References

### Standards
- **BIP39**: Mnemonic code for generating deterministic keys
- **BIP32**: Hierarchical Deterministic Wallets
- **BIP44**: Multi-Account Hierarchy for Deterministic Wallets
- **EIP-55**: Mixed-case checksum address encoding (Ethereum)

### Technologies
- **Rust**: https://www.rust-lang.org/
- **Slint UI**: https://slint.dev/
- **secp256k1**: Bitcoin/Ethereum elliptic curve
- **Ed25519**: Modern signature scheme
- **PBKDF2**: Password-based key derivation

### Related Documents
- `WALLET_REQUIREMENTS.md`: Detailed requirements
- `AGENT_PLAN.md`: Implementation roadmap
- `RPC_API_Spec.md`: Node API reference

---

**Document Owner**: BitCell Development Team  
**Review Cycle**: After architectural changes  
**Next Review**: Post-RC2 release
