# WebSocket API Documentation

This document describes the WebSocket API for real-time event subscriptions in BitCell.

## Overview

BitCell provides a WebSocket API compatible with the Ethereum JSON-RPC WebSocket specification. The API supports real-time subscriptions to blockchain events with filtering capabilities.

## Connection

Connect to the WebSocket endpoint:

```
ws://<host>:<port>/ws
```

Default port: 8545

## Supported Subscriptions

### 1. New Block Headers (`newHeads`)

Subscribe to new block headers as they are added to the blockchain.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "eth_subscribe",
  "params": ["newHeads"]
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": "0x1"
}
```

**Notification:**
```json
{
  "jsonrpc": "2.0",
  "method": "eth_subscription",
  "params": {
    "subscription": "0x1",
    "result": {
      "number": "0x64",
      "hash": "0x...",
      "parentHash": "0x...",
      "timestamp": "0x5f5e100",
      "miner": "0x...",
      "transactionsRoot": "0x...",
      "stateRoot": "0x..."
    }
  }
}
```

### 2. Pending Transactions (`pendingTransactions`)

Subscribe to pending transactions as they enter the transaction pool.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "eth_subscribe",
  "params": ["pendingTransactions"]
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": "0x2"
}
```

**Notification:**
```json
{
  "jsonrpc": "2.0",
  "method": "eth_subscription",
  "params": {
    "subscription": "0x2",
    "result": "0x..."
  }
}
```

The result is the transaction hash.

### 3. Event Logs (`logs`)

Subscribe to event logs with optional filtering.

**Note:** The current implementation provides basic log extraction from transaction data. Full transaction receipt and event log support will be enhanced in future releases with proper EVM integration.

**Request without filter:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "eth_subscribe",
  "params": ["logs"]
}
```

**Request with address filter:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "eth_subscribe",
  "params": [
    "logs",
    {
      "address": ["0x1234...", "0x5678..."]
    }
  ]
}
```

**Request with topics filter:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "eth_subscribe",
  "params": [
    "logs",
    {
      "address": ["0x1234..."],
      "topics": [
        ["0xabc...", "0xdef..."],
        null,
        ["0x123..."]
      ]
    }
  ]
}
```

**Filter Parameters:**
- `address`: (optional) Array of contract addresses. Only logs from these addresses will be emitted.
- `topics`: (optional) Array of topic filters. Each element can be:
  - `null` - matches any topic
  - A single topic hash - matches that specific topic
  - An array of topic hashes - matches any of the topics (OR condition)

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": "0x3"
}
```

**Notification:**
```json
{
  "jsonrpc": "2.0",
  "method": "eth_subscription",
  "params": {
    "subscription": "0x3",
    "result": {
      "address": "0x...",
      "topics": ["0x..."],
      "data": "0x...",
      "blockNumber": "0x64",
      "transactionHash": "0x...",
      "transactionIndex": "0x0",
      "blockHash": "0x...",
      "logIndex": "0x0"
    }
  }
}
```

## Unsubscribe

To unsubscribe from a subscription, use the `eth_unsubscribe` method:

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "eth_unsubscribe",
  "params": ["0x1"]
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": true
}
```

## Limits and Rate Limiting

### Connection Limits

- **Maximum subscriptions per client**: 100
- **Maximum messages per second per client**: 100

When limits are exceeded, you'll receive an error response:

```json
{
  "jsonrpc": "2.0",
  "id": null,
  "error": {
    "code": -32005,
    "message": "Rate limit exceeded"
  }
}
```

or

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "error": {
    "code": -32005,
    "message": "Exceeded max subscriptions (100)"
  }
}
```

### Heartbeat

The server sends periodic ping frames to keep the connection alive. Clients should respond with pong frames (handled automatically by most WebSocket libraries).

## Error Codes

| Code | Message | Description |
|------|---------|-------------|
| -32700 | Parse error | Invalid JSON |
| -32600 | Invalid Request | Invalid JSON-RPC request |
| -32601 | Method not found | Method does not exist |
| -32602 | Invalid params | Invalid method parameters |
| -32005 | Limit exceeded | Rate limit or subscription limit exceeded |

## Example Usage

### JavaScript (Node.js)

```javascript
const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:8545/ws');

ws.on('open', function open() {
  // Subscribe to new blocks
  ws.send(JSON.stringify({
    jsonrpc: '2.0',
    id: 1,
    method: 'eth_subscribe',
    params: ['newHeads']
  }));
});

ws.on('message', function incoming(data) {
  const response = JSON.parse(data);
  console.log('Received:', response);
  
  if (response.method === 'eth_subscription') {
    console.log('New block:', response.params.result);
  }
});

ws.on('error', console.error);
```

### Python

```python
import asyncio
import json
import websockets

async def subscribe_to_blocks():
    uri = "ws://localhost:8545/ws"
    async with websockets.connect(uri) as websocket:
        # Subscribe to new blocks
        await websocket.send(json.dumps({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_subscribe",
            "params": ["newHeads"]
        }))
        
        # Receive subscription ID
        response = await websocket.recv()
        print(f"Subscription response: {response}")
        
        # Listen for notifications
        async for message in websocket:
            data = json.loads(message)
            if data.get('method') == 'eth_subscription':
                print(f"New block: {data['params']['result']}")

asyncio.run(subscribe_to_blocks())
```

### Rust

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures::{StreamExt, SinkExt};
use serde_json::json;

#[tokio::main]
async fn main() {
    let url = "ws://localhost:8545/ws";
    let (ws_stream, _) = connect_async(url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();
    
    // Subscribe to new blocks
    let subscribe_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_subscribe",
        "params": ["newHeads"]
    });
    
    write.send(Message::Text(subscribe_req.to_string())).await.unwrap();
    
    // Read responses
    while let Some(Ok(Message::Text(text))) = read.next().await {
        let response: serde_json::Value = serde_json::from_str(&text).unwrap();
        println!("Received: {}", response);
    }
}
```

## Legacy Endpoints

For backward compatibility, the following legacy endpoints are also available:

- `/ws/battles` - Subscribe to battle/tournament phase changes
- `/ws/blocks` - Simple block height updates

These endpoints use a simpler non-JSON-RPC format and are intended for specific use cases.

## Best Practices

1. **Handle reconnections**: Implement exponential backoff when reconnecting
2. **Resubscribe after reconnect**: Subscriptions are not persisted across connections
3. **Process notifications asynchronously**: Don't block the WebSocket receive loop
4. **Implement timeouts**: Set appropriate timeouts for WebSocket operations
5. **Handle errors gracefully**: Check for error responses and handle them appropriately
6. **Monitor resource usage**: Be aware of subscription and rate limits

## Security Considerations

1. **Authentication**: Currently not implemented. In production, implement proper authentication.
2. **Rate limiting**: The server implements rate limiting to prevent abuse.
3. **Resource limits**: Maximum subscriptions per client prevents resource exhaustion.
4. **Input validation**: All parameters are validated before processing.

## Future Enhancements

Planned improvements for future releases:

- Authentication and authorization
- Subscription persistence across reconnections
- More granular filtering options
- Subscription statistics and monitoring
- Custom BitCell-specific subscriptions (tournaments, battles, etc.)
