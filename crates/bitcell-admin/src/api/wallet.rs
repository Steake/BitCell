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
async fn send_transaction(
    State(_config_manager): State<Arc<ConfigManager>>,
    Json(_req): Json<SendTransactionRequest>,
) -> impl IntoResponse {
    // Transaction sending is not yet implemented.
    // In a real implementation, we would:
    // 1. Create a transaction object
    // 2. Sign it with a key managed by admin console (or passed in)
    // 3. Encode it
    // 4. Send via eth_sendRawTransaction
    (StatusCode::NOT_IMPLEMENTED, "Transaction sending is not yet implemented. Please use the wallet CLI or GUI to send transactions.").into_response()
}
