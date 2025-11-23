//! Metrics API endpoints

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;

#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub chain: ChainMetrics,
    pub network: NetworkMetrics,
    pub ebsl: EbslMetrics,
    pub system: SystemMetrics,
}

#[derive(Debug, Serialize)]
pub struct ChainMetrics {
    pub height: u64,
    pub latest_block_hash: String,
    pub latest_block_time: chrono::DateTime<chrono::Utc>,
    pub total_transactions: u64,
    pub pending_transactions: u64,
    pub average_block_time: f64,
}

#[derive(Debug, Serialize)]
pub struct NetworkMetrics {
    pub connected_peers: usize,
    pub total_peers: usize,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
}

#[derive(Debug, Serialize)]
pub struct EbslMetrics {
    pub active_miners: usize,
    pub banned_miners: usize,
    pub average_trust_score: f64,
    pub total_slashing_events: u64,
}

#[derive(Debug, Serialize)]
pub struct SystemMetrics {
    pub uptime_seconds: u64,
    pub cpu_usage: f64,
    pub memory_usage_mb: u64,
    pub disk_usage_mb: u64,
}

/// Get all metrics
pub async fn get_metrics(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<MetricsResponse>, (StatusCode, Json<String>)> {
    // TODO: Integrate with actual Prometheus metrics
    // For now, return mock data

    let response = MetricsResponse {
        chain: ChainMetrics {
            height: 12345,
            latest_block_hash: "0x1234567890abcdef".to_string(),
            latest_block_time: chrono::Utc::now(),
            total_transactions: 54321,
            pending_transactions: 42,
            average_block_time: 6.5,
        },
        network: NetworkMetrics {
            connected_peers: 8,
            total_peers: 12,
            bytes_sent: 1_234_567,
            bytes_received: 2_345_678,
            messages_sent: 9876,
            messages_received: 8765,
        },
        ebsl: EbslMetrics {
            active_miners: 25,
            banned_miners: 3,
            average_trust_score: 0.87,
            total_slashing_events: 15,
        },
        system: SystemMetrics {
            uptime_seconds: 86400,
            cpu_usage: 45.2,
            memory_usage_mb: 2048,
            disk_usage_mb: 10240,
        },
    };

    Ok(Json(response))
}

/// Get chain-specific metrics
pub async fn chain_metrics(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<ChainMetrics>, (StatusCode, Json<String>)> {
    let metrics = ChainMetrics {
        height: 12345,
        latest_block_hash: "0x1234567890abcdef".to_string(),
        latest_block_time: chrono::Utc::now(),
        total_transactions: 54321,
        pending_transactions: 42,
        average_block_time: 6.5,
    };

    Ok(Json(metrics))
}

/// Get network-specific metrics
pub async fn network_metrics(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<NetworkMetrics>, (StatusCode, Json<String>)> {
    let metrics = NetworkMetrics {
        connected_peers: 8,
        total_peers: 12,
        bytes_sent: 1_234_567,
        bytes_received: 2_345_678,
        messages_sent: 9876,
        messages_received: 8765,
    };

    Ok(Json(metrics))
}
