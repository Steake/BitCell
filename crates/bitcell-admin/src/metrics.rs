//! Metrics integration

use prometheus_client::registry::Registry;

pub struct MetricsCollector {
    registry: Registry,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            registry: Registry::default(),
        }
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    // TODO: Add actual metrics collection from node
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
