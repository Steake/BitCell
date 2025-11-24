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

#[derive(Debug, Clone, Serialize)]
pub struct ChainMetrics {
    pub height: u64,
    pub latest_block_hash: String,
    pub latest_block_time: chrono::DateTime<chrono::Utc>,
    pub total_transactions: u64,
    pub pending_transactions: u64,
    pub average_block_time: f64,
}

#[derive(Debug, Clone, Serialize)]
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

/// Get all metrics from running nodes
pub async fn get_metrics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<MetricsResponse>, (StatusCode, Json<String>)> {
    let nodes = state.setup.get_nodes();

    if nodes.is_empty() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json("No nodes configured. Please complete setup wizard and deploy nodes first.".to_string()),
        ));
    }

    // Get endpoints for metrics fetching
    let endpoints: Vec<(String, String)> = nodes
        .iter()
        .map(|n| (n.id.clone(), n.metrics_endpoint.clone()))
        .collect();

    // Fetch aggregated metrics
    let aggregated = state.metrics_client.aggregate_metrics(&endpoints)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e)))?;

    // Calculate system metrics
    // TODO: Track actual node start times to compute real uptime
    let uptime_seconds = 0u64; // Placeholder - requires node start time tracking

    let response = MetricsResponse {
        chain: ChainMetrics {
            height: aggregated.chain_height,
            latest_block_hash: format!("0x{:016x}", aggregated.chain_height), // Simplified
            latest_block_time: chrono::Utc::now(),
            total_transactions: aggregated.total_txs_processed,
            pending_transactions: aggregated.pending_txs as u64,
            average_block_time: 6.0, // TODO: Calculate from actual block times
        },
        network: NetworkMetrics {
            connected_peers: aggregated.total_peers,
            total_peers: aggregated.total_nodes * 10, // Estimate
            bytes_sent: aggregated.bytes_sent,
            bytes_received: aggregated.bytes_received,
            messages_sent: 0, // TODO: Requires adding message_sent to node metrics
            messages_received: 0, // TODO: Requires adding message_received to node metrics
        },
        ebsl: EbslMetrics {
            active_miners: aggregated.active_miners,
            banned_miners: aggregated.banned_miners,
            average_trust_score: 0.85, // TODO: Requires adding trust scores to node metrics
            total_slashing_events: 0, // TODO: Requires adding slashing events to node metrics
        },
        system: SystemMetrics {
            uptime_seconds,
            cpu_usage: 0.0, // TODO: Requires system metrics collection (e.g., sysinfo crate)
            memory_usage_mb: 0, // TODO: Requires system metrics collection
            disk_usage_mb: 0, // TODO: Requires system metrics collection
        },
    };

    Ok(Json(response))
}

/// Get chain-specific metrics
pub async fn chain_metrics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ChainMetrics>, (StatusCode, Json<String>)> {
    // Reuse get_metrics logic and extract chain metrics
    let full_metrics = get_metrics(State(state)).await?;
    Ok(Json(full_metrics.chain.clone()))
}

/// Get network-specific metrics
pub async fn network_metrics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<NetworkMetrics>, (StatusCode, Json<String>)> {
    // Reuse get_metrics logic and extract network metrics
    let full_metrics = get_metrics(State(state)).await?;
    Ok(Json(full_metrics.network.clone()))
}
