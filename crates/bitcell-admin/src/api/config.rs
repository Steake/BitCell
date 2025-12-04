//! Configuration API endpoints

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub network: NetworkConfig,
    pub consensus: ConsensusConfig,
    pub ebsl: EbslConfig,
    pub economics: EconomicsConfig,
    pub wallet: WalletConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkConfig {
    pub listen_addr: String,
    pub bootstrap_peers: Vec<String>,
    pub max_peers: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConsensusConfig {
    pub battle_steps: usize,
    pub tournament_rounds: usize,
    pub block_time: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EbslConfig {
    pub evidence_threshold: f64,
    pub slash_percentage: f64,
    pub decay_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EconomicsConfig {
    pub initial_reward: u64,
    pub halving_interval: u64,
    pub base_gas_price: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WalletConfig {
    pub node_rpc_host: String,
    pub node_rpc_port: u16,
}

/// Get current configuration
pub async fn get_config(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Config>, (StatusCode, Json<String>)> {
    match state.config.get_config() {
        Ok(config) => Ok(Json(config)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(format!("Failed to get config: {}", e)),
        )),
    }
}

/// Update configuration
pub async fn update_config(
    State(state): State<Arc<AppState>>,
    Json(config): Json<Config>,
) -> Result<Json<Config>, (StatusCode, Json<String>)> {
    match state.config.update_config(config.clone()) {
        Ok(_) => Ok(Json(config)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(format!("Failed to update config: {}", e)),
        )),
    }
}
