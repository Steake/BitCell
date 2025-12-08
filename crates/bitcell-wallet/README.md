# BitCell Wallet

**Status**: RC2 - Wallet & Security Infrastructure  
**Version**: 0.1.0  
**License**: MIT OR Apache-2.0

## Overview

The BitCell Wallet is a modular, high-performance, cross-platform cryptocurrency wallet built in Rust using the Slint UI framework. It provides a secure and user-friendly interface for managing assets on the BitCell blockchain and other supported chains (Bitcoin, Ethereum).

### Key Features

✅ **Implemented**:
- BIP39 mnemonic seed phrase generation and recovery (12/18/24 words)
- Hierarchical deterministic (HD) key derivation (BIP44)
- Multi-chain support (BitCell, Bitcoin, Ethereum, testnets)
- Secure transaction creation and signing
- Balance tracking and history
- Cross-platform native GUI (Slint)
- Zero key persistence (memory only)
- Automatic secure memory clearing

🟡 **Partial**:
- RPC integration (methods exist, integration pending)
- Hardware wallet support (interface defined, devices pending)
- Transaction broadcasting (structure exists, usage pending)

🔴 **Planned**:
- Full BIP32 compatibility
- Hardware wallet devices (Ledger, Trezor)
- Advanced fee estimation
- Multi-signature support
- Mobile wallet variants

## Architecture

The wallet consists of two main components:

### 1. Core Wallet Library (`bitcell-wallet`)

Pure Rust library providing fundamental wallet functionality:

```
bitcell-wallet/
├── mnemonic.rs         # BIP39 seed phrase generation
├── wallet.rs           # Main wallet logic and state
├── address.rs          # Multi-chain address generation
├── transaction.rs      # Transaction building and signing
├── balance.rs          # Balance tracking
├── history.rs          # Transaction history
├── chain.rs            # Multi-chain configuration
└── hardware.rs         # Hardware wallet interface
```

**Test Status**: ✅ 87/87 tests passing

### 2. GUI Application (`bitcell-wallet-gui`)

Native cross-platform application using Slint:

```
bitcell-wallet-gui/
├── src/
│   ├── main.rs         # Application logic and state
│   ├── rpc_client.rs   # BitCell node communication
│   ├── qrcode.rs       # QR code generation
│   └── game_viz.rs     # CA battle visualization
└── ui/
    └── main.slint      # UI definitions
```

**Build Status**: ✅ Compiles successfully (Linux verified)

## Quick Start

### Prerequisites

- Rust 1.82+
- Cargo
- Platform-specific dependencies for Slint UI

### Build

```bash
# Build core wallet library
cargo build -p bitcell-wallet

# Build GUI application
cargo build -p bitcell-wallet-gui --release

# Run tests
cargo test -p bitcell-wallet

# Run wallet GUI
./target/release/bitcell-wallet-gui
```

### Usage

**Creating a New Wallet**:

```rust
use bitcell_wallet::{Wallet, WalletConfig, Mnemonic};

// Create new wallet with fresh mnemonic
let (wallet, mnemonic) = Wallet::create_new(WalletConfig::default());

// IMPORTANT: User must backup mnemonic phrase
println!("Backup these words: {}", mnemonic.phrase());
```

**Recovering a Wallet**:

```rust
// Recover from existing mnemonic
let mnemonic = Mnemonic::from_phrase("word1 word2 ... word12")?;
let wallet = Wallet::from_mnemonic(&mnemonic, "", WalletConfig::default());
```

**Generating Addresses**:

```rust
// Generate BitCell address
let addr = wallet.generate_address(Chain::BitCell, 0)?;
println!("BitCell address: {}", addr.to_string_formatted());

// Generate Bitcoin address
let btc_addr = wallet.generate_address(Chain::Bitcoin, 0)?;
println!("Bitcoin address: {}", btc_addr.to_string_formatted());
```

**Creating and Signing Transactions**:

```rust
use bitcell_wallet::Chain;

// Set balance (in real usage, fetch from RPC)
wallet.update_balance(&from_addr, 1_000_000);

// Create and sign transaction
let signed_tx = wallet.send(
    &from_addr,
    &to_addr,
    100_000,  // amount
    100,      // fee
)?;

// Serialize for broadcasting
let tx_hex = signed_tx.hash_hex();
println!("Transaction hash: {}", tx_hex);
```

## Documentation

Comprehensive documentation is available in the `docs/` directory:

- **[WALLET_REQUIREMENTS.md](../docs/WALLET_REQUIREMENTS.md)**: Complete requirements specification
  - Functional and non-functional requirements
  - Implementation status summary
  - Testing requirements
  - Acceptance criteria

- **[WALLET_ARCHITECTURE.md](../docs/WALLET_ARCHITECTURE.md)**: Technical architecture
  - Component details and interactions
  - Security architecture
  - Data flow diagrams
  - Performance considerations
  - Extensibility points

- **[WALLET_TESTING_STRATEGY.md](../docs/WALLET_TESTING_STRATEGY.md)**: Testing and QA
  - Unit testing approach (87 tests)
  - Integration test requirements
  - Security testing checklist
  - Performance benchmarks
  - UAT scenarios

- **[WALLET_IMPLEMENTATION_CHECKLIST.md](../docs/WALLET_IMPLEMENTATION_CHECKLIST.md)**: Status tracking
  - Component implementation status
  - Priority matrix
  - Timeline estimates
  - Success criteria

## Security

### Current Security Measures

✅ **Implemented**:
- Private keys never written to disk
- Automatic secure memory clearing on lock
- Drop trait ensures cleanup
- Input validation on all operations
- Locked wallet prevents sensitive operations

⚠️ **Important Notes**:
- Current key derivation uses simplified approach (not full BIP32)
- For external wallet compatibility, full BIP32 implementation recommended
- See `wallet.rs::derive_key()` documentation for details

### Security Best Practices

1. **Always backup your mnemonic phrase**
   - Store in a secure, offline location
   - Never share with anyone
   - Never store digitally

2. **Use a strong passphrase** (optional)
   - Adds extra layer of security
   - Required to recover wallet
   - Cannot be reset if forgotten

3. **Lock your wallet when not in use**
   - Clears keys from memory
   - Prevents unauthorized transactions

4. **Verify addresses before sending**
   - Double-check recipient addresses
   - Use QR codes to prevent typos
   - Start with small test transactions

### Threat Model

**Protected Against**:
- Memory dumps (keys cleared)
- Malicious transactions (validation)
- Network eavesdropping (no keys sent)
- Clipboard attacks (address validation)

**Not Protected Against** (Future Work):
- Malware with elevated privileges
- Hardware keyloggers
- Screen capture attacks
- Supply chain attacks

**Future Enhancements**:
- Hardware wallet integration
- Biometric authentication (platform-dependent)
- Auto-lock timeout
- Secure enclave support (iOS/Android)

## Multi-Chain Support

The wallet supports multiple blockchain networks:

| Chain | Status | Coin Type | Address Format |
|-------|--------|-----------|----------------|
| BitCell | ✅ Complete | 9999 | Custom (version byte) |
| Bitcoin | ✅ Complete | 0 | P2PKH (Base58Check) |
| Bitcoin Testnet | ✅ Complete | 1 | P2PKH (Base58Check) |
| Ethereum | ✅ Complete | 60 | Keccak256 + EIP-55 |
| Ethereum Sepolia | ✅ Complete | 60 | Keccak256 + EIP-55 |
| Custom | ✅ Extensible | User-defined | Configurable |

### Adding New Chains

See `WALLET_ARCHITECTURE.md` section 7.1 for details on adding support for additional blockchains.

## Performance

### Target Metrics

- **Startup time**: < 2 seconds
- **Memory footprint**: < 100MB idle
- **Address generation**: < 10ms per address
- **Transaction signing**: < 5ms
- **UI frame rate**: 60fps sustained

### Optimization Features

- Lazy key derivation (on-demand only)
- Limited address lookahead (configurable)
- Native rendering (no WebView)
- Hardware acceleration where available

## Testing

### Unit Tests

Run the comprehensive test suite:

```bash
# All wallet tests
cargo test -p bitcell-wallet

# With output
cargo test -p bitcell-wallet -- --nocapture

# Specific module
cargo test -p bitcell-wallet mnemonic::tests

# With property tests
cargo test -p bitcell-wallet --features proptest
```

**Current Status**: ✅ 87/87 tests passing

### Test Coverage

| Module | Tests | Coverage |
|--------|-------|----------|
| mnemonic | 11 | High |
| wallet | 16 | High |
| transaction | 11 | High |
| address | 8 | High |
| balance | 13 | High |
| history | 13 | High |
| hardware | 7 | Medium |
| chain | 7 | High |
| lib | 1 | High |

### Benchmarks

```bash
# Run performance benchmarks
cargo bench -p bitcell-wallet

# Results in target/criterion/
```

## Development

### Code Style

```bash
# Format code
cargo fmt --all

# Lint
cargo clippy --all -- -D warnings

# Generate documentation
cargo doc --no-deps --open
```

### Project Structure

```
crates/
├── bitcell-wallet/          # Core wallet library
│   ├── src/
│   │   ├── lib.rs
│   │   ├── wallet.rs
│   │   ├── mnemonic.rs
│   │   ├── address.rs
│   │   ├── transaction.rs
│   │   ├── balance.rs
│   │   ├── history.rs
│   │   ├── chain.rs
│   │   └── hardware.rs
│   ├── tests/               # Integration tests
│   └── Cargo.toml
│
└── bitcell-wallet-gui/      # GUI application
    ├── src/
    │   ├── main.rs
    │   ├── rpc_client.rs
    │   ├── qrcode.rs
    │   └── game_viz.rs
    ├── ui/
    │   └── main.slint
    ├── build.rs
    └── Cargo.toml
```

## Known Limitations

1. **Key Derivation**: Uses simplified approach, not full BIP32 compatible
   - Impact: May not be compatible with other BIP32-compliant wallets
   - Workaround: Use exclusively with BitCell wallet
   - Fix: Planned for v1.0 (full BIP32 implementation)

2. **Hardware Wallet Support**: Interface only, no device integration
   - Impact: Cannot use Ledger/Trezor devices
   - Workaround: Use software signing only
   - Fix: Planned for v1.0

3. **Transaction Broadcasting**: GUI integration incomplete
   - Impact: Cannot submit transactions from GUI yet
   - Workaround: Use CLI or RPC directly
   - Fix: High priority for RC2

4. **Balance Updates**: No RPC polling in GUI
   - Impact: Manual balance refresh required
   - Workaround: Restart application
   - Fix: High priority for RC2

## Roadmap

### RC2 (Current Sprint)
- [ ] Complete RPC integration in GUI
- [ ] Transaction submission flow
- [ ] Real-time balance updates
- [ ] Transaction history UI
- [ ] User documentation
- [ ] Platform verification (macOS, Windows)

### v1.0 (Mainnet)
- [ ] Full BIP32 compatibility
- [ ] Hardware wallet support (Ledger, Trezor)
- [ ] External security audit
- [ ] Mobile wallet variants (iOS, Android)
- [ ] Light client mode
- [ ] Advanced features (multi-sig, time-locks)

### Future Enhancements
- Browser extension
- DApp browser integration
- Cross-chain swaps
- Staking interface
- NFT management
- DEX integration

## Contributing

We welcome contributions! Areas that need help:

- [ ] Hardware wallet device integration
- [ ] Additional chain support
- [ ] Performance optimizations
- [ ] UI/UX improvements
- [ ] Documentation and tutorials
- [ ] Security reviews

### Development Setup

1. Clone the repository
2. Install Rust 1.82+
3. Run `cargo test -p bitcell-wallet` to verify setup
4. See `WALLET_ARCHITECTURE.md` for architectural details

## Support

- **Documentation**: See `docs/` directory
- **Issues**: GitHub Issues
- **Security**: Report vulnerabilities privately
- **Status**: Pre-audit alpha - DO NOT use with real funds

## License

Dual-licensed under MIT / Apache 2.0.

Choose whichever makes your lawyer happier.

## Credits

- **BIP39/BIP44 Standards**: Bitcoin community
- **Slint UI Framework**: Slint team
- **Rust Ecosystem**: Rust Foundation and community
- **Cryptography Libraries**: k256, ed25519-dalek maintainers

---

**Built with** 🦀 Rust + 🎨 Slint + 🔐 Zero-Knowledge

_"Your keys, your coins, your control"_
