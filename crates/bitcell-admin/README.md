# BitCell Admin Console

A comprehensive web-based administrative interface for managing and monitoring BitCell blockchain nodes.

## Features

### 🎛️ Node Management
- **Register and manage multiple nodes** (validators, miners, full nodes)
- **Start/stop nodes** remotely via web interface
- **Real-time status monitoring** with automatic updates
- **Node health checks** and diagnostics

### 📊 Metrics & Monitoring
- **Chain Metrics**: Block height, transactions, pending pool, block times
- **Network Metrics**: Peer connections, bandwidth usage, message throughput
- **EBSL Metrics**: Active miners, banned miners, trust scores, slashing events
- **System Metrics**: CPU usage, memory usage, disk usage, uptime

### 🚀 Deployment Management
- **Automated node deployment** with configurable parameters
- **Multi-node deployment** for testnets and production
- **Deployment status tracking** and history
- **Configuration management** with validation

### 🧪 Testing Utilities
- **Battle simulation testing** with custom glider patterns
- **Transaction testing** for stress testing and validation
- **Network connectivity testing** for peer discovery
- **Performance benchmarking** tools

### ⚙️ Configuration
- **Network configuration**: Listen addresses, bootstrap peers, max peers
- **Consensus configuration**: Battle steps, tournament rounds, block time
- **EBSL configuration**: Evidence thresholds, slash percentages, decay rates
- **Economics configuration**: Rewards, halving intervals, gas pricing

## Quick Start

### Running the Admin Console

```bash
# Start on default port (8080)
cargo run -p bitcell-admin

# Start on custom port
cargo run -p bitcell-admin -- 0.0.0.0:9999
```

### Access the Dashboard

Open your browser and navigate to:
```
http://localhost:8080
```

## API Endpoints

### Node Management
- `GET /api/nodes` - List all nodes
- `GET /api/nodes/:id` - Get node details
- `POST /api/nodes/:id/start` - Start a node
- `POST /api/nodes/:id/stop` - Stop a node

### Metrics
- `GET /api/metrics` - Get all metrics
- `GET /api/metrics/chain` - Chain-specific metrics
- `GET /api/metrics/network` - Network-specific metrics

### Deployment
- `POST /api/deployment/deploy` - Deploy new nodes
- `GET /api/deployment/status` - Get deployment status

### Configuration
- `GET /api/config` - Get current configuration
- `POST /api/config` - Update configuration

### Testing
- `POST /api/test/battle` - Run battle simulation
- `POST /api/test/transaction` - Send test transaction

## API Examples

### Deploy Validator Nodes

```bash
curl -X POST http://localhost:8080/api/deployment/deploy \
  -H "Content-Type: application/json" \
  -d '{
    "node_type": "validator",
    "count": 3,
    "config": {
      "network": "testnet",
      "log_level": "info",
      "port_start": 9000
    }
  }'
```

### Run Battle Test

```bash
curl -X POST http://localhost:8080/api/test/battle \
  -H "Content-Type: application/json" \
  -d '{
    "glider_a": "Standard",
    "glider_b": "Heavyweight",
    "steps": 1000
  }'
```

### Update Configuration

```bash
curl -X POST http://localhost:8080/api/config \
  -H "Content-Type: application/json" \
  -d '{
    "network": {
      "listen_addr": "0.0.0.0:9000",
      "bootstrap_peers": ["127.0.0.1:9001"],
      "max_peers": 50
    },
    "consensus": {
      "battle_steps": 1000,
      "tournament_rounds": 5,
      "block_time": 6
    },
    "ebsl": {
      "evidence_threshold": 0.7,
      "slash_percentage": 0.1,
      "decay_rate": 0.95
    },
    "economics": {
      "initial_reward": 50000000,
      "halving_interval": 210000,
      "base_gas_price": 1000
    }
  }'
```

## Architecture

```
bitcell-admin/
├── src/
│   ├── lib.rs              # Main library interface
│   ├── main.rs             # Binary entry point
│   ├── api/                # REST API endpoints
│   │   ├── mod.rs          # API types and core
│   │   ├── nodes.rs        # Node management
│   │   ├── metrics.rs      # Metrics endpoints
│   │   ├── deployment.rs   # Deployment endpoints
│   │   ├── config.rs       # Configuration endpoints
│   │   └── test.rs         # Testing utilities
│   ├── web/                # Web interface
│   │   ├── mod.rs          # Template engine setup
│   │   └── dashboard.rs    # Dashboard HTML/JS
│   ├── deployment.rs       # Deployment manager
│   ├── config.rs           # Configuration manager
│   └── metrics.rs          # Metrics collector
└── static/                 # Static assets (CSS, JS, images)
```

## Security Considerations

⚠️ **CRITICAL SECURITY WARNING** ⚠️

**NO AUTHENTICATION IS CURRENTLY IMPLEMENTED**

The admin console currently allows **unrestricted access** to all endpoints. This is a **critical security vulnerability**.

**DO NOT expose this admin console to any network (including localhost) in production without implementing authentication first.**

For production deployments, you MUST:

1. **Implement authentication** before exposing to any network
2. **Use HTTPS/TLS** for all communication (never HTTP in production)
3. **Restrict network access** via firewall rules, VPN, or IP allowlisting
4. **Use strong passwords** and rotate them regularly
5. **Enable comprehensive audit logging** for all administrative actions
6. **Implement API rate limiting** to prevent abuse
7. **Run with least-privilege** user accounts (never as root)

## Development

### Building

```bash
cargo build -p bitcell-admin
```

### Testing

```bash
cargo test -p bitcell-admin
```

### Running in Development

```bash
# With auto-reload (requires cargo-watch)
cargo watch -x 'run -p bitcell-admin'
```

## Future Enhancements

- [ ] Authentication and authorization (JWT tokens)
- [ ] WebSocket support for real-time updates
- [ ] Advanced charting and visualization
- [ ] Log aggregation and search
- [ ] Automated health checks and alerting
- [ ] Backup and restore functionality
- [ ] Multi-chain support
- [ ] Mobile-responsive UI improvements

## License

Same as BitCell project

## Contributing

Contributions welcome! Please follow the BitCell contribution guidelines.
