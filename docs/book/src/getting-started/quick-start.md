# Quick Start

Get up and running with BitCell in under 5 minutes!

## Prerequisites

- Linux, macOS, or WSL2 on Windows
- 8GB+ RAM
- 10GB free disk space
- Internet connection

## Installation

### Option 1: Download Pre-built Binary (Recommended)

```bash
# Download the latest release
curl -LO https://github.com/Steake/BitCell/releases/latest/download/bitcell-linux-x86_64.tar.gz

# Extract
tar xzf bitcell-linux-x86_64.tar.gz

# Move to PATH
sudo mv bitcell-* /usr/local/bin/

# Verify installation
bitcell-node --version
```

### Option 2: Build from Source

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone https://github.com/Steake/BitCell.git
cd BitCell

# Build
cargo build --release

# Binaries will be in target/release/
```

## Running Your First Node

### Start a Validator Node

```bash
# Initialize node configuration
bitcell-node init --network testnet

# Start the node
bitcell-node start --validator

# Check node status
bitcell-node status
```

You should see output like:

```
✓ Node running
✓ Connected to 12 peers
✓ Syncing blocks (height: 5432/10890)
✓ EBSL trust score: 0.40 (below eligibility threshold)
```

### Create a Wallet

```bash
# Create a new wallet
bitcell-wallet create --name my-wallet

# This will output:
# ✓ Wallet created: my-wallet
# ✓ Address: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
# ⚠ IMPORTANT: Save your recovery phrase in a secure location!
#
# Recovery phrase:
# abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about

# Get your balance
bitcell-wallet balance

# Output: 0 CELL
```

## Send Your First Transaction

### Request Testnet Tokens

```bash
# Get tokens from the faucet
curl -X POST https://faucet.testnet.bitcell.network/request \
  -H "Content-Type: application/json" \
  -d '{"address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"}'

# Wait ~30 seconds for confirmation
bitcell-wallet balance

# Output: 100 CELL
```

### Send Tokens

```bash
# Send 10 CELL to another address
bitcell-wallet send \
  --to 0x1234567890abcdef1234567890abcdef12345678 \
  --amount 10 \
  --fee 0.001

# Output:
# ✓ Transaction sent
# ✓ Hash: 0xabcd...ef12
# ⏳ Waiting for confirmation...
# ✓ Confirmed in block 10892
```

## What's Next?

Now that you have BitCell running, explore these topics:

- **[Run a Miner Node](../node/miner-setup.md)** - Participate in tournaments
- **[Use the GUI Wallet](../wallet/gui-wallet.md)** - Visual wallet interface
- **[Deploy a Smart Contract](../contracts/deployment.md)** - Build dApps
- **[API Reference](../api/overview.md)** - Integrate programmatically

## Common Issues

### "Cannot connect to peers"

Make sure your firewall allows:
- TCP port 30303 (P2P networking)
- UDP port 30303 (DHT discovery)

```bash
# Linux: Open ports with ufw
sudo ufw allow 30303/tcp
sudo ufw allow 30303/udp
```

### "Trust score below threshold"

New nodes start with trust score 0.40, below the eligibility threshold of 0.75. Build reputation by:
- Running a validator consistently
- Submitting valid blocks
- Participating in tournaments

See [EBSL Trust System](../concepts/ebsl-trust.md) for details.

### "Insufficient funds"

You need CELL tokens for:
- Transaction fees
- Miner bonds (for tournament participation)

Get testnet tokens from the [faucet](https://faucet.testnet.bitcell.network).

## Need Help?

- Check the [FAQ](../appendix/faq.md)
- Review [Troubleshooting](../appendix/troubleshooting.md)
- Ask on [GitHub Issues](https://github.com/Steake/BitCell/issues)
