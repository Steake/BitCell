# Installation

Complete installation guide for BitCell node software and tools.

## System Requirements

### Minimum Requirements

- **OS**: Linux (Ubuntu 20.04+), macOS (11+), or Windows WSL2
- **CPU**: 4 cores / 8 threads
- **RAM**: 8GB
- **Storage**: 100GB SSD (grows with blockchain)
- **Network**: 10 Mbps upload/download

### Recommended Requirements

- **CPU**: 8+ cores / 16+ threads
- **RAM**: 16GB+
- **Storage**: 500GB+ NVMe SSD
- **Network**: 100+ Mbps with low latency

### For Mining

Additional requirements for tournament participation:

- **CPU**: 16+ cores (CA simulation is CPU-intensive)
- **RAM**: 32GB+ (for large grid simulations)
- **Storage**: 1TB+ (for storing battle replays)

## Installation Methods

### Method 1: Pre-built Binaries (Easiest)

Download and install pre-compiled binaries:

**Linux (x86_64):**

```bash
wget https://github.com/Steake/BitCell/releases/download/v0.3.0/bitcell-linux-x86_64.tar.gz
tar xzf bitcell-linux-x86_64.tar.gz
sudo install bitcell-* /usr/local/bin/
```

**macOS (Apple Silicon):**

```bash
wget https://github.com/Steake/BitCell/releases/download/v0.3.0/bitcell-macos-aarch64.tar.gz
tar xzf bitcell-macos-aarch64.tar.gz
sudo install bitcell-* /usr/local/bin/
```

**macOS (Intel):**

```bash
wget https://github.com/Steake/BitCell/releases/download/v0.3.0/bitcell-macos-x86_64.tar.gz
tar xzf bitcell-macos-x86_64.tar.gz
sudo install bitcell-* /usr/local/bin/
```

### Method 2: Build from Source

Building from source gives you the latest features and optimizations.

#### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version  # Should be 1.82+
```

#### 2. Install Dependencies

**Ubuntu/Debian:**

```bash
sudo apt update
sudo apt install -y \
  build-essential \
  pkg-config \
  libssl-dev \
  git \
  clang \
  cmake
```

**macOS:**

```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install cmake pkg-config openssl
```

#### 3. Clone and Build

```bash
# Clone repository
git clone https://github.com/Steake/BitCell.git
cd BitCell

# Build in release mode (optimized, ~5-10 minutes)
cargo build --release

# Binaries will be in target/release/
# - bitcell-node       (validator/miner node)
# - bitcell-wallet     (CLI wallet)
# - bitcell-wallet-gui (GUI wallet)
# - bitcell-admin      (admin console)
```

#### 4. Install Binaries

```bash
# Copy to system PATH
sudo cp target/release/bitcell-{node,wallet,wallet-gui,admin} /usr/local/bin/

# Or add to PATH in your shell config
echo 'export PATH="$HOME/BitCell/target/release:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Method 3: Docker (Experimental)

Run BitCell in Docker containers:

```bash
# Pull the official image
docker pull bitcell/node:latest

# Run a validator node
docker run -d \
  --name bitcell-validator \
  -p 30303:30303 \
  -p 8545:8545 \
  -v bitcell-data:/root/.bitcell \
  bitcell/node:latest \
  start --validator --rpc-addr 0.0.0.0:8545

# Check logs
docker logs -f bitcell-validator
```

## Post-Installation

### Verify Installation

```bash
# Check versions
bitcell-node --version
bitcell-wallet --version
bitcell-admin --version

# Should output:
# bitcell-node 0.3.0
# bitcell-wallet 0.3.0
# bitcell-admin 0.3.0
```

### Initialize Node

```bash
# Create default configuration
bitcell-node init

# Configuration files will be created at:
# - Linux/macOS: ~/.bitcell/
# - Windows: %APPDATA%\BitCell\
```

Configuration files:

- `config.toml` - Node configuration
- `genesis.json` - Genesis block
- `peers.json` - Bootstrap peer list

### Create Data Directory

```bash
# Default data directory
mkdir -p ~/.bitcell/{blocks,state,logs}

# Or specify custom location
export BITCELL_HOME=/path/to/custom/location
bitcell-node init --data-dir $BITCELL_HOME
```

## Updating

### Update Pre-built Binary

```bash
# Download latest release
wget https://github.com/Steake/BitCell/releases/latest/download/bitcell-linux-x86_64.tar.gz

# Replace existing binaries
tar xzf bitcell-linux-x86_64.tar.gz
sudo install bitcell-* /usr/local/bin/

# Restart node
sudo systemctl restart bitcell-node
```

### Update from Source

```bash
cd BitCell
git pull origin master
cargo build --release
sudo cp target/release/bitcell-* /usr/local/bin/
```

## Uninstallation

```bash
# Stop node
bitcell-node stop

# Remove binaries
sudo rm /usr/local/bin/bitcell-*

# Remove data (WARNING: This deletes your blockchain data)
rm -rf ~/.bitcell

# Remove source (if built from source)
rm -rf ~/BitCell
```

## Next Steps

- **[Building from Source](./building.md)** - Detailed build instructions
- **[Running a Node](../node/running-node.md)** - Start your node
- **[Network Configuration](../node/network-config.md)** - Configure networking

## Troubleshooting

### Build Errors

**Error: "rustc version too old"**

```bash
rustup update stable
rustc --version  # Verify 1.82+
```

**Error: "linker `cc` not found"**

Install build tools:

```bash
# Ubuntu/Debian
sudo apt install build-essential

# macOS
xcode-select --install
```

**Error: "could not find OpenSSL"**

```bash
# Ubuntu/Debian
sudo apt install libssl-dev pkg-config

# macOS
brew install openssl
export OPENSSL_DIR=$(brew --prefix openssl)
```

### Runtime Issues

**Issue: "Cannot write to data directory"**

Fix permissions:

```bash
sudo chown -R $USER:$USER ~/.bitcell
chmod 700 ~/.bitcell
```

**Issue: "Port already in use"**

Change default ports in `~/.bitcell/config.toml`:

```toml
[network]
p2p_port = 30304  # Changed from 30303
rpc_port = 8546   # Changed from 8545
```

For more help, see [Troubleshooting Guide](../appendix/troubleshooting.md).
