//! Monitoring and metrics collection for BitCell nodes
//!
//! Provides Prometheus-compatible metrics for observability.

pub mod metrics;
pub mod logging;

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Global metrics registry
#[derive(Clone)]
pub struct MetricsRegistry {
    // Chain metrics
    chain_height: Arc<AtomicU64>,
    sync_progress: Arc<AtomicU64>,
    
    // Network metrics
    peer_count: Arc<AtomicUsize>,
    bytes_sent: Arc<AtomicU64>,
    bytes_received: Arc<AtomicU64>,
    
    // Transaction pool metrics
    pending_txs: Arc<AtomicUsize>,
    total_txs_processed: Arc<AtomicU64>,
    
    // Proof metrics
    proofs_generated: Arc<AtomicU64>,
    proofs_verified: Arc<AtomicU64>,
    proof_gen_time_ms: Arc<AtomicU64>,
    proof_verify_time_ms: Arc<AtomicU64>,
    
    // EBSL metrics
    active_miners: Arc<AtomicUsize>,
    banned_miners: Arc<AtomicUsize>,
    avg_trust_score: Arc<AtomicU64>, // Stored as fixed-point * 1000
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            chain_height: Arc::new(AtomicU64::new(0)),
            sync_progress: Arc::new(AtomicU64::new(0)),
            peer_count: Arc::new(AtomicUsize::new(0)),
            bytes_sent: Arc::new(AtomicU64::new(0)),
            bytes_received: Arc::new(AtomicU64::new(0)),
            pending_txs: Arc::new(AtomicUsize::new(0)),
            total_txs_processed: Arc::new(AtomicU64::new(0)),
            proofs_generated: Arc::new(AtomicU64::new(0)),
            proofs_verified: Arc::new(AtomicU64::new(0)),
            proof_gen_time_ms: Arc::new(AtomicU64::new(0)),
            proof_verify_time_ms: Arc::new(AtomicU64::new(0)),
            active_miners: Arc::new(AtomicUsize::new(0)),
            banned_miners: Arc::new(AtomicUsize::new(0)),
            avg_trust_score: Arc::new(AtomicU64::new(0)),
        }
    }
    
    // Chain metrics
    pub fn set_chain_height(&self, height: u64) {
        self.chain_height.store(height, Ordering::Relaxed);
    }
    
    pub fn get_chain_height(&self) -> u64 {
        self.chain_height.load(Ordering::Relaxed)
    }
    
    pub fn set_sync_progress(&self, progress: u64) {
        self.sync_progress.store(progress, Ordering::Relaxed);
    }
    
    pub fn get_sync_progress(&self) -> u64 {
        self.sync_progress.load(Ordering::Relaxed)
    }
    
    // Network metrics
    pub fn set_peer_count(&self, count: usize) {
        self.peer_count.store(count, Ordering::Relaxed);
    }
    
    pub fn get_peer_count(&self) -> usize {
        self.peer_count.load(Ordering::Relaxed)
    }
    
    pub fn add_bytes_sent(&self, bytes: u64) {
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
    }
    
    pub fn add_bytes_received(&self, bytes: u64) {
        self.bytes_received.fetch_add(bytes, Ordering::Relaxed);
    }
    
    pub fn get_bytes_sent(&self) -> u64 {
        self.bytes_sent.load(Ordering::Relaxed)
    }
    
    pub fn get_bytes_received(&self) -> u64 {
        self.bytes_received.load(Ordering::Relaxed)
    }
    
    // Transaction pool metrics
    pub fn set_pending_txs(&self, count: usize) {
        self.pending_txs.store(count, Ordering::Relaxed);
    }
    
    pub fn get_pending_txs(&self) -> usize {
        self.pending_txs.load(Ordering::Relaxed)
    }
    
    pub fn inc_total_txs_processed(&self) {
        self.total_txs_processed.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn get_total_txs_processed(&self) -> u64 {
        self.total_txs_processed.load(Ordering::Relaxed)
    }
    
    // Proof metrics
    pub fn inc_proofs_generated(&self) {
        self.proofs_generated.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn inc_proofs_verified(&self) {
        self.proofs_verified.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_proof_gen_time(&self, time_ms: u64) {
        self.proof_gen_time_ms.store(time_ms, Ordering::Relaxed);
    }
    
    pub fn record_proof_verify_time(&self, time_ms: u64) {
        self.proof_verify_time_ms.store(time_ms, Ordering::Relaxed);
    }
    
    pub fn get_proofs_generated(&self) -> u64 {
        self.proofs_generated.load(Ordering::Relaxed)
    }
    
    pub fn get_proofs_verified(&self) -> u64 {
        self.proofs_verified.load(Ordering::Relaxed)
    }
    
    // EBSL metrics
    pub fn set_active_miners(&self, count: usize) {
        self.active_miners.store(count, Ordering::Relaxed);
    }
    
    pub fn set_banned_miners(&self, count: usize) {
        self.banned_miners.store(count, Ordering::Relaxed);
    }
    
    pub fn get_active_miners(&self) -> usize {
        self.active_miners.load(Ordering::Relaxed)
    }
    
    pub fn get_banned_miners(&self) -> usize {
        self.banned_miners.load(Ordering::Relaxed)
    }
    
    /// Export metrics in Prometheus format
    pub fn export_prometheus(&self) -> String {
        format!(
            "# HELP bitcell_chain_height Current blockchain height\n\
             # TYPE bitcell_chain_height gauge\n\
             bitcell_chain_height {}\n\
             \n\
             # HELP bitcell_sync_progress Sync progress percentage (0-100)\n\
             # TYPE bitcell_sync_progress gauge\n\
             bitcell_sync_progress {}\n\
             \n\
             # HELP bitcell_peer_count Number of connected peers\n\
             # TYPE bitcell_peer_count gauge\n\
             bitcell_peer_count {}\n\
             \n\
             # HELP bitcell_bytes_sent_total Total bytes sent\n\
             # TYPE bitcell_bytes_sent_total counter\n\
             bitcell_bytes_sent_total {}\n\
             \n\
             # HELP bitcell_bytes_received_total Total bytes received\n\
             # TYPE bitcell_bytes_received_total counter\n\
             bitcell_bytes_received_total {}\n\
             \n\
             # HELP bitcell_pending_txs Number of pending transactions\n\
             # TYPE bitcell_pending_txs gauge\n\
             bitcell_pending_txs {}\n\
             \n\
             # HELP bitcell_txs_processed_total Total transactions processed\n\
             # TYPE bitcell_txs_processed_total counter\n\
             bitcell_txs_processed_total {}\n\
             \n\
             # HELP bitcell_proofs_generated_total Total proofs generated\n\
             # TYPE bitcell_proofs_generated_total counter\n\
             bitcell_proofs_generated_total {}\n\
             \n\
             # HELP bitcell_proofs_verified_total Total proofs verified\n\
             # TYPE bitcell_proofs_verified_total counter\n\
             bitcell_proofs_verified_total {}\n\
             \n\
             # HELP bitcell_active_miners Number of active eligible miners\n\
             # TYPE bitcell_active_miners gauge\n\
             bitcell_active_miners {}\n\
             \n\
             # HELP bitcell_banned_miners Number of banned miners\n\
             # TYPE bitcell_banned_miners gauge\n\
             bitcell_banned_miners {}\n",
            self.get_chain_height(),
            self.get_sync_progress(),
            self.get_peer_count(),
            self.get_bytes_sent(),
            self.get_bytes_received(),
            self.get_pending_txs(),
            self.get_total_txs_processed(),
            self.get_proofs_generated(),
            self.get_proofs_verified(),
            self.get_active_miners(),
            self.get_banned_miners(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_registry() {
        let metrics = MetricsRegistry::new();
        
        metrics.set_chain_height(100);
        assert_eq!(metrics.get_chain_height(), 100);
        
        metrics.set_peer_count(5);
        assert_eq!(metrics.get_peer_count(), 5);
        
        metrics.add_bytes_sent(1000);
        metrics.add_bytes_sent(500);
        assert_eq!(metrics.get_bytes_sent(), 1500);
        
        metrics.inc_proofs_generated();
        metrics.inc_proofs_generated();
        assert_eq!(metrics.get_proofs_generated(), 2);
    }

    #[test]
    fn test_prometheus_export() {
        let metrics = MetricsRegistry::new();
        metrics.set_chain_height(42);
        metrics.set_peer_count(3);
        
        let export = metrics.export_prometheus();
        assert!(export.contains("bitcell_chain_height 42"));
        assert!(export.contains("bitcell_peer_count 3"));
    }
}
