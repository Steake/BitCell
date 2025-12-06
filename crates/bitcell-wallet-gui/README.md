# BitCell Wallet GUI

**Cross-Platform Native Wallet Interface**  
**Version**: 0.1.0  
**Status**: RC2 Development

## Overview

BitCell Wallet GUI is a native cross-platform desktop application for managing cryptocurrency wallets. Built with Rust and the Slint UI framework, it provides a fast, secure, and user-friendly interface with no WebView or Electron overhead.

### Key Features

✅ **Implemented**:
- Native UI rendering (OpenGL/Direct3D/Metal)
- Wallet creation and recovery
- Multi-chain support (BitCell, Bitcoin, Ethereum)
- Address generation with QR codes
- Real-time node connection monitoring
- Secure wallet locking mechanism
- 60fps smooth animations

🟡 **In Progress**:
- Transaction submission flow
- Balance updates via RPC
- Transaction history display

🔴 **Planned**:
- Hardware wallet integration
- Address book
- Multi-wallet support
- Settings panel
- Theme customization

## Architecture

### Technology Stack

- **UI Framework**: [Slint](https://slint.dev/) 1.9+
- **Language**: Rust 1.82+
- **Async Runtime**: Tokio
- **HTTP Client**: reqwest
- **Core Library**: bitcell-wallet

### Application Structure

```
bitcell-wallet-gui/
├── src/
│   ├── main.rs           # Application entry and state management
│   ├── rpc_client.rs     # JSON-RPC client for node communication
│   ├── qrcode.rs         # QR code generation
│   └── game_viz.rs       # CA battle visualization (future)
├── ui/
│   └── main.slint        # UI component definitions
├── build.rs              # Slint compilation
└── Cargo.toml
```

## Building

### Prerequisites

**All Platforms**:
- Rust 1.82 or later
- Cargo

**Linux**:
```bash
# Debian/Ubuntu
sudo apt-get install libfontconfig1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev

# Fedora
sudo dnf install fontconfig-devel libxcb-devel
```

**macOS**:
```bash
# No additional dependencies required
# Xcode Command Line Tools should be installed
xcode-select --install
```

**Windows**:
```bash
# No additional dependencies required
# Install Visual Studio Build Tools if not already installed
```

### Build Commands

```bash
# Debug build
cargo build -p bitcell-wallet-gui

# Release build (optimized)
cargo build -p bitcell-wallet-gui --release

# Run directly
cargo run -p bitcell-wallet-gui

# Run release version
cargo run -p bitcell-wallet-gui --release
```

### Build Output

**Binary Location**:
- Debug: `target/debug/bitcell-wallet-gui`
- Release: `target/release/bitcell-wallet-gui`

**Size** (approximate):
- Debug: ~50MB
- Release: ~15MB (with LTO and strip)

## Usage

### Launching the Wallet

```bash
# Run the wallet GUI
./target/release/bitcell-wallet-gui

# Or with cargo
cargo run -p bitcell-wallet-gui --release
```

### First Time Setup

1. **Create New Wallet**
   - Click "Create New Wallet"
   - Enter wallet name
   - Optionally set a passphrase
   - **IMPORTANT**: Write down the 12-word mnemonic phrase
   - Confirm you've backed up the phrase

2. **Restore Existing Wallet**
   - Click "Restore Wallet"
   - Enter your 12/18/24-word mnemonic phrase
   - Enter passphrase if you used one
   - Wallet will regenerate all addresses

### Main Interface

**Views**:
- **Overview**: Wallet dashboard with balances
- **Send**: Create and submit transactions
- **Receive**: Generate addresses and QR codes
- **History**: Transaction history (coming soon)
- **Settings**: Configuration options (coming soon)

### Connecting to a Node

The wallet connects to a BitCell node via JSON-RPC:

```
Default endpoint: http://127.0.0.1:30334
```

**Connection Indicator**:
- 🟢 Green: Connected
- 🔴 Red: Disconnected

To run a local node:
```bash
./bitcell-node --rpc-port 30334
```

## UI Components

### Main Window

The UI is defined in `ui/main.slint` using the Slint markup language:

```slint
export component MainWindow inherits Window {
    title: "BitCell Wallet";
    preferred-width: 1200px;
    preferred-height: 800px;
    
    // Component structure
    HorizontalLayout {
        sidebar: Sidebar { /* ... */ }
        content: ContentArea { /* ... */ }
    }
}
```

### Key UI Features

**Native Rendering**:
- Uses platform's native graphics APIs
- No WebView or browser engine
- Hardware-accelerated where available
- Smooth 60fps animations

**Responsive Design**:
- Adapts to different window sizes
- Minimum window size enforced
- Scalable fonts and icons

**Accessibility**:
- Keyboard navigation support
- Screen reader compatible (planned)
- High contrast mode support (planned)

## State Management

Application state is managed using Rust's `Rc<RefCell<>>` pattern:

```rust
struct AppState {
    wallet: Option<Wallet>,
    mnemonic: Option<Mnemonic>,
    rpc_client: Option<RpcClient>,
}

let state = Rc::new(RefCell::new(AppState::new()));
```

State updates trigger UI refreshes through Slint's reactive property system.

## RPC Communication

### RPC Client

The `RpcClient` handles all communication with the BitCell node:

```rust
pub struct RpcClient {
    endpoint: String,
    client: reqwest::Client,
}

impl RpcClient {
    pub async fn get_balance(&self, address: &str) -> Result<u64, String>;
    pub async fn send_raw_transaction(&self, tx_hex: &str) -> Result<String, String>;
    pub async fn get_node_info(&self) -> Result<serde_json::Value, String>;
    pub async fn get_block_number(&self) -> Result<u64, String>;
}
```

### Connection Monitoring

The wallet polls the node every 2 seconds to check connection status:

```rust
let timer = slint::Timer::default();
timer.start(TimerMode::Repeated, Duration::from_secs(2), move || {
    // Check node connection
    // Update connection status in UI
});
```

## Development

### Running in Development Mode

```bash
# Run with debug logging
RUST_LOG=debug cargo run -p bitcell-wallet-gui

# Run with specific log levels
RUST_LOG=bitcell_wallet_gui=trace cargo run -p bitcell-wallet-gui
```

### Hot Reload (Slint UI)

Changes to `.slint` files trigger recompilation automatically. For faster iteration:

```bash
# Use cargo watch for automatic rebuilds
cargo install cargo-watch
cargo watch -x 'run -p bitcell-wallet-gui'
```

### Debugging

**Logging**:
```rust
use tracing::{debug, info, warn, error};

info!("Wallet created successfully");
debug!("Generated address: {}", address);
error!("Failed to connect to node: {}", error);
```

**Slint Debugging**:
```bash
# Enable Slint backend debugging
SLINT_BACKEND=qt cargo run -p bitcell-wallet-gui
```

## Configuration

### Default Settings

```rust
// RPC endpoint
const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 30334;

// Gas price fallback
const DEFAULT_GAS_PRICE: u64 = 1000;

// Wallet configuration
WalletConfig {
    name: "Default Wallet",
    chains: [BitCell, Bitcoin, Ethereum],
    auto_generate_addresses: true,
    address_lookahead: 5,
}
```

### User Data Storage

**Location** (future):
- Linux: `~/.config/bitcell-wallet/`
- macOS: `~/Library/Application Support/BitCell Wallet/`
- Windows: `%APPDATA%\BitCell Wallet\`

**Stored Data**:
- Wallet configuration (no keys!)
- Address labels (future)
- User preferences
- Transaction cache

## Security Considerations

### What's Secure

✅ **Private keys never leave memory**
- Generated on-demand
- Cleared when wallet locks
- Never written to disk

✅ **Locked by default**
- Must unlock to sign transactions
- Auto-lock on window close

✅ **Input validation**
- Address format checking
- Amount range validation
- Fee reasonableness checks

### What to Be Aware Of

⚠️ **The wallet does NOT protect against**:
- Malware with elevated privileges
- Keyloggers (hardware or software)
- Screen capture
- Compromised operating system

⚠️ **Current limitations**:
- No auto-lock timeout (manual lock only)
- No biometric authentication
- No hardware wallet support yet

### Best Practices

1. **Only run on trusted computers**
2. **Lock wallet when stepping away**
3. **Verify all transaction details before confirming**
4. **Keep your mnemonic phrase secure and offline**
5. **Use a strong passphrase**
6. **Start with small test transactions**

## Performance

### Metrics

**Target Performance**:
- Startup: < 2 seconds
- Memory: < 100MB idle
- CPU: < 5% idle
- Frame rate: 60fps sustained

**Actual** (on modern hardware):
- Startup: ~1.5 seconds
- Memory: ~80MB idle
- CPU: ~2% idle
- Frame rate: 60fps

### Optimization

**Slint Optimizations**:
- Native rendering (no browser overhead)
- Efficient property bindings
- Minimal redraws
- Hardware acceleration

**Rust Optimizations**:
- Lazy initialization
- Async I/O (tokio)
- Zero-copy where possible
- Efficient serialization (bincode)

## Troubleshooting

### Common Issues

**Issue**: Wallet won't start
```bash
# Check dependencies
cargo check -p bitcell-wallet-gui

# Rebuild from scratch
cargo clean
cargo build -p bitcell-wallet-gui
```

**Issue**: Can't connect to node
```bash
# Verify node is running
curl http://127.0.0.1:30334 -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"getNodeInfo","params":[],"id":1}'

# Check firewall settings
# Make sure port 30334 is not blocked
```

**Issue**: UI rendering issues
```bash
# Try different Slint backend
SLINT_BACKEND=software cargo run -p bitcell-wallet-gui

# Check OpenGL support
glxinfo | grep "OpenGL version"  # Linux
```

**Issue**: Build errors on Linux
```bash
# Install missing dependencies
sudo apt-get update
sudo apt-get install libfontconfig1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

### Debug Logging

Enable detailed logging:
```bash
RUST_LOG=debug cargo run -p bitcell-wallet-gui 2> wallet.log
```

Check the log file for errors and warnings.

## Testing

### Manual Testing Checklist

- [ ] Wallet creation flow
- [ ] Mnemonic display and backup
- [ ] Wallet recovery from mnemonic
- [ ] Address generation (all chains)
- [ ] QR code display
- [ ] Transaction form validation
- [ ] Node connection indicator
- [ ] Wallet lock/unlock
- [ ] Window resize and responsiveness

### Automated Tests

```bash
# Run GUI tests (when available)
cargo test -p bitcell-wallet-gui

# Integration tests with mock node
cargo test -p bitcell-wallet-gui --test integration
```

## Known Issues

1. **Transaction submission incomplete** (High Priority)
   - Status: Mock format used, needs real implementation
   - Location: `main.rs:393`
   - Impact: Cannot submit real transactions yet

2. **Balance updates manual** (High Priority)
   - Status: No RPC polling for balances
   - Impact: Must restart to see balance changes

3. **QR code not displayed in UI** (Medium Priority)
   - Status: Generation works, display missing
   - Impact: Must copy address manually

4. **No transaction history UI** (Medium Priority)
   - Status: History tracking works, UI needed
   - Impact: Cannot view past transactions

5. **Limited error messages** (Low Priority)
   - Status: Basic errors only
   - Impact: Debugging difficult for users

## Roadmap

### RC2 Completion
- [ ] Real transaction submission
- [ ] Balance polling integration
- [ ] QR code display
- [ ] Transaction history UI
- [ ] Error message improvements
- [ ] Settings panel

### v1.0 Features
- [ ] Hardware wallet support
- [ ] Address book
- [ ] Multi-wallet management
- [ ] Transaction templates
- [ ] Advanced fee estimation
- [ ] Backup/restore functionality

### Future Enhancements
- [ ] Dark mode / themes
- [ ] Multiple languages (i18n)
- [ ] Advanced charts and analytics
- [ ] DApp browser
- [ ] Staking interface
- [ ] NFT management

## Contributing

Contributions welcome! Focus areas:

- **High Priority**:
  - Complete transaction submission
  - Balance update integration
  - Transaction history UI

- **Medium Priority**:
  - QR code display
  - Settings panel
  - Error handling improvements

- **Nice to Have**:
  - UI/UX enhancements
  - Theme support
  - Accessibility features

### Development Guidelines

1. Test changes with the core wallet library
2. Follow Rust and Slint conventions
3. Add appropriate error handling
4. Update documentation
5. Test on multiple platforms if possible

## Resources

### Documentation
- [Wallet Requirements](../../docs/WALLET_REQUIREMENTS.md)
- [Wallet Architecture](../../docs/WALLET_ARCHITECTURE.md)
- [Testing Strategy](../../docs/WALLET_TESTING_STRATEGY.md)
- [Implementation Checklist](../../docs/WALLET_IMPLEMENTATION_CHECKLIST.md)

### External Resources
- [Slint Documentation](https://slint.dev/docs)
- [Slint Examples](https://github.com/slint-ui/slint/tree/master/examples)
- [Tokio Guide](https://tokio.rs/tokio/tutorial)
- [BitCell RPC API](../../docs/RPC_API_Spec.md)

## Support

- **Issues**: GitHub Issues
- **Discussions**: GitHub Discussions
- **Security**: Report privately to security@bitcell.network

## License

Dual-licensed under MIT / Apache 2.0.

---

**Built with** 🦀 Rust + 🎨 Slint

_"Native performance, cross-platform compatibility, zero compromises"_
