//! Testing utilities API endpoints

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct RunBattleTestRequest {
    pub glider_a: String,
    pub glider_b: String,
    pub steps: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct BattleTestResponse {
    pub test_id: String,
    pub winner: String,
    pub steps: usize,
    pub final_energy_a: u64,
    pub final_energy_b: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct SendTestTransactionRequest {
    pub from: Option<String>,
    pub to: String,
    pub amount: u64,
}

#[derive(Debug, Serialize)]
pub struct TransactionTestResponse {
    pub tx_hash: String,
    pub status: String,
    pub message: String,
}

/// Run a battle test
pub async fn run_battle_test(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<RunBattleTestRequest>,
) -> Result<Json<BattleTestResponse>, (StatusCode, Json<String>)> {
    // TODO: Actually run battle simulation
    // For now, return mock response

    let test_id = format!("test-{}", chrono::Utc::now().timestamp());

    let response = BattleTestResponse {
        test_id,
        winner: "glider_a".to_string(),
        steps: req.steps.unwrap_or(1000),
        final_energy_a: 8500,
        final_energy_b: 7200,
        duration_ms: 235,
    };

    Ok(Json(response))
}

/// Send a test transaction
pub async fn send_test_transaction(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<SendTestTransactionRequest>,
) -> Result<Json<TransactionTestResponse>, (StatusCode, Json<String>)> {
    // TODO: Actually send transaction
    // For now, return mock response

    let tx_hash = format!("0x{:x}", chrono::Utc::now().timestamp());

    let response = TransactionTestResponse {
        tx_hash,
        status: "pending".to_string(),
        message: format!("Test transaction sent: {} -> {}",
            req.from.unwrap_or_else(|| "genesis".to_string()),
            req.to
        ),
    };

    Ok(Json(response))
}
