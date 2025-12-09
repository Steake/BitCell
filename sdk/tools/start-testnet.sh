#!/bin/bash
# BitCell Local Testnet Launcher
# Starts a local single-node testnet for contract development

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TESTNET_DIR="/tmp/bitcell-testnet"

echo "🚀 BitCell Local Testnet Launcher"
echo "=================================="
echo ""

# Check if bitcell-node is built
if [ ! -f "$PROJECT_ROOT/target/debug/bitcell-node" ] && [ ! -f "$PROJECT_ROOT/target/release/bitcell-node" ]; then
    echo "⚠️  BitCell node not found. Building..."
    cd "$PROJECT_ROOT"
    cargo build -p bitcell-node
fi

# Clean up old testnet data
if [ -d "$TESTNET_DIR" ]; then
    echo "🧹 Cleaning up old testnet data..."
    rm -rf "$TESTNET_DIR"
fi

# Create testnet directory
mkdir -p "$TESTNET_DIR"

echo "📝 Creating testnet configuration..."
cat > "$TESTNET_DIR/config.json" << EOF
{
  "network": {
    "listen_address": "127.0.0.1:9944",
    "bootstrap_nodes": []
  },
  "consensus": {
    "tournament_duration": 10,
    "min_participants": 1
  },
  "state": {
    "data_dir": "$TESTNET_DIR/state"
  },
  "rpc": {
    "address": "127.0.0.1:8545",
    "cors_origins": ["*"]
  }
}
EOF

echo "🔑 Generating testnet genesis accounts..."
# In a real implementation, this would generate accounts with initial balances
# For now, we'll create a simple genesis file

mkdir -p "$TESTNET_DIR/state"

echo "✨ Starting local testnet node..."
echo ""
echo "RPC endpoint:  http://127.0.0.1:8545"
echo "Node endpoint: http://127.0.0.1:9944"
echo ""
echo "Press Ctrl+C to stop the testnet"
echo ""

# Start the node (if bitcell-node binary exists)
if [ -f "$PROJECT_ROOT/target/debug/bitcell-node" ]; then
    exec "$PROJECT_ROOT/target/debug/bitcell-node" --config "$TESTNET_DIR/config.json" --dev
elif [ -f "$PROJECT_ROOT/target/release/bitcell-node" ]; then
    exec "$PROJECT_ROOT/target/release/bitcell-node" --config "$TESTNET_DIR/config.json" --dev
else
    echo "ℹ️  Note: bitcell-node binary will be available after implementing the node CLI"
    echo "ℹ️  For now, this script sets up the configuration for when it's ready"
    echo ""
    echo "✅ Testnet configuration created at: $TESTNET_DIR"
    echo ""
    echo "To start manually once node is built:"
    echo "  cargo run -p bitcell-node -- --config $TESTNET_DIR/config.json --dev"
fi
