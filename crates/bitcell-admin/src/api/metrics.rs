//! Metrics API endpoints
//!
//! Provides real-time system and network metrics for monitoring.

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Serialize;
use std::sync::Arc;

use crate::AppState;

#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub chain: ChainMetrics,
    pub network: NetworkMetrics,
    pub ebsl: EbslMetrics,
    pub system: SystemMetrics,
    pub node_metrics: Option<Vec<crate::metrics_client::NodeMetrics>>,
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

#[derive(Debug, Clone, Serialize)]
pub struct SystemMetrics {
    pub uptime_seconds: u64,
    pub cpu_usage: f64,
    pub memory_usage_mb: u64,
    pub total_memory_mb: u64,
    pub disk_usage_mb: u64,
    pub total_disk_mb: u64,
}

/// Get all metrics from running nodes
pub async fn get_metrics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<MetricsResponse>, (StatusCode, Json<String>)> {
    // Collect real system metrics
    let sys_metrics = state.system_metrics.collect();
    
    // Get all registered nodes from ProcessManager (which has status info)
    let all_nodes = state.process.list_nodes();
    tracing::info!("get_metrics: Found {} nodes", all_nodes.len());
    
    if all_nodes.is_empty() {
        tracing::warn!("get_metrics: No nodes found, returning 503");
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json("No nodes configured. Please deploy nodes first.".to_string()),
        ));
    }

    // Get endpoints for metrics fetching (try all nodes)
    let endpoints: Vec<(String, String)> = all_nodes
        .iter()
        .map(|n| {
            let metrics_port = n.port + 1; // Metrics port is node port + 1
            (n.id.clone(), format!("http://127.0.0.1:{}/metrics", metrics_port))
        })
        .collect();

    if endpoints.is_empty() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json("No running nodes. Please start some nodes first.".to_string()),
        ));
    }

    // Fetch aggregated metrics
    let aggregated = state.metrics_client.aggregate_metrics(&endpoints)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e)))?;

    let response = MetricsResponse {
        chain: ChainMetrics {
            height: aggregated.chain_height,
            latest_block_hash: format!("0x{:016x}", aggregated.chain_height), // Simplified
            latest_block_time: chrono::Utc::now(),
            total_transactions: aggregated.total_txs_processed,
            pending_transactions: aggregated.pending_txs as u64,
            average_block_time: 6.0, // Block time target
        },
        network: NetworkMetrics {
            connected_peers: aggregated.total_peers,
            total_peers: aggregated.total_nodes * 10, // Estimate
            bytes_sent: aggregated.bytes_sent,
            bytes_received: aggregated.bytes_received,
            messages_sent: aggregated.messages_sent,
            messages_received: aggregated.messages_received,
        },
        ebsl: EbslMetrics {
            active_miners: aggregated.active_miners,
            banned_miners: aggregated.banned_miners,
            average_trust_score: aggregated.average_trust_score,
            total_slashing_events: aggregated.total_slashing_events,
        },
        system: SystemMetrics {
            uptime_seconds: sys_metrics.uptime_seconds,
            cpu_usage: sys_metrics.cpu_usage,
            memory_usage_mb: sys_metrics.memory_usage_mb,
            total_memory_mb: sys_metrics.total_memory_mb,
            disk_usage_mb: sys_metrics.disk_usage_mb,
            total_disk_mb: sys_metrics.total_disk_mb,
        },
        node_metrics: Some(aggregated.node_metrics),
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

/// Get system-specific metrics (CPU, memory, disk, uptime)
pub async fn system_metrics(
    State(state): State<Arc<AppState>>,
) -> Json<SystemMetrics> {
    let sys = state.system_metrics.collect();
    Json(SystemMetrics {
        uptime_seconds: sys.uptime_seconds,
        cpu_usage: sys.cpu_usage,
        memory_usage_mb: sys.memory_usage_mb,
        total_memory_mb: sys.total_memory_mb,
        disk_usage_mb: sys.disk_usage_mb,
        total_disk_mb: sys.total_disk_mb,
    })
}
