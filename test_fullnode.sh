#!/bin/bash

# Test FullNode with visible output

echo "Starting test FullNode with visible console output..."
echo "This will show all println! statements"
echo ""

# FullNode uses ValidatorNode logic under the hood, so we expect similar output
RUST_LOG=info ./target/release/bitcell-node full-node \
  --port 30000 \
  --rpc-port 30001 \
  --data-dir /tmp/test-fullnode-output
