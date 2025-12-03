use axum::{
    extract::{State, Json, Path},
    routing::{get, post},
    Router,
    response::{IntoResponse},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use crate::config::ConfigManager;
use bitcell_wallet::{Chain, Transaction as WalletTx};
use bitcell_crypto::SecretKey;

/// Wallet API Router
pub fn router() -> Router<Arc<ConfigManager>> {
    Router::new()
        .route("/balance/:address", get(get_balance))
        .route("/send", post(send_transaction))
        .route("/nonce/:address", get(get_nonce))
}

#[derive(Debug, Serialize)]
struct BalanceResponse {
    address: String,
    balance: String,
    confirmed_balance: String,
    unconfirmed_balance: String,
}

#[derive(Debug, Deserialize)]
struct SendTransactionRequest {
    /// Sender address (hex string)
    from: String,
    /// Recipient address (hex string)
    to: String,
    /// Amount in smallest units (as string to avoid float precision issues)
    amount: String,
    /// Fee in smallest units
    fee: String,
    /// Optional private key (hex string) for signing - INSECURE, for testing only
    /// In production, use proper key management (HSM, hardware wallet, etc.)
    #[serde(default)]
    private_key: Option<String>,
    /// Optional memo
    memo: Option<String>,
}

#[derive(Debug, Serialize)]
struct SendTransactionResponse {
    tx_hash: String,
    status: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct NonceResponse {
    address: String,
    nonce: u64,
}

/// Get wallet balance
async fn get_balance(
    State(config_manager): State<Arc<ConfigManager>>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    // Get config
    let config = match config_manager.get_config() {
        Ok(c) => c,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get config").into_response(),
    };

    // Call bitcell-node RPC eth_getBalance
    let rpc_url = format!("http://{}:{}/rpc", config.wallet.node_rpc_host, config.wallet.node_rpc_port);
    
    let client = reqwest::Client::new();
    let rpc_req = json!({
        "jsonrpc": "2.0",
        "method": "eth_getBalance",
        "params": [address, "latest"],
        "id": 1
    });

    match client.post(&rpc_url).json(&rpc_req).send().await {
        Ok(resp) => {
            if let Ok(json) = resp.json::<Value>().await {
                if let Some(result) = json.get("result").and_then(|v| v.as_str()) {
                    // Parse hex balance
                    // For now just return as string
                    return Json(BalanceResponse {
                        address,
                        balance: result.to_string(),
                        confirmed_balance: result.to_string(),
                        unconfirmed_balance: "0".to_string(),
                    }).into_response();
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to call RPC: {}", e);
        }
    }

    // Fallback/Error
    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch balance").into_response()
}

/// Get account nonce for transaction building
async fn get_nonce(
    State(config_manager): State<Arc<ConfigManager>>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    let config = match config_manager.get_config() {
        Ok(c) => c,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get config").into_response(),
    };

    let rpc_url = format!("http://{}:{}/rpc", config.wallet.node_rpc_host, config.wallet.node_rpc_port);
    
    let client = reqwest::Client::new();
    let rpc_req = json!({
        "jsonrpc": "2.0",
        "method": "eth_getTransactionCount",
        "params": [address, "latest"],
        "id": 1
    });

    match client.post(&rpc_url).json(&rpc_req).send().await {
        Ok(resp) => {
            if let Ok(json) = resp.json::<Value>().await {
                if let Some(result) = json.get("result").and_then(|v| v.as_str()) {
                    // Parse hex nonce
                    let nonce = u64::from_str_radix(result.trim_start_matches("0x"), 16)
                        .unwrap_or(0);
                    return Json(NonceResponse {
                        address,
                        nonce,
                    }).into_response();
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to get nonce: {}", e);
        }
    }

    // Default to nonce 0 for new accounts
    Json(NonceResponse { address, nonce: 0 }).into_response()
}

/// Send transaction
/// 
/// This endpoint builds, signs, and broadcasts a transaction.
/// 
/// **Security Warning**: Providing a private key via API is insecure.
/// This is intended for testing purposes only. Production systems should use:
/// - Hardware wallets (Ledger, Trezor)
/// - HSM (Hardware Security Module)
/// - Secure key management services
/// - Multi-sig setups
async fn send_transaction(
    State(config_manager): State<Arc<ConfigManager>>,
    Json(req): Json<SendTransactionRequest>,
) -> impl IntoResponse {
    // Validate request
    if req.from.is_empty() || req.to.is_empty() {
        return Json(SendTransactionResponse {
            tx_hash: String::new(),
            status: "error".to_string(),
            message: "Missing from or to address".to_string(),
        }).into_response();
    }

    let amount: u64 = match req.amount.parse() {
        Ok(a) => a,
        Err(_) => return Json(SendTransactionResponse {
            tx_hash: String::new(),
            status: "error".to_string(),
            message: "Invalid amount format".to_string(),
        }).into_response(),
    };

    let fee: u64 = match req.fee.parse() {
        Ok(f) => f,
        Err(_) => return Json(SendTransactionResponse {
            tx_hash: String::new(),
            status: "error".to_string(),
            message: "Invalid fee format (must be a positive integer)".to_string(),
        }).into_response(),
    };

    // Check for private key
    let private_key = match &req.private_key {
        Some(pk) if !pk.is_empty() => pk,
        _ => {
            return Json(SendTransactionResponse {
                tx_hash: String::new(),
                status: "error".to_string(),
                message: "Private key required for signing. For security, use proper key management in production.".to_string(),
            }).into_response();
        }
    };

    // Parse private key
    let secret_key = match hex::decode(private_key.trim_start_matches("0x")) {
        Ok(bytes) if bytes.len() == 32 => {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            match SecretKey::from_bytes(&arr) {
                Ok(sk) => sk,
                Err(_) => return Json(SendTransactionResponse {
                    tx_hash: String::new(),
                    status: "error".to_string(),
                    message: "Invalid private key format".to_string(),
                }).into_response(),
            }
        }
        _ => return Json(SendTransactionResponse {
            tx_hash: String::new(),
            status: "error".to_string(),
            message: "Private key must be 32 bytes hex".to_string(),
        }).into_response(),
    };

    // Get config
    let config = match config_manager.get_config() {
        Ok(c) => c,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get config").into_response(),
    };

    let rpc_url = format!("http://{}:{}/rpc", config.wallet.node_rpc_host, config.wallet.node_rpc_port);
    let client = reqwest::Client::new();

    // Step 1: Get nonce
    let nonce_req = json!({
        "jsonrpc": "2.0",
        "method": "eth_getTransactionCount",
        "params": [&req.from, "latest"],
        "id": 1
    });

    let nonce: u64 = match client.post(&rpc_url).json(&nonce_req).send().await {
        Ok(resp) => {
            if let Ok(json) = resp.json::<Value>().await {
                if let Some(result) = json.get("result").and_then(|v| v.as_str()) {
                    u64::from_str_radix(result.trim_start_matches("0x"), 16).unwrap_or(0)
                } else {
                    0
                }
            } else {
                0
            }
        }
        Err(_) => 0,
    };

    // Step 2: Build transaction
    let tx = WalletTx::new(
        Chain::BitCell,
        req.from.clone(),
        req.to.clone(),
        amount,
        fee,
        nonce,
    ).with_data(req.memo.unwrap_or_default().into_bytes());

    // Step 3: Sign transaction
    let signed_tx = tx.sign(&secret_key);

    // Step 4: Serialize for broadcast
    let tx_bytes = match signed_tx.serialize() {
        Ok(b) => b,
        Err(e) => return Json(SendTransactionResponse {
            tx_hash: String::new(),
            status: "error".to_string(),
            message: format!("Failed to serialize transaction: {}", e),
        }).into_response(),
    };

    let tx_hex = format!("0x{}", hex::encode(&tx_bytes));

    // Step 5: Broadcast via RPC
    let send_req = json!({
        "jsonrpc": "2.0",
        "method": "eth_sendRawTransaction",
        "params": [tx_hex],
        "id": 1
    });

    match client.post(&rpc_url).json(&send_req).send().await {
        Ok(resp) => {
            if let Ok(json) = resp.json::<Value>().await {
                if let Some(error) = json.get("error") {
                    return Json(SendTransactionResponse {
                        tx_hash: String::new(),
                        status: "error".to_string(),
                        message: format!("RPC error: {}", error),
                    }).into_response();
                }
                
                if let Some(result) = json.get("result").and_then(|v| v.as_str()) {
                    return Json(SendTransactionResponse {
                        tx_hash: result.to_string(),
                        status: "submitted".to_string(),
                        message: "Transaction submitted successfully".to_string(),
                    }).into_response();
                }
            }
        }
        Err(e) => {
            return Json(SendTransactionResponse {
                tx_hash: String::new(),
                status: "error".to_string(),
                message: format!("Failed to broadcast: {}", e),
            }).into_response();
        }
    }

    // Use signed transaction hash as fallback
    Json(SendTransactionResponse {
        tx_hash: signed_tx.hash_hex(),
        status: "submitted".to_string(),
        message: "Transaction built and signed, broadcast may be pending".to_string(),
    }).into_response()
}
