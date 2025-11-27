#!/bin/bash
# Kill any existing nodes
pkill -f bitcell-node

# Run validator
echo "Starting validator..."
./target/release/bitcell-node validator --port 19000 &
PID=$!

# Wait for startup
sleep 5

# Check metrics
echo "Checking metrics..."
curl -v http://127.0.0.1:19001/metrics

# Kill validator
kill $PID
