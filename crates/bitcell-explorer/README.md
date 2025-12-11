# BitCell Block Explorer

A SvelteKit-based block explorer for the BitCell blockchain.

## Features

- Real-time block viewing
- Transaction search and details
- Account balance and transaction history
- Tournament battle visualization
- Trust score display (EBSL)
- Universal search (block height, hash, tx hash, address)

## Development

```bash
# Install dependencies
npm install

# Run development server
npm run dev

# Build for production
npm run build
```

## Configuration

The explorer connects to a BitCell node RPC endpoint. By default, it expects the node to be running on `localhost:9545`.

To change the RPC endpoint, edit `vite.config.js`:

```javascript
server: {
  proxy: {
    '/rpc': {
      target: 'http://your-node:port',
      changeOrigin: true
    }
  }
}
```

## Architecture

- **No SSR**: This is a client-side only application (SPA)
- **Real RPC calls**: All data comes from the BitCell node via JSON-RPC
- **No mock data**: Every piece of information is fetched from the live blockchain

## API Integration

The explorer uses these RPC methods:

- `eth_blockNumber` - Get current block height
- `eth_getBlockByNumber` - Get block details
- `eth_getTransactionByHash` - Get transaction details
- `eth_getBalance` - Get account balance
- `eth_getTransactionCount` - Get account nonce
- `bitcell_getNodeInfo` - Get node information
- `bitcell_getTournamentState` - Get tournament state
- `bitcell_getBattleReplay` - Get battle replay data
- `bitcell_getMinerStats` - Get miner statistics

## Security

- All user input is validated and sanitized
- XSS protection through proper escaping
- ARIA labels for accessibility
- Keyboard navigation support
