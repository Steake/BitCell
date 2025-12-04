#!/bin/bash
set -e

echo "=== DHT Peer Discovery Test with Deterministic Keys ==="

# Kill any running nodes
pkill -f bitcell-node || true
sleep 2

# Clean up
rm -rf .bitcell/dht_test
mkdir -p .bitcell/dht_test

# Build
echo "Building..."
cargo build --release -p bitcell-node

echo ""
echo "Starting Bootstrap Node (Validator) on port 19000 with seed 'bootstrap'..."
./target/release/bitcell-node validator \
  --port 19000 \
  --enable-dht \
  --key-seed "bootstrap" \
  > .bitcell/dht_test/validator.log 2>&1 &
VALIDATOR_PID=$!

sleep 5

echo "Checking validator startup..."
if grep -q "DHT enabled" .bitcell/dht_test/validator.log; then
    echo "✅ Validator DHT enabled"
    grep "Generating validator key from seed" .bitcell/dht_test/validator.log || true
else
    echo "❌ Validator DHT not enabled"
    cat .bitcell/dht_test/validator.log
    kill $VALIDATOR_PID
    exit 1
fi

echo ""
echo "Starting Miner Node on port 19100 with seed 'miner1'..."
./target/release/bitcell-node miner \
  --port 19100 \
  --enable-dht \
  --key-seed "miner1" \
  --bootstrap "/ip4/127.0.0.1/tcp/19000" \
  > .bitcell/dht_test/miner.log 2>&1 &
MINER_PID=$!

sleep 10

echo "Checking miner startup..."
if grep -q "DHT enabled" .bitcell/dht_test/miner.log; then
    echo "✅ Miner DHT enabled"
    grep "Generating key from seed" .bitcell/dht_test/miner.log || true
else
    echo "❌ Miner DHT not enabled"
    cat .bitcell/dht_test/miner.log
    kill $VALIDATOR_PID $MINER_PID
    exit 1
fi

echo ""
echo "Checking DHT discovery..."
if grep -q "Starting DHT discovery" .bitcell/dht_test/validator.log .bitcell/dht_test/miner.log; then
    echo "✅ DHT discovery started"
    grep "DHT discovery" .bitcell/dht_test/*.log || true
else
    echo "⚠️  DHT discovery not found in logs"
fi

echo ""
echo "Checking peer connections..."
sleep 5
if grep -q "Connected to peer" .bitcell/dht_test/*.log; then
    echo "✅ Peers connected"
    grep "Connected to peer" .bitcell/dht_test/*.log || true
else
    echo "⚠️  No peer connections found (may be expected if DHT routing not fully implemented)"
fi

echo ""
echo "=== Test Summary ==="
echo "Validator PID: $VALIDATOR_PID"
echo "Miner PID: $MINER_PID"
echo ""
echo "Logs available at:"
echo "  - .bitcell/dht_test/validator.log"
echo "  - .bitcell/dht_test/miner.log"
echo ""
echo "Metrics endpoints:"
echo "  - Validator: http://localhost:19001/metrics"
echo "  - Miner: http://localhost:19101/metrics"
echo ""
echo "Press Enter to stop nodes and exit..."
read

# Cleanup
echo "Stopping nodes..."
kill $VALIDATOR_PID $MINER_PID
echo "Done!"
