///! Miner node implementation

use crate::{NodeConfig, Result, MetricsRegistry, Blockchain, TransactionPool, NetworkManager};
use bitcell_crypto::SecretKey;
use bitcell_ca::{Glider, GliderPattern};
use bitcell_state::StateManager;
use std::sync::Arc;
use bitcell_consensus::Transaction;

/// Miner node
pub struct MinerNode {
    pub config: NodeConfig,
    pub secret_key: Arc<SecretKey>,
    pub state: StateManager,
    pub glider_strategy: GliderPattern,
    pub metrics: MetricsRegistry,
    pub blockchain: Blockchain,
    pub tx_pool: TransactionPool,
    pub network: NetworkManager,
}

impl MinerNode {
    pub fn new(config: NodeConfig, secret_key: SecretKey) -> Self {
        let secret_key = Arc::new(secret_key);
        let metrics = MetricsRegistry::new();
        let blockchain = Blockchain::new(secret_key.clone(), metrics.clone());
        let network = NetworkManager::new(secret_key.public_key(), metrics.clone());
        
        Self {
            config,
            secret_key,
            state: StateManager::new(),
            glider_strategy: GliderPattern::Standard,
            metrics,
            blockchain,
            tx_pool: TransactionPool::default(),
            network,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        println!("Starting miner node on port {}", self.config.network_port);
        println!("Glider strategy: {:?}", self.glider_strategy);
        
        // Start network layer
        self.network.start(self.config.network_port, self.config.bootstrap_nodes.clone()).await?;
        
        // Enable DHT if configured
        if self.config.enable_dht {
            println!("Enabling DHT with bootstrap nodes: {:?}", self.config.bootstrap_nodes);
            self.network.enable_dht(&self.secret_key, self.config.bootstrap_nodes.clone())?;
        }
        
        // Legacy peer discovery removed in favor of DHT/Bootstrap
        // The network stack now handles connections via NetworkManager::start()
        
        // Initialize metrics with actual state
        self.metrics.set_chain_height(self.blockchain.height());
        self.metrics.set_peer_count(self.network.peer_count());
        self.metrics.set_active_miners(1); // This miner is active

        // Broadcast a dummy transaction for testing P2P
        let pk = self.secret_key.public_key();
        let dummy_sig = self.secret_key.sign(b"dummy");
        let dummy_tx = Transaction {
            nonce: 0,
            from: pk,
            to: pk,
            amount: 0,
            gas_limit: 21000,
            gas_price: 1,
            data: vec![],
            signature: dummy_sig,
        };
        
        let network = self.network.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            println!("Broadcasting dummy transaction...");
            if let Err(e) = network.broadcast_transaction(&dummy_tx).await {
                eprintln!("Failed to broadcast dummy transaction: {}", e);
            }
        });

        Ok(())
    }

    pub async fn broadcast_tx(&self, tx: Transaction) -> Result<()> {
        self.network.broadcast_transaction(&tx).await
    }

    pub async fn start_with_metrics(&mut self, port: u16) -> Result<()> {
        self.start().await?;
        
        let metrics = self.metrics.clone();
        
        // Spawn metrics server
        tokio::spawn(async move {
            let addr = format!("0.0.0.0:{}", port);
            let listener = tokio::net::TcpListener::bind(&addr).await;
            
            match listener {
                Ok(listener) => {
                    loop {
                        if let Ok((mut socket, _)) = listener.accept().await {
                            let metrics = metrics.clone();
                            tokio::spawn(async move {
                                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                                
                                let mut buf = [0; 1024];
                                match socket.read(&mut buf).await {
                                    Ok(0) => return, // Connection closed
                                    Ok(n) => {
                                        let request = String::from_utf8_lossy(&buf[..n]);
                                        println!("Miner received metrics request: {:?}", request.lines().next());
                                        if request.contains("GET /metrics") {
                                            let body = metrics.export_prometheus();
                                            let response = format!(
                                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
                                                body.len(),
                                                body
                                            );
                                            if let Err(e) = socket.write_all(response.as_bytes()).await {
                                                eprintln!("Failed to write metrics response: {}", e);
                                            }
                                            let _ = socket.flush().await;
                                        } else {
                                            let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                                            let _ = socket.write_all(response.as_bytes()).await;
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to read from metrics socket: {}", e);
                                    }
                                }
                            });
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to bind metrics port {}: {}", port, e);
                }
            }
        });
        
        Ok(())
    }

    pub fn generate_glider(&self) -> Glider {
        Glider::new(self.glider_strategy, bitcell_ca::Position::new(256, 512))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_miner_creation() {
        let config = NodeConfig::default();
        let sk = SecretKey::generate();
        let miner = MinerNode::new(config, sk);
        assert_eq!(miner.glider_strategy, GliderPattern::Standard);
    }

    #[test]
    fn test_glider_generation() {
        let config = NodeConfig::default();
        let sk = SecretKey::generate();
        let miner = MinerNode::new(config, sk);
        let glider = miner.generate_glider();
        assert_eq!(glider.pattern, GliderPattern::Standard);
    }
}
