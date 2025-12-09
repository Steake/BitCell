# BitCell Block Explorer

## Overview

The BitCell Block Explorer is a comprehensive web-based interface for viewing and exploring blockchain data, including blocks, transactions, accounts, and tournament battle visualizations.

## Features

### 🔍 Search Functionality
- **Universal Search Bar**: Search by:
  - Block height (numeric)
  - Block hash (0x + 16 hex characters)
  - Transaction hash (0x + 64 hex characters)
  - Account address (0x + 40 hex characters)

### ⛓️ Block Explorer
- **Recent Blocks List**: View the most recent blocks with:
  - Block height
  - Block hash
  - Proposer address
  - Timestamp
  - Transaction count
  - Battle count

- **Block Details**: Click on any block to view:
  - Full block header information
  - Previous block hash
  - State root and transaction root
  - List of transactions
  - Link to battle replay visualization

### 💸 Transaction Explorer
- **Transaction Details**: View complete transaction information:
  - Transaction hash
  - Block height
  - From/To addresses (clickable to view account details)
  - Amount transferred
  - Transaction fee
  - Nonce
  - Status (confirmed/pending)
  - Timestamp

### 👤 Account Explorer
- **Account Information**:
  - Balance (in CELL tokens)
  - Transaction count
  - Current nonce
  - Trust score (for miners)

- **Trust Score Visualization**:
  - Visual meter showing trust score (0-100%)
  - Detailed EBSL metrics:
    - Belief, disbelief, uncertainty values
    - Positive and negative evidence counters
    - Total blocks proposed
    - Slashing events count
  - Account status (active/banned)

- **Transaction History**:
  - Recent transactions for the account
  - Sent/Received indication
  - Click to view transaction details

### ⚔️ Tournament Battle Visualization
- Integrated with the main dashboard
- View CA battle replays for any block
- Step-by-step grid visualization
- Energy tracking for both gliders
- Winner determination

## API Endpoints

### Block Endpoints
- `GET /api/blocks` - List recent blocks
- `GET /api/blocks/:height` - Get block details
- `GET /api/blocks/:height/battles` - Get battle visualization data

### Transaction Endpoints
- `GET /api/transactions/:hash` - Get transaction details
- `GET /api/accounts/:address/transactions` - Get transaction history for an account

### Account Endpoints
- `GET /api/accounts/:address` - Get account information
- `GET /api/accounts/:address/trust` - Get trust score details

### Search Endpoint
- `GET /api/search?q=<query>` - Search for blocks, transactions, or accounts

## Usage

### Accessing the Explorer

1. Start the BitCell admin console:
   ```bash
   cargo run --package bitcell-admin
   ```

2. Navigate to the explorer:
   ```
   http://127.0.0.1:8080/explorer
   ```

3. Or access from the dashboard:
   ```
   http://127.0.0.1:8080/
   ```

### Search Examples

**Search by block height:**
```
123
```

**Search by transaction hash:**
```
0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef
```

**Search by address:**
```
0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb8
```

### Viewing Battle Replays

1. Navigate to a block detail
2. Click "View Battle Replay" 
3. Use the playback controls to:
   - Play/pause the animation
   - Seek to specific frames
   - Adjust playback speed

## Design

The block explorer features a cyberpunk-inspired interface with:
- Matrix-style green text on black background
- Neon glow effects
- Animated scanlines
- Responsive grid layouts
- Modal dialogs for detailed views

## Development

### Adding New Explorer Features

1. **API Endpoint**: Add new endpoint in `crates/bitcell-admin/src/api/explorer.rs`
2. **UI Component**: Update `crates/bitcell-admin/src/web/explorer.rs`
3. **Route**: Register route in `crates/bitcell-admin/src/lib.rs`

### Mock Data

The current implementation uses mock data for demonstration. To integrate with real blockchain data:

1. Replace mock responses in `api/explorer.rs` with actual blockchain queries
2. Connect to `StateManager` for account data
3. Connect to `Blockchain` for block/transaction data
4. Connect to EBSL system for trust scores

## Screenshots

See the PR description for screenshots of:
- Explorer main page
- Account detail modal with trust score
- Transaction detail modal
- Battle visualization

## Future Enhancements

- Real-time updates via WebSocket
- Advanced filtering and sorting options
- Export functionality (CSV, JSON)
- Analytics and statistics dashboards
- Network topology visualization
- Mempool viewer
- Contract explorer (when smart contracts are added)

## Related Documentation

- [Admin Console](../README.md)
- [EBSL Trust System](EBSL.md)
- [Battle Visualization](CA_BATTLES.md)
- [API Reference](API.md)
