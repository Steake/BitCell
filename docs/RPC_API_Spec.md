# BitCell RPC & API Specification (v0.1-alpha)

This document defines external interfaces for interacting with the BitCell network: JSON-RPC, REST and WebSocket APIs. It is designed to align with the BitCell consensus, tournament, and reputation model.

---

## 1. Transport & General Conventions

- **JSON-RPC**: JSON-RPC 2.0 over HTTP(S) at `/rpc`
- **REST**: HTTP(S) with base path `/api/v1`
- **WebSocket**: WS/WSS at `/ws/*`
- **Encoding**:
  - All JSON: UTF-8, `application/json`
  - All amounts: either integer smallest units or decimal strings (implementation MUST specify)
- **Errors**:
  - JSON-RPC: standard `code` / `message` / optional `data`
  - REST: 
    ```json
    {
      "error": {
        "code": "string",
        "message": "string",
        "details": {}
      }
    }
    ```

Nodes SHOULD expose `bitcell_getNodeInfo` (JSON-RPC) to report `api_version`, `protocol_version`, `network_id`, and capabilities.

---

## 2. Coin Lifecycle (Informative)

The lifecycle of a BitCell coin from creation to finality:

```mermaid
graph TD
    subgraph Mining ["Phase 1: Creation (Mining)"]
        M[Miner] -->|Submits Glider| T[Tournament]
        T -->|Battle Winner| B[Block Creation]
        B -->|Coinbase Reward| S[State Manager]
    end

    subgraph Allocation ["Phase 2: Allocation"]
        S -->|Credit Balance| A[Miner Account]
        A -->|Available UTXO/Balance| W[Wallet UI]
    end

    subgraph Spending ["Phase 3: Spending"]
        W -->|Create Tx| TX[Transaction]
        TX -->|Sign| STX[Signed Tx]
        STX -->|Broadcast| MP[Mempool]
        MP -->|Include in Block| NB[New Block]
    end

    subgraph Finality ["Phase 4: Finality"]
        NB -->|Confirmations > 6| F[Finalized State]
        F -->|Update| R[Recipient Account]
    end

    style M fill:#f9f,stroke:#333,stroke-width:2px
    style F fill:#9f9,stroke:#333,stroke-width:2px
````

Finality is probabilistic (Nakamoto-style). Wallet UX SHOULD treat 6+ confirmations as “final” by default.

---

## 3. JSON-RPC API

JSON-RPC 2.0 endpoint: `/rpc`

### 3.1 Standard Namespace (`eth_*` Compatibility)

These methods are available **only if** the EVM compatibility layer is enabled.

#### `eth_blockNumber`

* **Params**: `[]`
* **Result**: `"0x..."` – current block height in hex.

#### `eth_getBalance`

* **Params**:
  `[ "0x<Address>", "latest" | "pending" | "<blockTag>" ]`
* **Result**: `"0x..."` – balance in smallest unit, hex-encoded.

#### `eth_sendRawTransaction`

* **Params**:
  `[ "0x<RLP_ENCODED_SIGNED_TX>" ]`
* **Result**: `"0x<TxHash>"`

#### `eth_getTransactionReceipt`

* **Params**:
  `[ "0x<TxHash>" ]`
* **Result**: Ethereum-style receipt object (status, logs, gasUsed, etc.), adapted to BitCell’s execution model.

---

### 3.2 BitCell Namespace (`bitcell_*`)

BitCell-specific methods for tournaments, reputation, and node info.

#### 3.2.1 Mining & Tournament

##### `bitcell_submitCommitment`

Submit a glider commitment for the current tournament.

* **Params**:

  ```json
  [
    {
      "commitment": "0x<32-byte-hash>",
      "ring_signature": "0x<opaque-sig-bytes>"
    }
  ]
  ```
* **Result**:

  ```json
  {
    "accepted": true,
    "reason": "optional string"
  }
  ```

##### `bitcell_submitReveal`

Reveal glider details corresponding to a previous commitment.

* **Params**:

  ```json
  [
    {
      "pattern": "base64-or-custom-encoding",
      "nonce": "0x<nonce>",
      "commitment": "0x<32-byte-hash>"
    }
  ]
  ```
* **Result**:

  ```json
  {
    "accepted": true,
    "reason": "optional string"
  }
  ```

##### `bitcell_getTournamentState`

Get the current tournament phase and bracket, or for a specific block.

* **Params**:

  * `[]` – for current height
  * or `[ "0x<blockNumberOrHash>" ]`
* **Result**:

  ```json
  {
    "block": "0x<height-or-hash>",
    "phase": "idle" | "commit" | "reveal" | "battles" | "complete",
    "round": 0,
    "global_seed": "0x<seed>",
    "participants": [ "0x<MinerId1>", "0x<MinerId2>" ],
    "matches": [
      {
        "match_id": "string",
        "round": 0,
        "index": 0,
        "participants": ["0x<MinerIdA>", "0x<MinerIdB>"],
        "status": "pending" | "running" | "finished",
        "winner": "0x<MinerIdA>",
        "mii": "string-or-number",
        "ted": "string-or-number"
      }
    ]
  }
  ```

##### `bitcell_getTournamentTranscript`

Full tournament transcript for a given block.

* **Params**:

  ```json
  [ "0x<blockNumberOrHash>" ]
  ```
* **Result**:

  ```json
  {
    "block": "0x<height-or-hash>",
    "matches": [
      {
        "match_id": "string",
        "round": 0,
        "index": 0,
        "participants": ["0x<MinerIdA>", "0x<MinerIdB>"],
        "winner": "0x<MinerIdA>",
        "entropy_seed": "0x<seed>",
        "metrics": {
          "EA": "string-or-number",
          "EB": "string-or-number",
          "MII_A_B": "string-or-number",
          "MII_B_A": "string-or-number",
          "TED_A_B": "string-or-number",
          "TED_B_A": "string-or-number"
        },
        "proof_commitment": "0x<hash-or-id>"
      }
    ]
  }
  ```

##### `bitcell_getBattleReplay`

Replay data for a specific match.

* **Params**:

  ```json
  [ "match_id" ]
  ```
* **Result**:

  ```json
  {
    "match_id": "string",
    "block": "0x<height-or-hash>",
    "grid_size": 128,
    "steps": 256,
    "participants": ["0x<MinerIdA>", "0x<MinerIdB>"],
    "winner": "0x<MinerIdA>",
    "timeline": {
      "encoding": "delta_compressed" | "full_frames",
      "frames": [ /* encoding-specific */ ]
    },
    "metrics": {
      "EA": "string-or-number",
      "EB": "string-or-number",
      "MII_A_B": "string-or-number",
      "MII_B_A": "string-or-number",
      "TED_A_B": "string-or-number",
      "TED_B_A": "string-or-number"
    }
  }
  ```

---

#### 3.2.2 Reputation & Behaviour

##### `bitcell_getReputation`

Return EBSL trust information for a miner.

* **Params**:

  ```json
  [ "0x<MinerId>" ]
  ```
* **Result**:

  ```json
  {
    "miner": "0x<MinerId>",
    "trust": "float-or-fixed",
    "r": "number",   // positive evidence
    "s": "number"    // negative evidence
  }
  ```

##### `bitcell_getMinerStats`

Expose behaviour-layer metrics (Aggression, Volatility, etc.).

* **Params**:

  ```json
  [ "0x<MinerId>" ]
  ```
* **Result**:

  ```json
  {
    "miner": "0x<MinerId>",
    "aggression_index": "float",
    "volatility_index": "float",
    "win_rate": "float",
    "tournaments_played": "number",
    "finals_reached": "number"
  }
  ```

##### `bitcell_getNetworkMetrics`

Network-level metrics.

* **Params**: `[]`
* **Result**:

  ```json
  {
    "mii_usage_rate": "float",
    "avg_rounds": "float",
    "peer_count": "number",
    "height": "0x<height>",
    "version": "string"
  }
  ```

---

#### 3.2.3 Node & Network

##### `bitcell_getPeerCount`

* **Params**: `[]`
* **Result**: `number`

##### `bitcell_getNodeInfo`

* **Params**: `[]`
* **Result**:

  ```json
  {
    "node_id": "string",
    "version": "string",
    "protocol_version": "string",
    "network_id": "string",
    "api_version": "string",
    "capabilities": [
      "zkvm",
      "evm",
      "proxy_eth",
      "proxy_btc"
    ]
  }
  ```

---

## 4. REST API (Wallet & Admin)

Base path: `/api/v1`

Authentication is implementation-defined, but recommended:

* Local-only by default, or
* `Authorization: Bearer <token>` for remote admin.

### 4.1 Wallet Endpoints

#### `GET /api/v1/wallet/balance/:address`

* **Response**:

  ```json
  {
    "address": "0x...",
    "balance": "string-or-number",
    "confirmed_balance": "string-or-number",
    "unconfirmed_balance": "string-or-number"
  }
  ```

#### `GET /api/v1/wallet/transactions/:address`

* **Query**:

  * `?limit=<int>`
  * `?offset=<int>`
* **Response**:

  ```json
  {
    "address": "0x...",
    "transactions": [
      {
        "tx_hash": "0x...",
        "from": "0x...",
        "to": "0x...",
        "amount": "string-or-number",
        "fee": "string-or-number",
        "timestamp": "iso8601",
        "confirmations": "number",
        "status": "pending" | "confirmed" | "failed"
      }
    ]
  }
  ```

#### `POST /api/v1/wallet/send`

Create and broadcast a transaction.

* **Body**:

  ```json
  {
    "from": "0x...",      // optional if single-wallet context
    "to": "0x...",
    "amount": "string-or-number",
    "fee": "string-or-number",
    "memo": "optional string"
  }
  ```
* **Response**:

  ```json
  {
    "tx_hash": "0x...",
    "status": "pending"
  }
  ```

---

### 4.2 Mining Endpoints

#### `POST /api/v1/mining/glider`

Upload or construct a glider pattern for use in tournaments.

* **Body**:

  ```json
  {
    "name": "optional label",
    "pattern": "base64-or-custom-encoding",
    "metadata": {
      "notes": "optional",
      "created_at": "iso8601"
    }
  }
  ```
* **Response**:

  ```json
  {
    "id": "glider_id",
    "status": "stored"
  }
  ```

#### `GET /api/v1/mining/status`

* **Response**:

  ```json
  {
    "phase": "idle" | "commit" | "reveal" | "battles",
    "height": "0x<height>",
    "current_glider_id": "optional string",
    "auto_miner": true
  }
  ```

#### `POST /api/v1/mining/auto-miner`

Toggle automatic commit/reveal of configured patterns.

* **Body**:

  ```json
  {
    "enabled": true
  }
  ```
* **Response**:

  ```json
  {
    "enabled": true
  }
  ```

---

### 4.3 External Chain Connector (Proxy)

These endpoints forward JSON-RPC requests to configured external nodes.

#### `POST /api/v1/proxy/eth`

* **Body**: any valid Ethereum JSON-RPC request object.
* **Response**: upstream Ethereum JSON-RPC response.

#### `POST /api/v1/proxy/btc`

* **Body**: any valid Bitcoin JSON-RPC request object.
* **Response**: upstream Bitcoin JSON-RPC response.

---

## 5. WebSocket API (Real-time)

Base: `/ws`

All messages are JSON objects with:

```json
{
  "event": "string",
  "data": { }
}
```

### 5.1 Battles Stream

Endpoint: `/ws/battles`

Events:

#### `battle_start`

* **Payload**:

  ```json
  {
    "match_id": "string",
    "block": "0x<height-or-hash>",
    "participants": ["0x<MinerIdA>", "0x<MinerIdB>"],
    "grid_size": 128,
    "steps": 256
  }
  ```

#### `step_update`

* **Payload**:

  ```json
  {
    "match_id": "string",
    "step": 42,
    "delta_encoding": "base64-or-custom",
    "mii_partial": "optional string-or-number",
    "ted_partial": "optional string-or-number"
  }
  ```

#### `battle_end`

* **Payload**:

  ```json
  {
    "match_id": "string",
    "winner": "0x<MinerIdA>",
    "EA": "string-or-number",
    "EB": "string-or-number",
    "MII_A_B": "string-or-number",
    "MII_B_A": "string-or-number",
    "TED_A_B": "string-or-number",
    "TED_B_A": "string-or-number"
  }
  ```

---

### 5.2 Blocks Stream

Endpoint: `/ws/blocks`

Event: `block`

* **Payload**:

  ```json
  {
    "height": "0x<height>",
    "hash": "0x<hash>",
    "parent_hash": "0x<hash>",
    "timestamp": "number",
    "tx_count": "number",
    "tournament_summary": {
      "winner": "0x<MinerId>",
      "match_count": "number"
    }
  }
  ```

---

## 6. Wallet GUI Specification (Summary)

The Wallet GUI consumes the REST and WebSocket APIs above and provides:

1. **Asset Management**

   * Retrieve balances and transaction histories (BitCell, ETH, BTC).
   * Initiate new transactions via `/api/v1/wallet/send`.
2. **Mining Operations**

   * Glider editor using `/api/v1/mining/glider`.
   * Status via `/api/v1/mining/status`.
   * Auto-miner toggle via `/api/v1/mining/auto-miner`.
3. **Battle Theater**

   * Subscribe to `/ws/battles` for live CA visualization.
   * Render 128x128 grid and update with `step_update` events.
   * Display MII/TED metrics in a HUD.
4. **Multi-Chain Configuration**

   * Configure ETH/BTC RPC targets used by `/api/v1/proxy/eth` and `/api/v1/proxy/btc`.
   * Display a unified balance view aggregating external chains.

Details of GUI layout, components, and styling are implementation-specific.

---

## 7. Implementation Roadmap

1. **Phase 1 – Core JSON-RPC**

   * Implement `bitcell_*` read methods (tournament, reputation, metrics, node info).
   * Implement write-paths for commitments, reveals, and basic tx submission.
   * Optional: `eth_*` compatibility if/when EVM layer exists.

2. **Phase 2 – REST Wallet/Admin**

   * Wallet endpoints for balances, history, and send.
   * Mining endpoints for glider management and status.

3. **Phase 3 – WebSocket Streams**

   * Implement `/ws/battles` with `battle_start`, `step_update`, `battle_end`.
   * Implement `/ws/blocks` with basic block summaries.

4. **Phase 4 – Wallet GUI**

   * Implement Glider Editor, Battle Theater, and unified asset dashboard.
   * Integrate with both REST and WebSocket APIs.

5. **Phase 5 – External Chain Proxy**

   * Implement ETH/BTC proxy endpoints and configuration.
   * Optional: rate limiting, logging, and access control.

