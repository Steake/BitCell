# Building from Source

Detailed guide for building BitCell from source code, including optimization options and development builds.

## Why Build from Source?

- Get the absolute latest features
- Optimize for your specific hardware
- Contribute to development
- Audit the code yourself

## Prerequisites

### Required Tools

1. **Rust 1.82+**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup toolchain install 1.82
rustup default 1.82
```

2. **Build Tools**

**Ubuntu/Debian:**
```bash
sudo apt update && sudo apt install -y \
  build-essential \
  pkg-config \
  libssl-dev \
  libclang-dev \
  clang \
  cmake \
  git
```

**Fedora/RHEL:**
```bash
sudo dnf install -y \
  gcc gcc-c++ \
  pkgconfig \
  openssl-devel \
  clang-devel \
  cmake \
  git
```

**macOS:**
```bash
xcode-select --install
brew install cmake pkg-config openssl
```

3. **Git**

```bash
# Verify git is installed
git --version
```

## Clone the Repository

```bash
# Clone with full history
git clone https://github.com/Steake/BitCell.git
cd BitCell

# Or clone with shallow history (faster)
git clone --depth 1 https://github.com/Steake/BitCell.git
cd BitCell
```

## Build Configurations

### Release Build (Production)

Fully optimized for performance:

```bash
cargo build --release

# Takes 5-15 minutes depending on hardware
# Binaries in: target/release/
```

Release build options in `Cargo.toml`:
```toml
[profile.release]
opt-level = 3           # Maximum optimization
lto = "fat"             # Link-time optimization
codegen-units = 1       # Better optimization, slower compilation
panic = "abort"         # Smaller binary
strip = true            # Remove debug symbols
```

### Debug Build (Development)

Faster compilation, with debug info:

```bash
cargo build

# Takes 2-5 minutes
# Binaries in: target/debug/
```

### Optimized Dev Build

Balance between speed and compilation time:

```bash
cargo build --profile dev-optimized

# Or modify Cargo.toml:
[profile.dev]
opt-level = 2           # Some optimization
incremental = true      # Faster rebuilds
```

## Component-Specific Builds

Build individual crates:

```bash
# Build just the node
cargo build --release -p bitcell-node

# Build just the wallet
cargo build --release -p bitcell-wallet

# Build just the admin console
cargo build --release -p bitcell-admin

# Build multiple specific crates
cargo build --release -p bitcell-node -p bitcell-wallet
```

## Feature Flags

### Standard Features

```bash
# Full node with all features (default)
cargo build --release --all-features

# Minimal build (validator only)
cargo build --release --no-default-features --features validator

# With GPU acceleration (requires CUDA or OpenCL)
cargo build --release --features gpu

# With hardware wallet support
cargo build --release --features hardware-wallet
```

### Available Features

| Feature | Description | Default |
|---------|-------------|---------|
| `validator` | Validator node capability | ✓ |
| `miner` | Mining/tournament participation | ✓ |
| `rpc` | JSON-RPC server | ✓ |
| `ws` | WebSocket API | ✓ |
| `admin` | Admin console | ✓ |
| `gpu` | GPU-accelerated CA simulation | ✗ |
| `hardware-wallet` | Ledger/Trezor support | ✗ |
| `metrics` | Prometheus metrics export | ✓ |

Example custom build:

```bash
# Lightweight validator (no mining, no admin)
cargo build --release \
  --no-default-features \
  --features "validator,rpc,ws"
```

## Platform-Specific Optimizations

### Linux (x86_64)

```bash
# Target native CPU for maximum performance
RUSTFLAGS="-C target-cpu=native" cargo build --release

# With link-time optimization
RUSTFLAGS="-C target-cpu=native -C link-arg=-fuse-ld=lld" \
  cargo build --release
```

### macOS (Apple Silicon)

```bash
# Native ARM64 build
cargo build --release --target aarch64-apple-darwin

# Universal binary (Intel + ARM)
rustup target add x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
lipo -create \
  target/aarch64-apple-darwin/release/bitcell-node \
  target/x86_64-apple-darwin/release/bitcell-node \
  -output bitcell-node
```

### Cross-Compilation

Build for different architectures:

```bash
# Install target
rustup target add aarch64-unknown-linux-gnu

# Install cross-compilation tools
sudo apt install gcc-aarch64-linux-gnu

# Build
cargo build --release --target aarch64-unknown-linux-gnu
```

## Testing the Build

### Run Tests

```bash
# Run all tests
cargo test --all

# Run tests with output
cargo test --all -- --nocapture

# Run specific crate tests
cargo test -p bitcell-crypto
cargo test -p bitcell-ca

# Run integration tests
cargo test --test '*'
```

### Run Benchmarks

```bash
# Run all benchmarks
cargo bench

# Specific benchmark
cargo bench --bench ca_evolution

# Results in: target/criterion/
```

### Verify Binary

```bash
# Check binary size
ls -lh target/release/bitcell-node

# Run version check
target/release/bitcell-node --version

# Quick smoke test
target/release/bitcell-node --help
```

## Installation

### System-wide Installation

```bash
# Copy binaries to /usr/local/bin
sudo cp target/release/bitcell-{node,wallet,wallet-gui,admin} \
  /usr/local/bin/

# Verify installation
which bitcell-node
bitcell-node --version
```

### User Installation

```bash
# Create bin directory in home
mkdir -p ~/.local/bin

# Copy binaries
cp target/release/bitcell-* ~/.local/bin/

# Add to PATH (add to ~/.bashrc or ~/.zshrc)
export PATH="$HOME/.local/bin:$PATH"
```

### Creating System Service

Create systemd service for automatic startup:

```bash
sudo tee /etc/systemd/system/bitcell-node.service > /dev/null <<EOF
[Unit]
Description=BitCell Node
After=network.target

[Service]
Type=simple
User=$USER
ExecStart=/usr/local/bin/bitcell-node start --validator
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl enable bitcell-node
sudo systemctl start bitcell-node

# Check status
sudo systemctl status bitcell-node
```

## Build Troubleshooting

### Slow Compilation

Speed up builds:

```bash
# Use mold linker (much faster)
cargo install mold
RUSTFLAGS="-C link-arg=-fuse-ld=mold" cargo build --release

# Or use lld
RUSTFLAGS="-C link-arg=-fuse-ld=lld" cargo build --release

# Enable parallel codegen
CARGO_BUILD_JOBS=8 cargo build --release
```

### Out of Memory

Reduce memory usage during compilation:

```bash
# Reduce codegen units (uses less memory)
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 cargo build --release

# Or disable LTO temporarily
CARGO_PROFILE_RELEASE_LTO=false cargo build --release
```

### Missing Dependencies

If you get dependency errors:

```bash
# Update cargo registry
cargo update

# Clean and rebuild
cargo clean
cargo build --release

# Check for system library issues
pkg-config --list-all | grep ssl
```

## Development Setup

For active development:

```bash
# Install development tools
cargo install cargo-watch cargo-expand cargo-edit

# Watch mode - rebuild on file changes
cargo watch -x build

# Check code without building
cargo check

# Format code
cargo fmt --all

# Lint code
cargo clippy --all -- -D warnings

# Generate documentation
cargo doc --no-deps --open
```

## Next Steps

- **[Running a Node](../node/running-node.md)** - Start your built node
- **[Development Setup](../development/dev-setup.md)** - Configure dev environment
- **[Contributing](../development/contributing.md)** - Contribute to BitCell

## Performance Tips

### Optimize for Your CPU

```bash
# Find your CPU features
rustc --print target-cpus
rustc --print target-features

# Build with specific features
RUSTFLAGS="-C target-cpu=native -C target-feature=+avx2" \
  cargo build --release
```

### Profile-Guided Optimization

```bash
# 1. Build instrumented binary
RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data" \
  cargo build --release

# 2. Run typical workload
target/release/bitcell-node benchmark

# 3. Rebuild with profile data
RUSTFLAGS="-Cprofile-use=/tmp/pgo-data -Cllvm-args=-pgo-warn-missing-function" \
  cargo build --release
```

### Build Cache

Use sccache to cache compilation artifacts:

```bash
cargo install sccache
export RUSTC_WRAPPER=sccache
cargo build --release

# Check cache statistics
sccache --show-stats
```
