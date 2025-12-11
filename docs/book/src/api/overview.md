# API Overview

BitCell provides multiple API interfaces for interacting with the blockchain network programmatically.

## Available APIs

### JSON-RPC API

The primary interface for blockchain interactions, compatible with Ethereum tooling.

- **Endpoint**: `http://localhost:8545` (default)
- **Protocol**: JSON-RPC 2.0 over HTTP/HTTPS
- **Methods**: `eth_*`, `bitcell_*`, `net_*`, `web3_*`
- **Use cases**: Transactions, queries, smart contracts

[Read JSON-RPC Documentation →](./json-rpc.md)

### WebSocket API

Real-time event streaming and subscriptions.

- **Endpoint**: `ws://localhost:8546` (default)
- **Protocol**: WebSocket with JSON-RPC 2.0
- **Features**: Block subscriptions, log filtering, transaction notifications
- **Use cases**: Live updates, event monitoring, reactive applications

[Read WebSocket Documentation →](./websocket.md)

### REST API

Simple HTTP REST endpoints for common queries.

- **Endpoint**: `http://localhost:8080` (default)
- **Protocol**: REST over HTTP/HTTPS
- **Features**: Explorer data, statistics, health checks
- **Use cases**: Simple queries, monitoring, dashboards

[Read REST Documentation →](./rest.md)

## Quick Start

### Making Your First API Call

**Using curl:**

```bash
# Get current block number
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_blockNumber",
    "params": [],
    "id": 1
  }'

# Response:
# {"jsonrpc":"2.0","id":1,"result":"0x2a5f"}
```

**Using JavaScript (web3.js):**

```javascript
const Web3 = require('web3');
const web3 = new Web3('http://localhost:8545');

// Get block number
const blockNumber = await web3.eth.getBlockNumber();
console.log('Current block:', blockNumber);

// Get balance
const balance = await web3.eth.getBalance('0x742d...');
console.log('Balance:', web3.utils.fromWei(balance, 'ether'), 'CELL');
```

**Using Python (web3.py):**

```python
from web3 import Web3

# Connect to node
w3 = Web3(Web3.HTTPProvider('http://localhost:8545'))

# Get block number
block_number = w3.eth.block_number
print(f'Current block: {block_number}')

# Get balance
balance = w3.eth.get_balance('0x742d...')
print(f'Balance: {w3.from_wei(balance, "ether")} CELL')
```

## Authentication

### Public Endpoints

Most query methods are publicly accessible:
- Block data
- Transaction data
- Account balances
- Network info

### Protected Endpoints

Some methods require authentication:
- `personal_*` methods (wallet management)
- `admin_*` methods (node administration)
- `miner_*` methods (mining control)

**API Key Authentication:**

```bash
# Set API key in config
echo "rpc_api_key = \"your-secret-key\"" >> ~/.bitcell/config.toml

# Use in requests
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-secret-key" \
  -d '{"jsonrpc":"2.0","method":"personal_listAccounts","params":[],"id":1}'
```

## Rate Limiting

Default rate limits:
- **Public methods**: 100 requests/minute per IP
- **Authenticated methods**: 1000 requests/minute per API key

Configure in `~/.bitcell/config.toml`:

```toml
[rpc]
rate_limit_per_minute = 100
rate_limit_burst = 10
```

## CORS Configuration

For browser-based applications:

```toml
[rpc]
cors_origins = ["http://localhost:3000", "https://yourdapp.com"]
cors_methods = ["POST", "GET", "OPTIONS"]
cors_headers = ["Content-Type", "Authorization"]
```

## Client Libraries

### Official

- **Rust**: `bitcell-client` (crates.io/crates/bitcell-client)
- **JavaScript**: `@bitcell/web3` (npm install @bitcell/web3)
- **Python**: `bitcell.py` (pip install bitcell)

### Ethereum-Compatible

BitCell implements Ethereum JSON-RPC, so these libraries work:

- **JavaScript**: web3.js, ethers.js
- **Python**: web3.py
- **Go**: go-ethereum (geth)
- **Java**: web3j
- **Rust**: ethers-rs

Example with ethers.js:

```javascript
const { ethers } = require('ethers');

// Connect to BitCell node
const provider = new ethers.JsonRpcProvider('http://localhost:8545');

// Get block
const block = await provider.getBlock('latest');
console.log('Latest block:', block.number);

// Send transaction
const wallet = new ethers.Wallet(privateKey, provider);
const tx = await wallet.sendTransaction({
  to: '0x1234...',
  value: ethers.parseEther('10')
});
await tx.wait();
```

## API Versioning

Current API version: **v1**

Version is specified in URL path for REST API:
```
http://localhost:8080/api/v1/blocks/latest
```

JSON-RPC methods are versioned by prefix:
- `eth_*` - Ethereum-compatible (stable)
- `bitcell_*` - BitCell-specific (stable)
- `experimental_*` - Experimental features (unstable)

## Error Handling

### JSON-RPC Errors

Standard error format:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "param": "address",
      "reason": "Invalid address format"
    }
  }
}
```

Common error codes:

| Code | Message | Description |
|------|---------|-------------|
| -32700 | Parse error | Invalid JSON |
| -32600 | Invalid request | Missing required fields |
| -32601 | Method not found | Unknown method |
| -32602 | Invalid params | Wrong parameters |
| -32603 | Internal error | Server error |
| -32000 | Transaction reverted | Smart contract error |

[Full Error Code Reference →](./error-codes.md)

### HTTP Status Codes

REST API uses standard HTTP codes:

| Code | Meaning |
|------|---------|
| 200 | Success |
| 400 | Bad request |
| 401 | Unauthorized |
| 404 | Not found |
| 429 | Rate limited |
| 500 | Server error |
| 503 | Service unavailable |

## Data Types

### Addresses

20-byte hex string with `0x` prefix:
```
"0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
```

### Hashes

32-byte hex string with `0x` prefix:
```
"0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
```

### Numbers

Hex-encoded with `0x` prefix:
```
"0x2a5f"  // 10847 decimal
```

### Byte Arrays

Hex-encoded with `0x` prefix:
```
"0x48656c6c6f"  // "Hello" in hex
```

### Block Tags

Special identifiers:
- `"latest"` - Most recent block
- `"earliest"` - Genesis block
- `"pending"` - Pending block (mempool)
- `"0x..."` - Specific block number (hex)

## Performance Tips

### Batch Requests

Send multiple requests in one HTTP call:

```json
[
  {"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1},
  {"jsonrpc":"2.0","method":"eth_gasPrice","params":[],"id":2},
  {"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":3}
]
```

### Connection Pooling

Reuse HTTP connections for better performance:

```javascript
const http = require('http');
const agent = new http.Agent({ keepAlive: true, maxSockets: 10 });

const provider = new Web3(new Web3.providers.HttpProvider(
  'http://localhost:8545',
  { agent }
));
```

### WebSocket for Real-time

Use WebSocket for continuous updates instead of polling:

```javascript
const provider = new ethers.WebSocketProvider('ws://localhost:8546');

// Subscribe to new blocks
provider.on('block', (blockNumber) => {
  console.log('New block:', blockNumber);
});
```

## Monitoring & Debugging

### Enable RPC Logging

```toml
[rpc]
log_requests = true
log_responses = false
log_errors = true
```

### View RPC Metrics

```bash
# Get RPC statistics
curl http://localhost:8545 \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"rpc_stats","params":[],"id":1}'

# Response:
# {
#   "total_requests": 12847,
#   "errors": 23,
#   "avg_response_time_ms": 45,
#   "requests_per_minute": 120
# }
```

### Debug Mode

Enable verbose debugging:

```bash
# Start node with RPC debug logging
bitcell-node start --rpc-debug

# Or set in config
[rpc]
debug = true
trace_calls = true
```

## Security Best Practices

1. **Never expose RPC publicly** without authentication
2. **Use HTTPS** in production
3. **Implement rate limiting** to prevent abuse
4. **Whitelist methods** - disable dangerous methods in production
5. **Use API keys** for authentication
6. **Monitor for unusual activity**

Example secure configuration:

```toml
[rpc]
bind_address = "127.0.0.1:8545"  # Localhost only
api_key_required = true
allowed_methods = ["eth_*", "bitcell_getBlock*", "bitcell_getTransaction*"]
denied_methods = ["personal_*", "admin_*", "debug_*"]
rate_limit_per_minute = 100
enable_https = true
tls_cert = "/path/to/cert.pem"
tls_key = "/path/to/key.pem"
```

## Next Steps

- **[JSON-RPC Methods](./json-rpc.md)** - Detailed method documentation
- **[WebSocket Subscriptions](./websocket.md)** - Real-time updates
- **[Error Codes](./error-codes.md)** - Complete error reference
- **[Smart Contract Interaction](../contracts/deployment.md)** - Call contracts via API

## Examples Repository

Find complete examples at:
```
https://github.com/Steake/BitCell/tree/master/examples/api
```

Includes:
- Simple queries
- Transaction sending
- Contract deployment
- Event monitoring
- WebSocket subscriptions
- Batch requests
