#!/bin/bash

# Test validator with visible output

echo "Starting test validator with visible console output..."
echo "This will show all println! statements"
echo ""

RUST_LOG=info ./target/release/bitcell-node validator \
  --port 29999 \
  --rpc-port 29998 \
  --data-dir /tmp/test-validator-output

