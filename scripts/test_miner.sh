#!/bin/bash

# Test Miner with visible output

echo "Starting test Miner with visible console output..."
echo "This will show all println! statements"
echo ""

RUST_LOG=info ./target/release/bitcell-node miner \
  --port 30002 \
  --rpc-port 30003 \
  --data-dir /tmp/test-miner-output
