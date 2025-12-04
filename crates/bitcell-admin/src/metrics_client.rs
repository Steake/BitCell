//! Metrics client for fetching real data from running nodes

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub node_id: String,
    pub endpoint: String,
    pub chain_height: u64,
    pub sync_progress: u64,
    pub peer_count: usize,
    pub dht_peer_count: usize,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub pending_txs: usize,
    pub total_txs_processed: u64,
    pub proofs_generated: u64,
    pub proofs_verified: u64,
    pub active_miners: usize,
    pub banned_miners: usize,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct MetricsClient {
    client: reqwest::Client,
}

impl MetricsClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("Failed to build HTTP client for metrics"),
        }
    }

    /// Fetch metrics from a node's Prometheus endpoint
    pub async fn fetch_node_metrics(&self, node_id: &str, endpoint: &str) -> Result<NodeMetrics, String> {
        let url = if endpoint.ends_with("/metrics") {
            endpoint.to_string()
        } else {
            format!("{}/metrics", endpoint)
        };

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to node {}: {}", node_id, e))?;

        if !response.status().is_success() {
            return Err(format!("Node {} returned status: {}", node_id, response.status()));
        }

        let text = response.text().await
            .map_err(|e| format!("Failed to read response from node {}: {}", node_id, e))?;

        self.parse_prometheus_metrics(node_id, endpoint, &text)
    }

    /// Parse Prometheus metrics format
    /// NOTE: This is a basic parser that only handles simple "metric_name value" format.
    /// It does NOT support metric labels (e.g., metric{label="value"}).
    /// For production use, consider using a proper Prometheus parsing library.
    fn parse_prometheus_metrics(&self, node_id: &str, endpoint: &str, text: &str) -> Result<NodeMetrics, String> {
        let mut metrics = HashMap::new();

        for line in text.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let key = parts[0];
                if let Ok(value) = parts[1].parse::<f64>() {
                    metrics.insert(key.to_string(), value);
                }
            }
        }

        Ok(NodeMetrics {
            node_id: node_id.to_string(),
            endpoint: endpoint.to_string(),
            chain_height: metrics.get("bitcell_chain_height").copied().unwrap_or(0.0) as u64,
            sync_progress: metrics.get("bitcell_sync_progress").copied().unwrap_or(0.0) as u64,
            peer_count: metrics.get("bitcell_peer_count").copied().unwrap_or(0.0) as usize,
            dht_peer_count: metrics.get("bitcell_dht_peer_count").copied().unwrap_or(0.0) as usize,
            bytes_sent: metrics.get("bitcell_bytes_sent_total").copied().unwrap_or(0.0) as u64,
            bytes_received: metrics.get("bitcell_bytes_received_total").copied().unwrap_or(0.0) as u64,
            pending_txs: metrics.get("bitcell_pending_txs").copied().unwrap_or(0.0) as usize,
            total_txs_processed: metrics.get("bitcell_txs_processed_total").copied().unwrap_or(0.0) as u64,
            proofs_generated: metrics.get("bitcell_proofs_generated_total").copied().unwrap_or(0.0) as u64,
            proofs_verified: metrics.get("bitcell_proofs_verified_total").copied().unwrap_or(0.0) as u64,
            active_miners: metrics.get("bitcell_active_miners").copied().unwrap_or(0.0) as usize,
            banned_miners: metrics.get("bitcell_banned_miners").copied().unwrap_or(0.0) as usize,
            last_updated: chrono::Utc::now(),
        })
    }

    /// Aggregate metrics from multiple nodes
    pub async fn aggregate_metrics(&self, endpoints: &[(String, String)]) -> Result<AggregatedMetrics, String> {
        if endpoints.is_empty() {
            return Err("No nodes configured. Please deploy nodes first.".to_string());
        }

        let mut node_metrics = Vec::new();
        let mut errors = Vec::new();

        for (node_id, endpoint) in endpoints {
            match self.fetch_node_metrics(node_id, endpoint).await {
                Ok(metrics) => node_metrics.push(metrics),
                Err(e) => {
                    errors.push(format!("{}: {}", node_id, e));
                    if e.contains("Connection refused") || e.contains("operation timed out") {
                        tracing::debug!("Failed to fetch metrics from {}: {}", node_id, e);
                    } else {
                        tracing::warn!("Failed to fetch metrics from {}: {}", node_id, e);
                    }
                }
            }
        }

        if node_metrics.is_empty() {
            return Err(format!(
                "Failed to fetch metrics from any node. Errors: {}",
                errors.join("; ")
            ));
        }

        // Aggregate across all responding nodes
        let chain_height = node_metrics.iter().map(|m| m.chain_height).max().unwrap_or(0);
        let total_peer_count: usize = node_metrics.iter().map(|m| m.peer_count).sum();
        let total_bytes_sent: u64 = node_metrics.iter().map(|m| m.bytes_sent).sum();
        let total_bytes_received: u64 = node_metrics.iter().map(|m| m.bytes_received).sum();
        let total_pending_txs: usize = node_metrics.iter().map(|m| m.pending_txs).sum();
        let total_txs_processed: u64 = node_metrics.iter().map(|m| m.total_txs_processed).sum();
        let total_active_miners: usize = node_metrics.iter().map(|m| m.active_miners).max().unwrap_or(0);
        let total_banned_miners: usize = node_metrics.iter().map(|m| m.banned_miners).max().unwrap_or(0);

        Ok(AggregatedMetrics {
            chain_height,
            total_nodes: node_metrics.len(),
            online_nodes: node_metrics.len(),
            total_peers: total_peer_count,
            bytes_sent: total_bytes_sent,
            bytes_received: total_bytes_received,
            pending_txs: total_pending_txs,
            total_txs_processed,
            active_miners: total_active_miners,
            banned_miners: total_banned_miners,
            node_metrics,
            errors,
        })
    }
}

impl Default for MetricsClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize)]
pub struct AggregatedMetrics {
    pub chain_height: u64,
    pub total_nodes: usize,
    pub online_nodes: usize,
    pub total_peers: usize,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub pending_txs: usize,
    pub total_txs_processed: u64,
    pub active_miners: usize,
    pub banned_miners: usize,
    pub node_metrics: Vec<NodeMetrics>,
    pub errors: Vec<String>,
}
