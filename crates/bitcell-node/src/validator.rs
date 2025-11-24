//! Validator node implementation

use crate::{NodeConfig, Result, MetricsRegistry, Blockchain, TransactionPool};
use bitcell_consensus::Block;
use bitcell_state::StateManager;
use bitcell_network::PeerManager;
use bitcell_crypto::SecretKey;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

/// Block production interval (10 seconds for testing, TODO: make this 10 minutes in production)
const BLOCK_TIME_SECS: u64 = 10;

/// Max transactions per block
const MAX_TXS_PER_BLOCK: usize = 1000;

/// Validator node
pub struct ValidatorNode {
    pub config: NodeConfig,
    pub state: StateManager,
    pub peers: PeerManager,
    pub metrics: MetricsRegistry,
    pub blockchain: Blockchain,
    pub tx_pool: TransactionPool,
    pub secret_key: Arc<SecretKey>,
    pub tournament_manager: Arc<crate::tournament::TournamentManager>,
    pub network: Arc<crate::network::NetworkManager>,
}

impl ValidatorNode {
    pub fn new(config: NodeConfig) -> Self {
        let secret_key = Arc::new(SecretKey::generate());
        let metrics = MetricsRegistry::new();
        let blockchain = Blockchain::new(secret_key.clone(), metrics.clone());
        let tournament_manager = Arc::new(crate::tournament::TournamentManager::new(metrics.clone()));
        let network = Arc::new(crate::network::NetworkManager::new(secret_key.public_key(), metrics.clone()));
        
        Self {
            config,
            state: StateManager::new(),
            peers: PeerManager::new(),
            metrics,
            blockchain,
            tx_pool: TransactionPool::default(),
            secret_key,
            tournament_manager,
            network,
        }
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
        self.network.start(self.config.network_port).await?;
        
        // Try to connect to other nodes (simple peer discovery for local testing)
        // In production, this would use mDNS or a bootstrap server
        let network = self.network.clone();
        let my_port = self.config.network_port;
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            
            // Try to connect to nearby ports (other nodes)
            for base_port in [19000, 19100, 19200] {
                for offset in 0..10 {
                    let port = base_port + offset * 2;
                    if port != my_port {
                        let addr = format!("127.0.0.1:{}", port);
                        let _ = network.connect_to_peer(&addr).await;
                    }
                }
            }
        });
        
        // Initialize real metrics with actual initial state
        self.metrics.set_chain_height(self.blockchain.height());
        self.metrics.set_peer_count(self.network.peer_count());
        
        // Start block production loop with tournaments
        let blockchain = Arc::new(self.blockchain.clone());
        let tx_pool = Arc::new(self.tx_pool.clone());
        let metrics = self.metrics.clone();
        let secret_key = self.secret_key.clone();
        let tournament_manager = self.tournament_manager.clone();
        let network = self.network.clone();
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(BLOCK_TIME_SECS));
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
                        let battle_proofs = tournament_manager.get_battle_proofs();
                        
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
                                
                                // Broadcast block to network
                                if let Err(e) = network.broadcast_block(&block).await {
                                    eprintln!("Failed to broadcast block: {}", e);
                                }
                                
                                next_height += 1;
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
                                        println!("Validator received metrics request: {:?}", request.lines().next());
                                        if request.contains("GET /metrics") {
                                            let body = metrics.export_prometheus();
                                            let response = format!(
                                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                                body.len(),
                                                body
                                            );
                                            let _ = socket.write_all(response.as_bytes()).await;
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
        let node = ValidatorNode::new(config);
        assert_eq!(node.state.accounts.len(), 0);
    }
}
