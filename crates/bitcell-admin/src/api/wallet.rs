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

/// Wallet API Router
pub fn router() -> Router<Arc<ConfigManager>> {
    Router::new()
        .route("/balance/:address", get(get_balance))
        .route("/send", post(send_transaction))
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
    to: String,
    amount: String,
    fee: String,
    memo: Option<String>,
}

#[derive(Debug, Serialize)]
struct SendTransactionResponse {
    tx_hash: String,
    status: String,
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

/// Send transaction
///
/// This endpoint is currently not implemented. A full implementation requires:
/// 1. Private key management in the admin console
/// 2. Transaction building with proper gas estimation
/// 3. Transaction signing with the managed key
/// 4. RLP encoding of the signed transaction
/// 5. Submission via eth_sendRawTransaction RPC
///
/// For now, returns NOT_IMPLEMENTED status code.
async fn send_transaction(
    State(_config_manager): State<Arc<ConfigManager>>,
    Json(_req): Json<SendTransactionRequest>,
) -> impl IntoResponse {
    // Transaction sending requires proper implementation of:
    // - Private key management (secure storage, HSM integration)
    // - Transaction building (nonce fetching, gas estimation)
    // - Transaction signing (ECDSA with secp256k1)
    // - RLP encoding for broadcast
    //
    // Until these are implemented, return NOT_IMPLEMENTED
    (StatusCode::NOT_IMPLEMENTED, "Transaction sending not yet implemented. Requires: key management, tx building, signing, and RLP encoding.").into_response()
}
