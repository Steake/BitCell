//! Metrics collection and export

pub use super::MetricsRegistry;

/// HTTP server for Prometheus metrics endpoint
pub struct MetricsServer {
    registry: MetricsRegistry,
    port: u16,
}

impl MetricsServer {
    pub fn new(registry: MetricsRegistry, port: u16) -> Self {
        Self { registry, port }
    }
    
    pub fn port(&self) -> u16 {
        self.port
    }
    
    /// Get metrics in Prometheus format
    pub fn get_metrics(&self) -> String {
        self.registry.export_prometheus()
    }
    
    // Future: Actual HTTP server implementation would go here
    // For now, just expose the metrics getter
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_server() {
        let registry = MetricsRegistry::new();
        registry.set_chain_height(100);
        
        let server = MetricsServer::new(registry, 9090);
        assert_eq!(server.port(), 9090);
        
        let metrics = server.get_metrics();
        assert!(metrics.contains("bitcell_chain_height 100"));
    }
}
