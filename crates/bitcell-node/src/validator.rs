//! Validator node implementation

use crate::{NodeConfig, Result, MetricsRegistry, Blockchain, TransactionPool};
use bitcell_consensus::Block;
use bitcell_network::PeerManager;
use bitcell_crypto::SecretKey;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

/// Max transactions per block
const MAX_TXS_PER_BLOCK: usize = 1000;

/// Validator node
pub struct ValidatorNode {
    pub config: NodeConfig,
    pub peers: PeerManager,
    pub metrics: MetricsRegistry,
    pub blockchain: Blockchain,
    pub tx_pool: TransactionPool,
    pub secret_key: Arc<SecretKey>,
    pub tournament_manager: Arc<crate::tournament::TournamentManager>,
    pub network: Arc<crate::network::NetworkManager>,
}

impl ValidatorNode {
    pub fn new(config: NodeConfig) -> crate::Result<Self> {
        let secret_key = if let Some(seed) = &config.key_seed {
            println!("Generating validator key from seed: {}", seed);
            let hash = bitcell_crypto::Hash256::hash(seed.as_bytes());
            Arc::new(SecretKey::from_bytes(hash.as_bytes()).expect("Invalid key seed"))
        } else {
            Arc::new(SecretKey::generate())
        };
        Self::with_key(config, secret_key)
    }

    pub fn with_key(config: NodeConfig, secret_key: Arc<SecretKey>) -> crate::Result<Self> {
        let metrics = MetricsRegistry::new();
        
        // Create blockchain with or without persistent storage based on config
        let blockchain = if let Some(ref data_path) = config.data_dir {
            // Ensure data directory exists
            std::fs::create_dir_all(data_path)
                .map_err(|e| crate::Error::Config(format!("Failed to create data directory: {}", e)))?;
            
            println!("📦 Using persistent storage at: {}", data_path.display());
            Blockchain::with_storage(secret_key.clone(), metrics.clone(), data_path)
                .map_err(|e| crate::Error::Config(format!("Failed to initialize blockchain with storage: {}", e)))?
        } else {
            println!("⚠️  Using in-memory storage (data will not persist)");
            Blockchain::new(secret_key.clone(), metrics.clone())
        };
        
        let tournament_manager = Arc::new(crate::tournament::TournamentManager::new(metrics.clone()));
        let network = Arc::new(crate::network::NetworkManager::new(secret_key.public_key(), metrics.clone()));
        
        Ok(Self {
            config,
            peers: PeerManager::new(),
            metrics,
            blockchain,
            tx_pool: TransactionPool::default(),
            secret_key,
            tournament_manager,
            network,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        println!("Starting validator node on port {}", self.config.network_port);
        // Launch network and metrics server (metrics on network_port + 1)
        let metrics_port = self.config.network_port + 1;
        self.start_with_metrics(metrics_port).await
    }

    pub async fn start_with_metrics(&mut self, port: u16) -> Result<()> {
        println!("Starting validator node on port {}", self.config.network_port);
        
        // Start network layer
        self.network.start(self.config.network_port, self.config.bootstrap_nodes.clone()).await?;
        
        // Enable DHT if configured
        if self.config.enable_dht {
            println!("Enabling DHT with bootstrap nodes: {:?}", self.config.bootstrap_nodes);
            self.network.enable_dht(&self.secret_key, self.config.bootstrap_nodes.clone())?;
        }
        
        // Legacy peer discovery removed in favor of DHT/Bootstrap
        // The network stack now handles connections via NetworkManager::start()
        
        
        let metrics_clone = self.metrics.clone();
        
        // Start metrics server FIRST to ensure it's not blocked by tournament loop
        tokio::spawn(async move {
            let addr = format!("0.0.0.0:{}", port);
            let listener = tokio::net::TcpListener::bind(&addr).await;
            
            match listener {
                Ok(listener) => {
                    loop {
                        if let Ok((mut socket, _)) = listener.accept().await {
                            let metrics = metrics_clone.clone();
                            tokio::spawn(async move {
                                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                                
                                let mut buf = [0; 1024];
                                // Add timeout to read
                                let read_result = tokio::time::timeout(
                                    tokio::time::Duration::from_secs(5),
                                    socket.read(&mut buf)
                                ).await;

                                match read_result {
                                    Ok(Ok(0)) => return, // Connection closed
                                    Ok(Ok(n)) => {
                                        let request = String::from_utf8_lossy(&buf[..n]);
                                        println!("Validator received metrics request: {:?}", request.lines().next());
                                        
                                        if request.contains("GET /metrics") {
                                            println!("Exporting metrics...");
                                            let body = metrics.export_prometheus();
                                            println!("Metrics exported, size: {}", body.len());
                                            
                                            let response = format!(
                                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
                                                body.len(),
                                                body
                                            );
                                            
                                            println!("Writing response...");
                                            // Add timeout to write
                                            if let Err(e) = tokio::time::timeout(
                                                tokio::time::Duration::from_secs(5),
                                                socket.write_all(response.as_bytes())
                                            ).await {
                                                eprintln!("Failed to write metrics response (timeout or error): {:?}", e);
                                            } else {
                                                println!("Response written.");
                                            }
                                            
                                            // Flush with timeout
                                            let _ = tokio::time::timeout(
                                                tokio::time::Duration::from_secs(2),
                                                socket.flush()
                                            ).await;
                                            
                                            // Explicitly shutdown
                                            let _ = socket.shutdown().await;
                                            println!("Socket closed.");
                                        } else {
                                            let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                                            let _ = socket.write_all(response.as_bytes()).await;
                                            let _ = socket.shutdown().await;
                                        }
                                    }
                                    Ok(Err(e)) => {
                                        eprintln!("Failed to read from metrics socket: {}", e);
                                    }
                                    Err(_) => {
                                        eprintln!("Timed out reading from metrics socket");
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
        
        // Start block production loop with tournaments
        let blockchain = Arc::new(self.blockchain.clone());
        let tx_pool = Arc::new(self.tx_pool.clone());
        let metrics = self.metrics.clone();
        let secret_key = self.secret_key.clone();
        let tournament_manager = self.tournament_manager.clone();
        let network = self.network.clone();
        let block_time_secs = self.config.block_time_secs;
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(block_time_secs));
            let mut next_height = 1u64;
            
            loop {
                interval.tick().await;
                
                // For simplified implementation, create a tournament with just this validator
                // In production, this would include all eligible miners from EBSL
                let eligible_miners = vec![secret_key.public_key()];
                let seed = bitcell_crypto::Hash256::hash(&next_height.to_le_bytes());
                
                println!("\n=== Starting tournament for block height {} ===", next_height);
                
                // Run tournament cycle (simplified - in production this would be distributed)
                match crate::tournament::run_tournament_cycle(
                    tournament_manager.clone(),
                    next_height,
                    eligible_miners,
                    seed,
                ).await {
                    Ok(winner) => {
                        println!("Tournament winner selected: {:?}", winner);
                        
                        // Get pending transactions
                        let pending_txs = tx_pool.get_transactions(MAX_TXS_PER_BLOCK);
                        
                        // Get battle proofs from tournament
                        let battle_proofs = tournament_manager.get_battle_proofs().await;
                        
                        // Produce block with tournament winner as proposer
                        match blockchain.produce_block(pending_txs.clone(), battle_proofs, winner) {
                            Ok(block) => {
                                println!("Produced block at height {}", block.header.height);
                                
                                // Add to our own chain
                                if let Err(e) = blockchain.add_block(block.clone()) {
                                    eprintln!("Failed to add own block: {}", e);
                                    // Record negative evidence for failed block
                                    tournament_manager.record_evidence(
                                        winner,
                                        bitcell_ebsl::EvidenceType::InvalidBlock
                                    );
                                    continue;
                                }
                                
                                // Record positive evidence for successful block production
                                tournament_manager.record_evidence(
                                    winner,
                                    bitcell_ebsl::EvidenceType::GoodBlock
                                );
                                
                                // Remove included transactions from pool
                                let tx_hashes: Vec<_> = pending_txs.iter().map(|tx| tx.hash()).collect();
                                tx_pool.remove_transactions(&tx_hashes);
                                
                                // Update metrics
                                metrics.set_chain_height(blockchain.height());
                                metrics.set_pending_txs(tx_pool.pending_count());
                                
                                for _tx in &pending_txs {
                                    metrics.inc_total_txs_processed();
                                }
                                
                                // Increment height BEFORE broadcast to ensure loop continues
                                next_height += 1;
                                
                                // Broadcast block to network
                                if let Err(e) = network.broadcast_block(&block).await {
                                    eprintln!("Failed to broadcast block: {}", e);
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to produce block: {}", e);
                                // Record negative evidence for production failure
                                tournament_manager.record_evidence(
                                    winner,
                                    bitcell_ebsl::EvidenceType::InvalidBlock
                                );
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Tournament failed: {}", e);
                    }
                }
            }
        });
        

        Ok(())
    }

    pub fn validate_block(&self, block: &Block) -> bool {
        self.blockchain.validate_block(block).is_ok()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let config = NodeConfig::default();
        let node = ValidatorNode::new(config).unwrap();
        let state = node.blockchain.state();
        let state_guard = state.read().unwrap();
        assert_eq!(state_guard.accounts.len(), 0);
    }
}
