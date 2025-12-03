use axum::{
    extract::{State, Json, Path, Query},
    routing::{get, post},
    Router,
    response::{IntoResponse, Response},
    http::StatusCode,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use crate::{Blockchain, NetworkManager, TransactionPool, NodeConfig};
use crate::tournament::TournamentManager;

/// Empty bloom filter (256 bytes of zeros) for blocks without logs
static EMPTY_BLOOM_FILTER: [u8; 256] = [0u8; 256];

/// RPC Server State
#[derive(Clone)]
pub struct RpcState {
    pub blockchain: Blockchain,
    pub network: NetworkManager,
    pub tx_pool: TransactionPool,
    pub tournament_manager: Option<Arc<TournamentManager>>,
    pub config: NodeConfig,
    pub node_type: String, // "validator", "miner", "full"
    pub node_id: String,   // Unique node identifier (public key hex)
}

/// Start the RPC server
pub async fn run_server(state: RpcState, port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = Router::new()
        .route("/rpc", post(handle_json_rpc))
        .nest("/api/v1", api_router())
        .nest("/ws", crate::ws::ws_router())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("RPC server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// JSON-RPC Request
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

/// JSON-RPC Response
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    id: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

/// Handle JSON-RPC requests
async fn handle_json_rpc(
    State(state): State<RpcState>,
    Json(req): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    // Validate JSON-RPC version
    if req.jsonrpc != "2.0" {
        return Json(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: -32600,
                message: "Invalid Request".to_string(),
                data: None,
            }),
            id: req.id,
        });
    }

    let result = match req.method.as_str() {
        // Standard Namespace
        "eth_blockNumber" => eth_block_number(&state).await,
        "eth_getBlockByNumber" => eth_get_block_by_number(&state, req.params).await,
        "eth_getTransactionByHash" => eth_get_transaction_by_hash(&state, req.params).await,
        "eth_getBalance" => eth_get_balance(&state, req.params).await,
        "eth_sendRawTransaction" => eth_send_raw_transaction(&state, req.params).await,
        "eth_getTransactionCount" => eth_get_transaction_count(&state, req.params).await,
        "eth_gasPrice" => eth_gas_price(&state).await,
        
        // BitCell Namespace
        "bitcell_getNodeInfo" => bitcell_get_node_info(&state).await,
        "bitcell_getPeerCount" => bitcell_get_peer_count(&state).await,
        "bitcell_getNetworkMetrics" => bitcell_get_network_metrics(&state).await,
        "bitcell_getTournamentState" => bitcell_get_tournament_state(&state).await,
        "bitcell_submitCommitment" => bitcell_submit_commitment(&state, req.params).await,
        "bitcell_submitReveal" => bitcell_submit_reveal(&state, req.params).await,
        "bitcell_getBattleReplay" => bitcell_get_battle_replay(&state, req.params).await,
        "bitcell_getReputation" => bitcell_get_reputation(&state, req.params).await,
        "bitcell_getMinerStats" => bitcell_get_miner_stats(&state, req.params).await,
        "bitcell_getPendingBlockInfo" => eth_pending_block_number(&state).await,
        
        // Default
        _ => Err(JsonRpcError {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        }),
    };

    match result {
        Ok(val) => Json(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(val),
            error: None,
            id: req.id,
        }),
        Err(err) => Json(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(err),
            id: req.id,
        }),
    }
}

// --- JSON-RPC Methods ---

/// Get current block number
/// 
/// Returns the highest confirmed block number.
/// If pending transactions exist, a "pending" query will return height + 1.
async fn eth_block_number(state: &RpcState) -> Result<Value, JsonRpcError> {
    let height = state.blockchain.height();
    Ok(json!(format!("0x{:x}", height)))
}

/// Get pending block number (height + 1 if pending transactions exist)
async fn eth_pending_block_number(state: &RpcState) -> Result<Value, JsonRpcError> {
    let height = state.blockchain.height();
    let pending_count = state.tx_pool.pending_count();
    let pending_height = if pending_count > 0 { height + 1 } else { height };
    Ok(json!({
        "confirmed": format!("0x{:x}", height),
        "pending": format!("0x{:x}", pending_height),
        "pendingTransactions": pending_count
    }))
}

async fn eth_get_block_by_number(state: &RpcState, params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or(JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?;

    let args = params.as_array().ok_or(JsonRpcError {
        code: -32602,
        message: "Params must be an array".to_string(),
        data: None,
    })?;
    
    if args.is_empty() {
        return Err(JsonRpcError {
            code: -32602,
            message: "Missing block number".to_string(),
            data: None,
        });
    }

    let block_param = args[0].as_str().ok_or(JsonRpcError {
        code: -32602,
        message: "Block number must be a string".to_string(),
        data: None,
    })?;
    
    let include_txs = if args.len() > 1 {
        args[1].as_bool().unwrap_or(false)
    } else {
        false
    };

    let height = if block_param == "latest" {
        state.blockchain.height()
    } else if block_param == "earliest" {
        0
    } else if block_param == "pending" {
        state.blockchain.height() // TODO: Support pending block
    } else {
        let hex = block_param.strip_prefix("0x").unwrap_or(block_param);
        u64::from_str_radix(hex, 16).map_err(|_| JsonRpcError {
            code: -32602,
            message: "Invalid block number format".to_string(),
            data: None,
        })?
    };
    
    if let Some(block) = state.blockchain.get_block(height) {
        let transactions = if include_txs {
            let txs: Vec<Value> = block.transactions.iter().enumerate().map(|(i, tx)| {
                json!({
                    "hash": format!("0x{}", hex::encode(tx.hash().as_bytes())),
                    "nonce": format!("0x{:x}", tx.nonce),
                    "blockHash": format!("0x{}", hex::encode(block.hash().as_bytes())),
                    "blockNumber": format!("0x{:x}", block.header.height),
                    "transactionIndex": format!("0x{:x}", i),
                    "from": format!("0x{}", hex::encode(tx.from.as_bytes())),
                    "to": format!("0x{}", hex::encode(tx.to.as_bytes())),
                    "value": format!("0x{:x}", tx.amount),
                    "gas": format!("0x{:x}", tx.gas_limit),
                    "gasPrice": format!("0x{:x}", tx.gas_price),
                    "input": format!("0x{}", hex::encode(&tx.data)),
                })
            }).collect();
            json!(txs)
        } else {
            let tx_hashes: Vec<String> = block.transactions.iter()
                .map(|tx| format!("0x{}", hex::encode(tx.hash().as_bytes())))
                .collect();
            json!(tx_hashes)
        };
        
        // Calculate actual block size
        let block_size = bincode::serialized_size(&block).unwrap_or(0);
        
        Ok(json!({
            "number": format!("0x{:x}", block.header.height),
            "hash": format!("0x{}", hex::encode(block.hash().as_bytes())),
            "parentHash": format!("0x{}", hex::encode(block.header.prev_hash.as_bytes())),
            "nonce": format!("0x{:016x}", block.header.work),
            "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347", // Empty uncle hash
            "logsBloom": format!("0x{}", hex::encode(&EMPTY_BLOOM_FILTER)),
            "transactionsRoot": format!("0x{}", hex::encode(block.header.tx_root.as_bytes())),
            "stateRoot": format!("0x{}", hex::encode(block.header.state_root.as_bytes())),
            "receiptsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421", // Empty receipts root
            "miner": format!("0x{}", hex::encode(block.header.proposer.as_bytes())),
            "difficulty": "0x1",
            "totalDifficulty": format!("0x{:x}", block.header.height), // Simplified
            "extraData": "0x",
            "size": format!("0x{:x}", block_size),
            "gasLimit": "0x1fffffffffffff",
            "gasUsed": "0x0",
            "timestamp": format!("0x{:x}", block.header.timestamp),
            "transactions": transactions,
            "uncles": [],
            "vrfOutput": format!("0x{}", hex::encode(block.header.vrf_output)),
            "battleProofsCount": block.battle_proofs.len()
        }))
    } else {
        Ok(Value::Null)
    }
}

async fn eth_get_transaction_by_hash(state: &RpcState, params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or(JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?;

    let args = params.as_array().ok_or(JsonRpcError {
        code: -32602,
        message: "Params must be an array".to_string(),
        data: None,
    })?;
    
    if args.is_empty() {
        return Err(JsonRpcError {
            code: -32602,
            message: "Missing transaction hash".to_string(),
            data: None,
        });
    }

    let tx_hash_str = args[0].as_str().ok_or(JsonRpcError {
        code: -32602,
        message: "Transaction hash must be a string".to_string(),
        data: None,
    })?;
    
    let tx_hash_hex = tx_hash_str.strip_prefix("0x").unwrap_or(tx_hash_str);
    let tx_hash_bytes = hex::decode(tx_hash_hex).map_err(|_| JsonRpcError {
        code: -32602,
        message: "Invalid hex encoding".to_string(),
        data: None,
    })?;
    
    if tx_hash_bytes.len() != 32 {
        return Err(JsonRpcError {
            code: -32602,
            message: "Transaction hash must be 32 bytes".to_string(),
            data: None,
        });
    }
    
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&tx_hash_bytes);
    let target_hash = bitcell_crypto::Hash256::from(hash);
    
    // Use efficient O(1) lookup via transaction hash index
    if let Some((tx, location)) = state.blockchain.get_transaction_by_hash(&target_hash) {
        // Get the block to include block hash in response
        if let Some(block) = state.blockchain.get_block(location.block_height) {
            return Ok(json!({
                "hash": format!("0x{}", hex::encode(tx.hash().as_bytes())),
                "nonce": format!("0x{:x}", tx.nonce),
                "blockHash": format!("0x{}", hex::encode(block.hash().as_bytes())),
                "blockNumber": format!("0x{:x}", location.block_height),
                "transactionIndex": format!("0x{:x}", location.tx_index),
                "from": format!("0x{}", hex::encode(tx.from.as_bytes())),
                "to": format!("0x{}", hex::encode(tx.to.as_bytes())),
                "value": format!("0x{:x}", tx.amount),
                "gas": format!("0x{:x}", tx.gas_limit),
                "gasPrice": format!("0x{:x}", tx.gas_price),
                "input": format!("0x{}", hex::encode(&tx.data)),
            }));
        }
    }
    
    Ok(Value::Null)
}

async fn eth_get_balance(state: &RpcState, params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or(JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?;

    let args = params.as_array().ok_or(JsonRpcError {
        code: -32602,
        message: "Params must be an array".to_string(),
        data: None,
    })?;
    
    if args.is_empty() {
        return Err(JsonRpcError {
            code: -32602,
            message: "Missing address".to_string(),
            data: None,
        });
    }

    let address_str = args[0].as_str().ok_or(JsonRpcError {
        code: -32602,
        message: "Address must be a string".to_string(),
        data: None,
    })?;

    // Parse address (hex string to PublicKey)
    let address_hex = address_str.strip_prefix("0x").unwrap_or(address_str);
    let address_bytes = hex::decode(address_hex).map_err(|_| JsonRpcError {
        code: -32602,
        message: "Invalid address format".to_string(),
        data: None,
    })?;
    
    if address_bytes.len() != 33 {
        return Err(JsonRpcError {
            code: -32602,
            message: "Address must be 33 bytes (compressed public key)".to_string(),
            data: None,
        });
    }
    
    let mut address = [0u8; 33];
    address.copy_from_slice(&address_bytes);
    
    // Fetch balance from blockchain state
    let balance = {
        let state_lock = state.blockchain.state();
        let state = state_lock.read().map_err(|_| JsonRpcError {
            code: -32603,
            message: "Failed to acquire state lock".to_string(),
            data: None,
        })?;
        state.get_account(&address)
            .map(|account| account.balance)
            .unwrap_or(0)
    };
    
    // Return balance as hex string
    Ok(json!(format!("0x{:x}", balance)))
}

/// Get transaction count (nonce) for an address
async fn eth_get_transaction_count(state: &RpcState, params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or(JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?;

    let args = params.as_array().ok_or(JsonRpcError {
        code: -32602,
        message: "Params must be an array".to_string(),
        data: None,
    })?;
    
    if args.is_empty() {
        return Err(JsonRpcError {
            code: -32602,
            message: "Missing address".to_string(),
            data: None,
        });
    }

    let address_str = args[0].as_str().ok_or(JsonRpcError {
        code: -32602,
        message: "Address must be a string".to_string(),
        data: None,
    })?;

    // Parse address (hex string to PublicKey)
    let address_hex = address_str.strip_prefix("0x").unwrap_or(address_str);
    let address_bytes = hex::decode(address_hex).map_err(|_| JsonRpcError {
        code: -32602,
        message: "Invalid address format".to_string(),
        data: None,
    })?;
    
    if address_bytes.len() != 33 {
        return Err(JsonRpcError {
            code: -32602,
            message: "Address must be 33 bytes (compressed public key)".to_string(),
            data: None,
        });
    }
    
    let mut address = [0u8; 33];
    address.copy_from_slice(&address_bytes);
    
    // Fetch nonce from blockchain state
    let nonce = {
        let state_lock = state.blockchain.state();
        let state = state_lock.read().map_err(|_| JsonRpcError {
            code: -32603,
            message: "Failed to acquire state lock".to_string(),
            data: None,
        })?;
        state.get_account(&address)
            .map(|account| account.nonce)
            .unwrap_or(0)
    };
    
    // Return nonce as hex string
    Ok(json!(format!("0x{:x}", nonce)))
}

/// Default gas price in wei (1 Gwei)
const DEFAULT_GAS_PRICE: u64 = 1_000_000_000;

/// Get current gas price
/// 
/// Returns the current gas price. In production, this should be
/// dynamically calculated based on network congestion and mempool state.
async fn eth_gas_price(_state: &RpcState) -> Result<Value, JsonRpcError> {
    // TODO: Calculate dynamic gas price based on:
    // - Transaction pool congestion
    // - Recent block gas usage
    // - Priority fee market
    Ok(json!(format!("0x{:x}", DEFAULT_GAS_PRICE)))
}

async fn eth_send_raw_transaction(state: &RpcState, params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or(JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?;

    let args = params.as_array().ok_or(JsonRpcError {
        code: -32602,
        message: "Params must be an array".to_string(),
        data: None,
    })?;
    
    if args.is_empty() {
        return Err(JsonRpcError {
            code: -32602,
            message: "Missing transaction data".to_string(),
            data: None,
        });
    }

    let tx_data = args[0].as_str().ok_or(JsonRpcError {
        code: -32602,
        message: "Transaction data must be a string".to_string(),
        data: None,
    })?;

    // Decode hex transaction data
    let tx_hex = tx_data.strip_prefix("0x").unwrap_or(tx_data);
    let tx_bytes = hex::decode(tx_hex).map_err(|_| JsonRpcError {
        code: -32602,
        message: "Invalid hex encoding".to_string(),
        data: None,
    })?;
    
    // Deserialize transaction
    let tx: bitcell_consensus::Transaction = bincode::deserialize(&tx_bytes).map_err(|e| JsonRpcError {
        code: -32602,
        message: format!("Failed to deserialize transaction: {}", e),
        data: None,
    })?;
    
    // Validate transaction signature
    let tx_hash = tx.hash();
    if tx.signature.verify(&tx.from, tx_hash.as_bytes()).is_err() {
        return Err(JsonRpcError {
            code: -32602,
            message: "Invalid transaction signature".to_string(),
            data: None,
        });
    }
    
    // Validate nonce and balance
    {
        let state_lock = state.blockchain.state();
        let state_guard = state_lock.read().map_err(|_| JsonRpcError {
            code: -32603,
            message: "Failed to acquire state lock".to_string(),
            data: None,
        })?;
        
        if let Some(account) = state_guard.get_account(tx.from.as_bytes()) {
            if tx.nonce != account.nonce {
                return Err(JsonRpcError {
                    code: -32602,
                    message: format!("Invalid nonce: expected {}, got {}", account.nonce, tx.nonce),
                    data: None,
                });
            }
            
            if tx.amount > account.balance {
                return Err(JsonRpcError {
                    code: -32602,
                    message: "Insufficient balance".to_string(),
                    data: None,
                });
            }
        } else {
            // Account doesn't exist - allow transactions with nonce 0
            // This supports sending to/from new accounts that haven't been
            // credited yet (e.g., funding transactions from coinbase rewards)
            //
            // DoS Mitigation Notes:
            // 1. The transaction still needs a valid signature, preventing random spam
            // 2. The transaction pool has capacity limits that reject excess transactions
            // 3. Gas fees will be burned even if the transaction fails, discouraging abuse
            // 4. Future improvement: Add per-address rate limiting in the mempool
            if tx.nonce != 0 {
                return Err(JsonRpcError {
                    code: -32602,
                    message: format!("Account not found and nonce is not zero (got nonce {}). New accounts must start with nonce 0.", tx.nonce),
                    data: None,
                });
            }
            
            // Validate gas parameters to prevent spam and overflow attacks
            // Gas price and limit must be non-zero and within reasonable bounds
            const MAX_GAS_PRICE: u64 = 10_000_000_000_000; // 10,000 Gwei max
            const MAX_GAS_LIMIT: u64 = 30_000_000; // 30M gas max (similar to Ethereum block limit)
            
            if tx.gas_price == 0 || tx.gas_limit == 0 {
                return Err(JsonRpcError {
                    code: -32602,
                    message: "Transactions from new accounts require non-zero gas price and limit to prevent DoS attacks".to_string(),
                    data: None,
                });
            }
            
            if tx.gas_price > MAX_GAS_PRICE {
                return Err(JsonRpcError {
                    code: -32602,
                    message: format!("Gas price {} exceeds maximum allowed {}", tx.gas_price, MAX_GAS_PRICE),
                    data: None,
                });
            }
            
            if tx.gas_limit > MAX_GAS_LIMIT {
                return Err(JsonRpcError {
                    code: -32602,
                    message: format!("Gas limit {} exceeds maximum allowed {}", tx.gas_limit, MAX_GAS_LIMIT),
                    data: None,
                });
            }
            
            tracing::debug!(
                from = %hex::encode(tx.from.as_bytes()),
                "Allowing transaction from new account with nonce 0"
            );
        }
    }
    
    // Add to transaction pool
    if let Err(e) = state.tx_pool.add_transaction(tx.clone()) {
        return Err(JsonRpcError {
            code: -32603,
            message: format!("Failed to add transaction to pool: {}", e),
            data: None,
        });
    }
    
    // Return transaction hash
    Ok(json!(format!("0x{}", hex::encode(tx_hash.as_bytes()))))
}

/// Get node information including ID, version, and capabilities
async fn bitcell_get_node_info(state: &RpcState) -> Result<Value, JsonRpcError> {
    Ok(json!({
        "node_id": state.node_id,
        "version": "0.1.0",
        "protocol_version": "1",
        "network_id": "bitcell-testnet",
        "api_version": "0.1-alpha",
        "capabilities": ["bitcell/1"],
        "node_type": state.node_type,
        "chain_height": state.blockchain.height(),
        "peer_count": state.network.peer_count(),
    }))
}

async fn bitcell_get_peer_count(state: &RpcState) -> Result<Value, JsonRpcError> {
    let count = state.network.peer_count();
    Ok(json!(count))
}

async fn bitcell_get_network_metrics(state: &RpcState) -> Result<Value, JsonRpcError> {
    Ok(json!({
        "peer_count": state.network.peer_count(),
        "height": format!("0x{:x}", state.blockchain.height()),
        "version": "0.1.0",
        // TODO: Add more metrics
    }))
}

async fn bitcell_get_tournament_state(state: &RpcState) -> Result<Value, JsonRpcError> {
    if let Some(tm) = &state.tournament_manager {
        let phase = tm.current_phase().await;
        let phase_str = match phase {
            Some(p) => format!("{:?}", p).to_lowercase(),
            None => "idle".to_string(),
        };
        
        let block_height = state.blockchain.height();
        let last_winner = if block_height > 0 {
            state.blockchain
                .get_block(block_height - 1)
                .map(|b| format!("{:?}", b.header.proposer))
                .unwrap_or_else(|| "None".to_string())
        } else {
            "None".to_string()
        };
        
        Ok(json!({
            "block": format!("0x{:x}", block_height),
            "current_round": block_height,
            "phase": phase_str,
            "last_winner": last_winner,
        }))
    } else {
        Ok(json!({
            "block": format!("0x{:x}", state.blockchain.height()),
            "current_round": state.blockchain.height(),
            "phase": "unknown",
            "last_winner": "None",
            "note": "Tournament state not available on this node type"
        }))
    }
}

async fn bitcell_submit_commitment(state: &RpcState, params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or(JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?;

    // Expecting [ { "commitment": "...", "ring_signature": "..." } ]
    let args = params.as_array().ok_or(JsonRpcError {
        code: -32602,
        message: "Params must be an array".to_string(),
        data: None,
    })?;
    
    if args.is_empty() {
        return Err(JsonRpcError {
            code: -32602,
            message: "Missing arguments".to_string(),
            data: None,
        });
    }

    let obj = args[0].as_object().ok_or(JsonRpcError {
        code: -32602,
        message: "Argument must be an object".to_string(),
        data: None,
    })?;

    let commitment_str = obj.get("commitment").and_then(|v| v.as_str()).ok_or(JsonRpcError {
        code: -32602,
        message: "Missing commitment".to_string(),
        data: None,
    })?;
    
    let signature_str = obj.get("ring_signature").and_then(|v| v.as_str()).ok_or(JsonRpcError {
        code: -32602,
        message: "Missing ring_signature".to_string(),
        data: None,
    })?;

    // Parse hex strings
    // TODO: Use proper hex parsing util
    let commitment_bytes = hex::decode(commitment_str.trim_start_matches("0x")).map_err(|_| JsonRpcError {
        code: -32602,
        message: "Invalid hex for commitment".to_string(),
        data: None,
    })?;
    
    let signature_bytes = hex::decode(signature_str.trim_start_matches("0x")).map_err(|_| JsonRpcError {
        code: -32602,
        message: "Invalid hex for signature".to_string(),
        data: None,
    })?;

    if commitment_bytes.len() != 32 {
        return Err(JsonRpcError {
            code: -32602,
            message: "Commitment must be 32 bytes".to_string(),
            data: None,
        });
    }

    let mut commitment_hash = [0u8; 32];
    commitment_hash.copy_from_slice(&commitment_bytes);

    if let Some(tm) = &state.tournament_manager {
        let commitment = bitcell_consensus::GliderCommitment {
            commitment: bitcell_crypto::Hash256::from(commitment_hash),
            ring_signature: signature_bytes,
            height: state.blockchain.height() + 1, // Committing for next block
        };

        tm.add_commitment(commitment).await.map_err(|e| JsonRpcError {
            code: -32000,
            message: format!("Failed to submit commitment: {}", e),
            data: None,
        })?;

        Ok(json!({
            "accepted": true
        }))
    } else {
        Err(JsonRpcError {
            code: -32000,
            message: "Node does not support tournament operations".to_string(),
            data: None,
        })
    }
}

async fn bitcell_submit_reveal(state: &RpcState, params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or(JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?;

    let args = params.as_array().ok_or(JsonRpcError {
        code: -32602,
        message: "Params must be an array".to_string(),
        data: None,
    })?;
    
    if args.is_empty() {
        return Err(JsonRpcError {
            code: -32602,
            message: "Missing arguments".to_string(),
            data: None,
        });
    }

    let obj = args[0].as_object().ok_or(JsonRpcError {
        code: -32602,
        message: "Argument must be an object".to_string(),
        data: None,
    })?;

    // Parse fields
    let miner_str = obj.get("miner").and_then(|v| v.as_str()).ok_or(JsonRpcError {
        code: -32602,
        message: "Missing miner".to_string(),
        data: None,
    })?;
    
    let nonce_str = obj.get("nonce").and_then(|v| v.as_str()).ok_or(JsonRpcError {
        code: -32602,
        message: "Missing nonce".to_string(),
        data: None,
    })?;
    
    let glider_obj = obj.get("glider").and_then(|v| v.as_object()).ok_or(JsonRpcError {
        code: -32602,
        message: "Missing glider".to_string(),
        data: None,
    })?;

    // Parse miner public key
    // TODO: Use proper hex parsing util
    let miner_bytes = hex::decode(miner_str.trim_start_matches("0x")).map_err(|_| JsonRpcError {
        code: -32602,
        message: "Invalid hex for miner".to_string(),
        data: None,
    })?;
    
    if miner_bytes.len() != 33 { // Compressed public key
        return Err(JsonRpcError {
            code: -32602,
            message: "Miner public key must be 33 bytes".to_string(),
            data: None,
        });
    }
    
    let mut miner_bytes_arr = [0u8; 33];
    miner_bytes_arr.copy_from_slice(&miner_bytes);
    
    let miner_pk = bitcell_crypto::PublicKey::from_bytes(miner_bytes_arr).map_err(|_| JsonRpcError {
        code: -32602,
        message: "Invalid miner public key".to_string(),
        data: None,
    })?;

    // Parse nonce
    let nonce_bytes = hex::decode(nonce_str.trim_start_matches("0x")).map_err(|_| JsonRpcError {
        code: -32602,
        message: "Invalid hex for nonce".to_string(),
        data: None,
    })?;

    // Parse Glider
    let pattern_str = glider_obj.get("pattern").and_then(|v| v.as_str()).ok_or(JsonRpcError {
        code: -32602,
        message: "Missing glider pattern".to_string(),
        data: None,
    })?;
    
    let pattern = match pattern_str {
        "Standard" => bitcell_ca::GliderPattern::Standard,
        "Lightweight" => bitcell_ca::GliderPattern::Lightweight,
        "Middleweight" => bitcell_ca::GliderPattern::Middleweight,
        "Heavyweight" => bitcell_ca::GliderPattern::Heavyweight,
        _ => return Err(JsonRpcError {
            code: -32602,
            message: "Unknown glider pattern".to_string(),
            data: None,
        }),
    };
    
    // Default position for now as it's set by battle logic usually, but struct requires it
    let glider = bitcell_ca::Glider::new(pattern, bitcell_ca::Position::new(0, 0));

    if let Some(tm) = &state.tournament_manager {
        let reveal = bitcell_consensus::GliderReveal {
            glider,
            nonce: nonce_bytes,
            miner: miner_pk,
        };

        tm.add_reveal(reveal).await.map_err(|e| JsonRpcError {
            code: -32000,
            message: format!("Failed to submit reveal: {}", e),
            data: None,
        })?;

        Ok(json!({
            "accepted": true
        }))
    } else {
        Err(JsonRpcError {
            code: -32000,
            message: "Node does not support tournament operations".to_string(),
            data: None,
        })
    }
}

async fn bitcell_get_battle_replay(state: &RpcState, params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or(JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?;
    
    let args = params.as_array().ok_or(JsonRpcError {
        code: -32602,
        message: "Params must be an array".to_string(),
        data: None,
    })?;
    
    if args.is_empty() {
        return Err(JsonRpcError {
            code: -32602,
            message: "Missing arguments (block_height)".to_string(),
            data: None,
        });
    }
    
    let block_height = args[0].as_u64().ok_or(JsonRpcError {
        code: -32602,
        message: "Invalid block height".to_string(),
        data: None,
    })?;
    
    // In a real implementation, we would fetch the match from history
    // For now, we'll generate a deterministic simulation based on the block height
    // so that it looks consistent for the same block
    
    use bitcell_ca::{Battle, Glider, GliderPattern, grid::Position};
    
    // Create deterministic gliders based on block height
    // This simulates different miners submitting different strategies
    let seed = block_height;
    
    let pattern_a = match seed % 3 {
        0 => GliderPattern::Standard,
        1 => GliderPattern::Heavyweight,
        _ => GliderPattern::Lightweight,
    };
    
    let pattern_b = match (seed + 1) % 3 {
        0 => GliderPattern::Standard,
        1 => GliderPattern::Heavyweight,
        _ => GliderPattern::Lightweight,
    };
    
    let glider_a = Glider::new(pattern_a, Position::new(256, 512));
    let glider_b = Glider::new(pattern_b, Position::new(768, 512));
    
    // Create battle with entropy derived from block height
    let mut entropy = [0u8; 32];
    for i in 0..8 {
        entropy[i] = ((seed >> (i * 8)) & 0xFF) as u8;
    }
    
    let battle = Battle::with_entropy(glider_a, glider_b, 100, entropy);
    
    // Get grid states at intervals for visualization
    // We'll take 10 snapshots
    let sample_steps: Vec<usize> = (0..=100).step_by(10).collect();
    let grids = battle.grid_states(&sample_steps);
    
    // Serialize grids to simple 2D arrays for JSON
    let serialized_grids: Vec<Vec<Vec<u8>>> = grids.iter().map(|grid| {
        // Downsample for UI performance (1024x1024 is too big for JSON)
        // We'll return a 64x64 view centered on the action
        let view_size = 64;
        let center_y = 512;
        let center_x = 512;
        let start_y = center_y - view_size / 2;
        let start_x = center_x - view_size / 2;
        
        let mut view = vec![vec![0u8; view_size]; view_size];
        
        for y in 0..view_size {
            for x in 0..view_size {
                let pos = Position::new(start_x + x, start_y + y);
                let cell = grid.get(pos);
                if cell.is_alive() {
                    // 1 for Player A (left), 2 for Player B (right)
                    // Simplified logic: left side is A, right side is B
                    view[y][x] = if (start_x + x) < 512 { 1 } else { 2 };
                }
            }
        }
        view
    }).collect();
    
    let outcome = battle.simulate();
    let outcome_str = match outcome {
        bitcell_ca::BattleOutcome::AWins => "Miner A Wins",
        bitcell_ca::BattleOutcome::BWins => "Miner B Wins",
        bitcell_ca::BattleOutcome::Tie => "Tie",
    };
    
    Ok(json!({
        "block_height": block_height,
        "grid_states": serialized_grids,
        "outcome": outcome_str
    }))
}

async fn bitcell_get_reputation(state: &RpcState, params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or(JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?;
    
    let args = params.as_array().ok_or(JsonRpcError {
        code: -32602,
        message: "Params must be an array".to_string(),
        data: None,
    })?;
    
    if args.is_empty() {
        return Err(JsonRpcError {
            code: -32602,
            message: "Missing miner ID".to_string(),
            data: None,
        });
    }
    
    let miner_id_str = args[0].as_str().ok_or(JsonRpcError {
        code: -32602,
        message: "Miner ID must be a string".to_string(),
        data: None,
    })?;
    
    if let Some(tm) = &state.tournament_manager {
        // TODO: Expose reputation from TournamentManager
        // For now, return placeholder
         Ok(json!({
            "miner": miner_id_str,
            "trust": 0.8,
            "r": 10,
            "s": 2
        }))
    } else {
         Ok(json!({
            "miner": miner_id_str,
            "trust": 0.0,
            "note": "Reputation not available"
        }))
    }
}

async fn bitcell_get_miner_stats(state: &RpcState, params: Option<Value>) -> Result<Value, JsonRpcError> {
     Ok(json!({
        "miner": "TODO",
        "aggression_index": 0.5,
        "volatility_index": 0.2,
        "win_rate": 0.6,
        "tournaments_played": 10,
        "finals_reached": 3
    }))
}

// --- REST API Router ---

fn api_router() -> Router<RpcState> {
    Router::new()
        .route("/wallet/balance/:address", get(get_balance))
        .route("/mining/status", get(get_mining_status))
}

// --- REST Handlers ---

async fn get_balance(
    State(state): State<RpcState>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    // TODO: Validate address and fetch real balance from state
    // For now, return mock data
    Json(json!({
        "address": address,
        "balance": "0",
        "confirmed_balance": "0",
        "unconfirmed_balance": "0"
    }))
}

async fn get_mining_status(
    State(state): State<RpcState>,
) -> impl IntoResponse {
    let phase = if let Some(tm) = &state.tournament_manager {
        match tm.current_phase().await {
            Some(p) => format!("{:?}", p).to_lowercase(),
            None => "idle".to_string(),
        }
    } else {
        "unknown".to_string()
    };

    Json(json!({
        "phase": phase,
        "height": format!("0x{:x}", state.blockchain.height()),
        "auto_miner": false // TODO: Check auto miner status
    }))
}
