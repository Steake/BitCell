#!/bin/bash
set -e

# Default values
REGION=${REGION:-"default"}
NODE_ID=${NODE_ID:-"node-1"}
P2P_PORT=${P2P_PORT:-9000}
RPC_PORT=${RPC_PORT:-8545}
METRICS_PORT=${METRICS_PORT:-9090}
DATA_DIR=${DATA_DIR:-"/data/bitcell"}
LOG_LEVEL=${LOG_LEVEL:-"info"}
BOOTSTRAP_NODES=${BOOTSTRAP_NODES:-""}

echo "Starting BitCell node..."
echo "Region: $REGION"
echo "Node ID: $NODE_ID"
echo "P2P Port: $P2P_PORT"
echo "RPC Port: $RPC_PORT"
echo "Metrics Port: $METRICS_PORT"
echo "Data Directory: $DATA_DIR"
echo "Bootstrap Nodes: $BOOTSTRAP_NODES"

# Create data directory if it doesn't exist
mkdir -p "$DATA_DIR"

# Build command arguments
ARGS=(
    "--data-dir" "$DATA_DIR"
    "--port" "$P2P_PORT"
    "--rpc-port" "$RPC_PORT"
    "--metrics-port" "$METRICS_PORT"
    "--log-level" "$LOG_LEVEL"
)

# Add bootstrap nodes if provided
if [ -n "$BOOTSTRAP_NODES" ]; then
    IFS=',' read -ra NODES <<< "$BOOTSTRAP_NODES"
    for node in "${NODES[@]}"; do
        ARGS+=("--bootstrap" "$node")
    done
fi

# Execute the node
exec /usr/local/bin/bitcell-node "${ARGS[@]}"
