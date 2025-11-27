#!/bin/bash

# Quick manual cleanup - just delete the state file and you're done!

echo "🧹 Clearing BitCell admin state..."

# Kill existing processes
echo "Killing existing BitCell processes..."
pkill -f bitcell-node || true
pkill -f bitcell-admin || true
# Wait for ports to free up
sleep 2

# Clear admin deployment state
rm -rf .bitcell/admin/*

# Clear temp node data
rm -rf /tmp/bitcell/*

echo "✅ State cleared!"
echo ""
echo "Now restart admin console:"
cargo run --release -p bitcell-admin
