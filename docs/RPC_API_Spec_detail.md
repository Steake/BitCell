# BitCell RPC & API Specification

## 1. Overview

This document defines the comprehensive interface specifications for interacting with the BitCell network, a novel blockchain platform leveraging Cellular Automata for its consensus mechanism. It outlines the various communication protocols and endpoints designed to facilitate robust node interaction, provide rich user experiences through Wallet GUI and Admin Console, and ensure seamless multi-chain interoperability with established networks like Ethereum (ETH) and Bitcoin (BTC). The goal is to provide a clear, detailed blueprint for developers building on or integrating with the BitCell ecosystem.

## 2. Coin Lifecycle

The following diagram illustrates the complete lifecycle of a BitCell coin, from its initial creation through the unique mining process to its eventual finality and transfer between accounts. Each phase highlights critical interactions within the BitCell network's consensus and state management systems.

```mermaid
graph TD
    subgraph Mining ["Phase 1: Creation (Mining)"]
        M[Miner] -->|Submits Glider Commitment| T[Tournament Manager]
        T -->|Collects Reveals & Initiates Battle| B[Battle Engine]
        B -->|Determines Winner & Generates Block| BC[Block Creation]
        BC -->|Awards Coinbase Reward| S[State Manager]
    end

    subgraph Allocation ["Phase 2: Allocation & Availability"]
        S -->|Credits Balance to Miner's Account| A[Miner Account]
        A -->|Reflects Available UTXO/Balance| W[Wallet UI/API]
    end

    subgraph Spending ["Phase 3: Transaction & Propagation"]
        W -->|User Initiates Transaction| TX[Transaction Creation]
        TX -->|Wallet Signs Transaction| STX[Signed Transaction]
        STX -->|Broadcasts to Network Peers| MP[Mempool]
        MP -->|Node Includes in New Block| NB[New Block Proposal]
    end

    subgraph Finality ["Phase 4: Confirmation & Finality"]
        NB -->|Propagates & Receives Confirmations (&gt; 6)| F[Finalized State]
        F -->|Updates Recipient's Account Balance| R[Recipient Account]
    end
        TX -->|Wallet Signs Transaction| STX[Signed Transaction]
        STX -->|Broadcasts to Network Peers| MP[Mempool]
        MP -->|Node Includes in New Block| NB[New Block Proposal]
    end

    subgraph Finality ["Phase 4: Confirmation & Finality"]
        NB -->|Propagates & Receives Confirmations (> 6)| F[Finalized State]
        F -->|Updates Recipient's Account Balance| R[Recipient Account]
    end

    style M fill:#f9f,stroke:#333,stroke-width:2px,color:#000
    style T fill:#add8e6,stroke:#333,stroke-width:2px,color:#000
    style B fill:#ffcccb,stroke:#333,stroke-width:2px,color:#000
    style BC fill:#d3d3d3,stroke:#333,stroke-width:2px,color:#000
    style S fill:#90ee90,stroke:#333,stroke-width:2px,color:#000
    style A fill:#ffe4b5,stroke:#333,stroke-width:2px,color:#000
    style W fill:#e0ffff,stroke:#333,stroke-width:2px,color:#000
    style TX fill:#f0e68c,stroke:#333,stroke-width:2px,color:#000
    style STX fill:#dda0dd,stroke:#333,stroke-width:2px,color:#000
    style MP fill:#b0c4de,stroke:#333,stroke-width:2px,color:#000
    style NB fill:#c0c0c0,stroke:#333,stroke-width:2px,color:#000
    style F fill:#9f9,stroke:#333,stroke-width:2px,color:#000
    style R fill:#afeeee,stroke:#333,stroke-width:2px,color:#000
```
**Phase 1: Creation (Mining)**: Miners participate in a unique tournament-based mining process. They first submit a cryptographic commitment to a "glider" (a Cellular Automata pattern). After a commitment phase, they reveal their glider. These gliders then battle within the Battle Engine, and the winner earns the right to propose a new block and receives a coinbase reward, which is then recorded by the State Manager.

**Phase 2: Allocation & Availability**: The State Manager credits the newly minted coins to the winning miner's account. These coins become available as UTXOs or an updated balance, accessible via the Wallet UI or directly through API calls.

**Phase 3: Transaction & Propagation**: Users initiate transactions through their wallets. The wallet signs the transaction and broadcasts it to the network's mempool. Nodes then select transactions from the mempool to include in new blocks they propose.

**Phase 4: Confirmation & Finality**: Once a new block containing the transaction is mined and propagated, it begins accumulating confirmations from subsequent blocks. After a predefined number of confirmations (e.g., 6 blocks), the transaction is considered final, and the recipient's account balance is updated irrevocably.

## 3. JSON-RPC API

The BitCell node exposes a JSON-RPC 2.0 interface, providing a standardized and extensible way for applications and services to interact directly with the core blockchain logic. This API is designed to be compatible with existing tooling where appropriate, while also offering specific methods tailored to BitCell's unique features.

### 3.1 Standard Namespace (`eth_` compatibility)

To ensure broad compatibility with existing blockchain tooling, such as Metamask, Ethers.js, and Web3.js, BitCell provides an EVM-compatible layer for basic account and transaction operations. This allows for a smoother onboarding experience for developers familiar with Ethereum.

-   `eth_blockNumber`:
    -   **Description**: Returns the number of the most recent block.
    -   **Parameters**: None.
    -   **Returns**: `QUANTITY` - The current block number encoded as hexadecimal.
-   `eth_getBalance(address: Address, blockParameter: BlockParameter)`:
    -   **Description**: Returns the balance of the account at the specified address.
    -   **Parameters**:
        -   `address`: `ADDRESS` - The address to check for balance.
        -   `blockParameter`: `QUANTITY|TAG` - An integer block number, or one of the string tags "latest", "earliest", "pending".
    -   **Returns**: `QUANTITY` - The balance of the account in Wei, encoded as hexadecimal.
-   `eth_sendRawTransaction(signedTransactionData: Bytes)`:
    -   **Description**: Submits a signed transaction for inclusion in the blockchain. This method expects a fully RLP-encoded and signed transaction.
    -   **Parameters**:
        -   `signedTransactionData`: `BYTES` - The RLP-encoded signed transaction data.
    -   **Returns**: `HASH` - The 32-byte transaction hash.
-   `eth_getTransactionReceipt(transactionHash: Hash)`:
    -   **Description**: Returns the receipt of a transaction by transaction hash. Note that the receipt is only available for mined transactions.
    -   **Parameters**:
        -   `transactionHash`: `HASH` - The hash of the transaction.
    -   **Returns**: `Object` - A transaction receipt object, or `null` if the transaction is not found or not yet mined. Includes fields like `blockHash`, `blockNumber`, `transactionHash`, `from`, `to`, `gasUsed`, `logs`, etc.

### 3.2 BitCell Namespace (`bitcell_`)

This namespace provides access to BitCell's core functionalities, including its Cellular Automata consensus, tournament system, and network-specific operations. These methods are unique to the BitCell protocol.

#### Mining & Tournament

-   `bitcell_submitCommitment(commitment: Hash)`:
    -   **Description**: Submits a cryptographic hash of a miner's chosen glider pattern. This commitment locks in the miner's glider for the upcoming tournament phase without revealing its details.
    -   **Parameters**:
        -   `commitment`: `HASH` - A 32-byte hash of the glider pattern and a nonce.
    -   **Returns**: `BOOLEAN` - `true` if the commitment was successfully accepted, `false` otherwise (e.g., duplicate commitment, wrong phase).
-   `bitcell_submitReveal(reveal: GliderReveal)`:
    -   **Description**: Reveals the actual glider pattern and nonce corresponding to a previously submitted commitment. This must be done within the designated reveal phase.
    -   **Parameters**:
        -   `reveal`: `Object` - An object containing:
            -   `pattern`: `STRING` - A serialized representation of the glider pattern (e.g., RLE string).
            -   `nonce`: `STRING` - The nonce used in conjunction with the pattern to generate the commitment hash.
    -   **Returns**: `BOOLEAN` - `true` if the reveal was successful and matched a valid commitment, `false` otherwise.
-   `bitcell_getTournamentState()`:
    -   **Description**: Retrieves the current state of the mining tournament, including the active phase, participating gliders, and battle schedule.
    -   **Parameters**: None.
    -   **Returns**: `Object` - An object detailing:
        -   `currentPhase`: `STRING` - "Commitment", "Reveal", "Battle", "Cooldown".
        -   `blockNumber`: `QUANTITY` - The current block number.
        -   `nextPhaseBlock`: `QUANTITY` - The block number when the next phase begins.
        -   `participants`: `ARRAY` - List of committed/revealed gliders (hashes or IDs).
        -   `bracket`: `ARRAY` - Current tournament bracket structure (if in Battle phase).
-   `bitcell_getBattleReplay(match_id: String)`:
    -   **Description**: Fetches the complete step-by-step history of a specific glider battle, allowing for detailed analysis and visualization.
    -   **Parameters**:
        -   `match_id`: `STRING` - The unique identifier for the battle.
    -   **Returns**: `Object` - An object containing:
        -   `battleId`: `STRING` - The ID of the battle.
        -   `gliderA`: `Object` - Details of the first glider (pattern, owner).
        -   `gliderB`: `Object` - Details of the second glider.
        -   `initialState`: `ARRAY` - The initial 128x128 grid state.
        -   `steps`: `ARRAY` - An array of grid state changes or full grid states for each step of the battle.
        -   `winner`: `STRING` - The ID of the winning glider.

#### Network & Node

-   `bitcell_getPeerCount()`:
    -   **Description**: Returns the number of currently connected peers to the BitCell node.
    -   **Parameters**: None.
    -   **Returns**: `QUANTITY` - The number of connected peers, encoded as hexadecimal.
-   `bitcell_getNodeInfo()`:
    -   **Description**: Provides detailed information about the BitCell node, including its version, unique ID, and supported capabilities.
    -   **Parameters**: None.
    -   **Returns**: `Object` - An object containing:
        -   `version`: `STRING` - Software version of the node.
        -   `nodeId`: `STRING` - Unique identifier of the node.
        -   `networkId`: `QUANTITY` - The network ID the node is connected to.
        -   `capabilities`: `ARRAY` - List of supported protocols/features (e.g., "eth/64", "bitcell/1").
        -   `syncing`: `BOOLEAN` - `true` if the node is currently syncing, `false` otherwise.

## 4. REST API (Wallet & Admin)

A lightweight and user-friendly RESTful API is provided for seamless integration with the BitCell Wallet GUI and Admin Dashboard. This API prioritizes ease of use and common web development patterns.

### 4.1 Wallet Endpoints

These endpoints facilitate core wallet functionalities, allowing users to manage their BitCell assets, view transaction history, and initiate transfers.

-   `GET /api/v1/wallet/balance/:address`:
    -   **Description**: Retrieves the current BitCell balance for a given address.
    -   **Parameters**:
        -   `address`: `STRING` - The BitCell address (e.g., `bc1q...`).
    -   **Response**:
        ```json
        {
          "address": "bc1q...",
          "balance": "1234567890000000000" // Balance in smallest unit (e.g., satoshis/wei equivalent)
        }
        ```
-   `GET /api/v1/wallet/transactions/:address`:
    -   **Description**: Fetches a paginated list of transaction history for a specific address.
    -   **Parameters**:
        -   `address`: `STRING` - The BitCell address.
        -   `limit`: `INTEGER` (optional, default: 10) - Maximum number of transactions to return.
        -   `offset`: `INTEGER` (optional, default: 0) - Number of transactions to skip.
    -   **Response**:
        ```json
        {
          "address": "bc1q...",
          "transactions": [
            {
              "txHash": "0x...",
              "from": "bc1q...",
              "to": "bc1q...",
              "amount": "100000000000000000",
              "fee": "100000000000000",
              "timestamp": 1678886400,
              "blockNumber": 12345,
              "status": "confirmed" // "pending", "confirmed", "failed"
            }
          ],
          "total": 150
        }
        ```
-   `POST /api/v1/wallet/send`:
    -   **Description**: Creates, signs, and broadcasts a new BitCell transaction. The wallet service handles key management and signing.
    -   **Request Body**:
        ```json
        {
          "from": "bc1q...", // Optional, if wallet manages multiple accounts
          "to": "bc1q...",
          "amount": "100000000000000000", // Amount in smallest unit
          "fee": "100000000000000",      // Fee in smallest unit
          "password": "your_wallet_password" // Required for signing
        }
        ```
    -   **Response**:
        ```json
        {
          "success": true,
          "txHash": "0xabcdef1234567890..."
        }
        ```

### 4.2 Mining Endpoints

These endpoints provide an interface for miners to interact with the BitCell mining process, including submitting gliders and checking mining status.

-   `POST /api/v1/mining/glider`:
    -   **Description**: Uploads or constructs a glider pattern for mining. This can be a new pattern or selecting a saved one.
    -   **Request Body**:
        ```json
        {
          "name": "MyAwesomeGlider",
          "pattern": "B3/S23/...", // RLE string or other serialized format
          "autoCommit": true      // Optional: automatically commit this glider in the next phase
        }
        ```
    -   **Response**:
        ```json
        {
          "success": true,
          "gliderId": "glider_abc123",
          "message": "Glider saved and ready for commitment."
        }
        ```
-   `GET /api/v1/mining/status`:
    -   **Description**: Provides the current mining status for the connected node or wallet, indicating the active tournament phase and any pending actions.
    -   **Response**:
        ```json
        {
          "currentPhase": "Commitment",
          "nextPhaseStartsInBlocks": 15,
          "committedGliderId": "glider_abc123", // If a glider has been committed
          "revealedGliderId": null,            // If a glider has been revealed
          "isMining": true,                    // True if auto-miner is active
          "lastBattleResult": {
            "battleId": "battle_xyz",
            "winner": "glider_def456"
          }
        }
        
