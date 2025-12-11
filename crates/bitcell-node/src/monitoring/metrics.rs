//! Metrics collection and export

use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
    
    /// Get health check status
    pub fn get_health(&self) -> String {
        // Basic health check - node is up if we can respond
        let chain_height = self.registry.get_chain_height();
        let peer_count = self.registry.get_peer_count();
        
        format!(
            r#"{{"status":"ok","chain_height":{},"peer_count":{}}}"#,
            chain_height, peer_count
        )
    }
    
    /// Start HTTP server for metrics and health endpoints
    pub async fn serve(self) -> Result<(), std::io::Error> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        let registry = Arc::new(self.registry);
        
        tracing::info!("Metrics server listening on {}", addr);
        
        loop {
            match listener.accept().await {
                Ok((mut socket, _)) => {
                    let registry_clone = Arc::clone(&registry);
                    
                    tokio::spawn(async move {
                        let mut buffer = [0; 1024];
                        
                        match socket.read(&mut buffer).await {
                            Ok(n) if n > 0 => {
                                let request = String::from_utf8_lossy(&buffer[..n]);
                                
                                let response = if request.starts_with("GET /health") {
                                    // Health check endpoint
                                    let chain_height = registry_clone.get_chain_height();
                                    let peer_count = registry_clone.get_peer_count();
                                    let body = format!(
                                        r#"{{"status":"ok","chain_height":{},"peer_count":{}}}"#,
                                        chain_height, peer_count
                                    );
                                    format!(
                                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                                        body.len(), body
                                    )
                                } else if request.starts_with("GET /metrics") {
                                    // Prometheus metrics endpoint
                                    let body = registry_clone.export_prometheus();
                                    format!(
                                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\n\r\n{}",
                                        body.len(), body
                                    )
                                } else {
                                    // 404 for other paths
                                    let body = "Not Found";
                                    format!(
                                        "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\n\r\n{}",
                                        body.len(), body
                                    )
                                };
                                
                                let _ = socket.write_all(response.as_bytes()).await;
                            }
                            _ => {}
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
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
