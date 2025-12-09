# BitCell Block Explorer

A SvelteKit-based block explorer for the BitCell blockchain that connects to live node RPC endpoints.

## Overview

The BitCell Block Explorer is a modern, client-side web application built with SvelteKit that provides real-time blockchain data visualization. Unlike the previous implementation, this explorer:

- **No mock data**: All information comes directly from BitCell node RPC endpoints
- **Real-time updates**: Connects to live blockchain nodes
- **Client-side only (SPA)**: No server-side rendering required
- **Modern framework**: Built with Svelte for optimal performance

## Features

### 🔍 Search Functionality
- Universal search bar supporting:
  - Block height (numeric)
  - Transaction hash (0x + 64 hex characters)
  - Account address (0x + 40 hex characters)

### ⛓️ Block Explorer
- Recent blocks list with real-time updates
- Block details with:
  - Block header information
  - Transaction list
  - Proposer information
  - Timestamp

### 💸 Transaction Explorer
- Transaction details including:
  - From/To addresses
  - Amount transferred
  - Transaction fee
  - Block confirmation
  - Status

### 👤 Account Explorer
- Account information:
  - Balance (in CELL tokens)
  - Transaction count (nonce)
  - Transaction history

### 🛡️ Trust Score Display
- EBSL (Evidence-Based Subjective Logic) metrics
- Miner statistics
- Battle history

## Architecture

### Frontend (SvelteKit)
Location: `crates/bitcell-explorer/`

- **Framework**: SvelteKit (static adapter)
- **Build**: Vite
- **Type checking**: TypeScript/JSDoc
- **Styling**: CSS with custom cyberpunk theme

### RPC Integration
The explorer communicates with BitCell nodes via JSON-RPC 2.0:

```javascript
// Example: Get current block number
const result = await rpcCall('eth_blockNumber');
```

### RPC Methods Used
- `eth_blockNumber` - Get current block height
- `eth_getBlockByNumber` - Get block details
- `eth_getTransactionByHash` - Get transaction details
- `eth_getBalance` - Get account balance
- `eth_getTransactionCount` - Get account nonce
- `bitcell_getNodeInfo` - Get node information
- `bitcell_getTournamentState` - Get tournament state
- `bitcell_getBattleReplay` - Get battle replay data
- `bitcell_getMinerStats` - Get miner statistics

## Development

### Prerequisites
- Node.js 18+ and npm
- A running BitCell node with RPC enabled

### Setup

```bash
cd crates/bitcell-explorer

# Install dependencies
npm install

# Start development server
npm run dev
```

The explorer will be available at `http://localhost:5173`

### Configuration

Edit `vite.config.js` to change the RPC endpoint:

```javascript
server: {
  proxy: {
    '/rpc': {
      target: 'http://localhost:9545',  // Your node RPC port
      changeOrigin: true
    }
  }
}
```

### Building for Production

```bash
npm run build
```

The built files will be in the `build/` directory and can be served by any static web server.

## Deployment

### Serve with Node
The explorer is designed to be integrated with the admin console. The built static files should be served from the admin console's web server.

### Standalone Deployment
You can also deploy the explorer as a standalone application:

1. Build the application: `npm run build`
2. Serve the `build/` directory with any web server (nginx, Apache, etc.)
3. Configure the web server to proxy `/rpc` requests to your BitCell node

Example nginx configuration:

```nginx
server {
    listen 80;
    server_name explorer.bitcell.dev;
    
    root /path/to/bitcell-explorer/build;
    index index.html;
    
    location / {
        try_files $uri $uri/ /index.html;
    }
    
    location /rpc {
        proxy_pass http://localhost:9545/rpc;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}
```

## Security

### Input Validation
- All user input is validated using regex patterns
- Hex strings are verified for valid characters
- Length checks prevent buffer overflows

### XSS Protection
- All dynamic content is properly escaped
- No use of `innerHTML` with user data
- Svelte's built-in escaping protects against XSS

### Accessibility
- ARIA labels on all interactive elements
- Keyboard navigation support
- Focus indicators for all focusable elements
- Semantic HTML structure

## Testing

```bash
# Run type checking
npm run check

# Run development server with hot reload
npm run dev
```

## Troubleshooting

### Connection Issues
If the explorer cannot connect to the node:

1. Verify the node is running: `curl http://localhost:9545/rpc`
2. Check the proxy configuration in `vite.config.js`
3. Ensure CORS is enabled on the node if accessing directly

### Build Issues
If the build fails:

1. Delete `node_modules` and reinstall: `rm -rf node_modules && npm install`
2. Clear SvelteKit cache: `rm -rf .svelte-kit`
3. Check Node.js version: `node --version` (should be 18+)

## Migration from Old Explorer

The previous explorer (inline HTML in bitcell-admin) has been removed. Key differences:

### Old Explorer (Removed)
- ❌ Mock data only
- ❌ Inline HTML/JS in Rust files
- ❌ No real blockchain connectivity
- ❌ Security issues (XSS vulnerabilities)

### New Explorer
- ✅ Real RPC connections
- ✅ Separate SvelteKit application
- ✅ Live blockchain data
- ✅ Proper input validation and XSS protection
- ✅ Better accessibility
- ✅ Modern framework with hot reload

## Future Enhancements

- WebSocket support for real-time updates
- Battle visualization with canvas rendering
- Advanced filtering and sorting
- Export functionality (CSV, JSON)
- Network topology visualization
- Mempool viewer
- Multi-language support

## Related Documentation

- [Admin Console](../crates/bitcell-admin/README.md)
- [RPC API](./RPC_API.md)
- [EBSL Trust System](./EBSL.md)
- [Battle Visualization](./CA_BATTLES.md)
