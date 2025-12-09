//! DHT-based peer discovery and Gossipsub using libp2p
//!
//! Provides decentralized peer discovery and message propagation.
//! 
//! # Features
//! - Kademlia DHT for peer discovery
//! - Gossipsub for efficient message propagation (D=6, 1s heartbeat)
//! - NAT traversal via AutoNAT, Relay, and Hole Punching
//! - Transport encryption via Noise protocol
//! - Compact block propagation for bandwidth efficiency

use libp2p::{
    gossipsub,
    kad::{store::MemoryStore, Behaviour as Kademlia, Config as KademliaConfig, Event as KademliaEvent},
    swarm::{NetworkBehaviour, SwarmEvent},
    identify, noise, tcp, yamux, PeerId, Multiaddr, StreamProtocol,
    identity::{Keypair, ed25519},
    autonat, relay,
    dcutr,
    SwarmBuilder,
};
use futures::prelude::*;
use std::time::Duration;
use std::collections::{hash_map::DefaultHasher, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use tokio::sync::mpsc;
use bitcell_consensus::{Block, Transaction};
use bitcell_crypto::Hash256;

/// Network behaviour combining Kademlia, Identify, Gossipsub, AutoNAT, Relay, and DCUtR
#[derive(NetworkBehaviour)]
struct NodeBehaviour {
    kademlia: Kademlia<MemoryStore>,
    identify: identify::Behaviour,
    gossipsub: gossipsub::Behaviour,
    autonat: autonat::Behaviour,
    relay_client: relay::client::Behaviour,
    dcutr: dcutr::Behaviour,
}

/// Compact block representation for bandwidth-efficient propagation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompactBlock {
    /// Block header (full)
    pub header: bitcell_consensus::BlockHeader,
    /// Short transaction IDs (first 8 bytes of hash)
    pub short_tx_ids: Vec<[u8; 8]>,
    /// Prefilled transactions (coinbase/critical txs)
    pub prefilled_txs: Vec<Transaction>,
    /// Battle proofs (preserved from original block)
    pub battle_proofs: Vec<bitcell_consensus::BattleProof>,
    /// Block signature (preserved from original block)
    pub signature: bitcell_crypto::Signature,
}

impl CompactBlock {
    /// Create a compact block from a full block
    pub fn from_block(block: &Block) -> Self {
        // Always include first transaction (typically coinbase/reward)
        let mut prefilled_txs = vec![];
        let mut short_tx_ids = vec![];
        
        for (idx, tx) in block.transactions.iter().enumerate() {
            if idx == 0 {
                // Include first transaction (reward distribution)
                prefilled_txs.push(tx.clone());
            } else {
                // Use short ID for others
                let tx_hash = tx.hash();
                let mut short_id = [0u8; 8];
                short_id.copy_from_slice(&tx_hash.as_bytes()[..8]);
                short_tx_ids.push(short_id);
            }
        }
        
        Self {
            header: block.header.clone(),
            short_tx_ids,
            prefilled_txs,
            battle_proofs: block.battle_proofs.clone(),
            signature: block.signature,
        }
    }
    
    /// Reconstruct full block from compact block and mempool
    /// 
    /// Note: This uses O(n*m) lookup for simplicity and correctness.
    /// In practice, n (short_tx_ids) is small (~10-100 txs per block)
    /// and m (mempool) is moderate (~1000-10000 txs), making this acceptable.
    /// The transaction order is preserved by iterating short_ids in order.
    /// 
    /// If performance becomes an issue, we could:
    /// - Build a short_id -> tx HashMap from mempool on first use
    /// - Use a Bloom filter for quick negative lookups
    pub fn to_block(&self, mempool: &HashMap<Hash256, Transaction>) -> Option<Block> {
        let mut transactions = self.prefilled_txs.clone();
        
        // Match short IDs to mempool transactions in order
        // This ensures the transaction order matches the original block
        for short_id in &self.short_tx_ids {
            let mut found = false;
            for (hash, tx) in mempool {
                let tx_short_id = &hash.as_bytes()[..8];
                if tx_short_id == short_id {
                    transactions.push(tx.clone());
                    found = true;
                    break;
                }
            }
            if !found {
                // Missing transaction, need to request it
                tracing::warn!("Missing transaction with short ID {:?}", short_id);
                return None;
            }
        }
        
        // Verify we have the expected number of transactions
        let expected_count = self.prefilled_txs.len() + self.short_tx_ids.len();
        if transactions.len() != expected_count {
            tracing::error!("Transaction count mismatch: expected {}, got {}", expected_count, transactions.len());
            return None;
        }
        
        Some(Block {
            header: self.header.clone(),
            transactions,
            battle_proofs: self.battle_proofs.clone(),
            signature: self.signature,
        })
    }
}

/// Commands for the DHT service
enum DhtCommand {
    StartDiscovery,
    BroadcastBlock(Vec<u8>),
    BroadcastCompactBlock(Vec<u8>),
    BroadcastTransaction(Vec<u8>),
    RequestMissingTransactions(Vec<[u8; 8]>),
}

/// DHT manager (client interface)
#[derive(Clone)]
pub struct DhtManager {
    cmd_tx: mpsc::Sender<DhtCommand>,
    local_peer_id: PeerId,
    /// Local transaction mempool for compact block reconstruction
    mempool: std::sync::Arc<parking_lot::RwLock<HashMap<Hash256, Transaction>>>,
}

impl DhtManager {
    /// Create a new DHT manager and spawn the swarm with full NAT traversal support
    pub fn new(
        secret_key: &bitcell_crypto::SecretKey, 
        bootstrap: Vec<String>,
        block_tx: mpsc::Sender<Block>,
        tx_tx: mpsc::Sender<Transaction>,
    ) -> crate::Result<Self> {
        // 1. Create libp2p keypair
        let keypair = Self::bitcell_to_libp2p_keypair(secret_key)?;
        let local_peer_id = PeerId::from(keypair.public());
        tracing::info!("Local Peer ID: {}", local_peer_id);

        // 2. Create transport with Noise encryption
        let mut swarm = SwarmBuilder::with_existing_identity(keypair.clone())
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|e| crate::Error::Network(format!("TCP transport error: {:?}", e)))?
            .with_dns()
            .map_err(|e| crate::Error::Network(format!("DNS transport error: {:?}", e)))?
            .with_relay_client(noise::Config::new, yamux::Config::default)
            .map_err(|e| crate::Error::Network(format!("Relay client error: {:?}", e)))?
            .with_behaviour(|key, relay_client| {
                // Kademlia with optimized config
                let store = MemoryStore::new(key.public().to_peer_id());
                let mut kad_config = KademliaConfig::default();
                kad_config.set_query_timeout(Duration::from_secs(60));
                let kademlia = Kademlia::with_config(key.public().to_peer_id(), store, kad_config);

                // Identify
                let identify = identify::Behaviour::new(identify::Config::new(
                    "/bitcell/1.0.0".to_string(),
                    key.public(),
                ));

                // Gossipsub with production config (D=6, heartbeat=1s)
                let message_id_fn = |message: &gossipsub::Message| {
                    let mut s = DefaultHasher::new();
                    message.data.hash(&mut s);
                    gossipsub::MessageId::from(s.finish().to_string())
                };
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(1))
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .message_id_fn(message_id_fn)
                    .mesh_n(6) // D = 6 as per requirements
                    .mesh_n_low(4)
                    .mesh_n_high(12)
                    .build()
                    .map_err(|msg| std::io::Error::new(std::io::ErrorKind::Other, msg))?;

                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )?;

                // AutoNAT for NAT detection
                let autonat = autonat::Behaviour::new(key.public().to_peer_id(), autonat::Config {
                    retry_interval: Duration::from_secs(90),
                    refresh_interval: Duration::from_secs(180),
                    boot_delay: Duration::from_secs(5),
                    throttle_server_period: Duration::ZERO,
                    only_global_ips: false,
                    ..Default::default()
                });

                // DCUtR for hole punching
                let dcutr = dcutr::Behaviour::new(key.public().to_peer_id());

                Ok(NodeBehaviour {
                    kademlia,
                    identify,
                    gossipsub,
                    autonat,
                    relay_client,
                    dcutr,
                })
            })
            .map_err(|e| crate::Error::Network(format!("Behaviour error: {:?}", e)))?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        // 3. Subscribe to topics
        let block_topic = gossipsub::IdentTopic::new("bitcell-blocks");
        let compact_block_topic = gossipsub::IdentTopic::new("bitcell-compact-blocks");
        let tx_topic = gossipsub::IdentTopic::new("bitcell-transactions");
        
        swarm.behaviour_mut().gossipsub.subscribe(&block_topic)?;
        swarm.behaviour_mut().gossipsub.subscribe(&compact_block_topic)?;
        swarm.behaviour_mut().gossipsub.subscribe(&tx_topic)?;

        // 4. Listen on multiple transports for NAT traversal
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
        swarm.listen_on("/ip6/::/tcp/0".parse()?)?;

        // 5. Add bootstrap nodes
        for addr_str in bootstrap {
            if let Ok(addr) = addr_str.parse::<Multiaddr>() {
                if let Some(peer_id) = Self::extract_peer_id(&addr) {
                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                }
            }
        }

        // 6. Spawn swarm task
        let (cmd_tx, mut cmd_rx) = mpsc::channel(32);
        let mempool = std::sync::Arc::new(parking_lot::RwLock::new(HashMap::new()));
        let mempool_clone = mempool.clone();
        
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    event = swarm.select_next_some() => match event {
                        SwarmEvent::Behaviour(NodeBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                            propagation_source: peer_id,
                            message_id: _,
                            message,
                        })) => {
                            if message.topic == block_topic.hash() {
                                if let Ok(block) = bincode::deserialize::<Block>(&message.data) {
                                    tracing::info!("Received full block via Gossipsub from {}", peer_id);
                                    let _ = block_tx.send(block).await;
                                }
                            } else if message.topic == compact_block_topic.hash() {
                                if let Ok(compact_block) = bincode::deserialize::<CompactBlock>(&message.data) {
                                    tracing::info!("Received compact block via Gossipsub from {}", peer_id);
                                    // Try to reconstruct block from mempool
                                    let block_opt = {
                                        let mempool_guard = mempool_clone.read();
                                        compact_block.to_block(&*mempool_guard)
                                    };
                                    
                                    if let Some(block) = block_opt {
                                        tracing::info!("Successfully reconstructed block from compact representation");
                                        let _ = block_tx.send(block).await;
                                    } else {
                                        tracing::warn!("Missing transactions for compact block, requesting full block");
                                        // TODO: Request missing transactions
                                    }
                                }
                            } else if message.topic == tx_topic.hash() {
                                if let Ok(tx) = bincode::deserialize::<Transaction>(&message.data) {
                                    tracing::info!("Received tx via Gossipsub from {}", peer_id);
                                    // Add to mempool
                                    let tx_hash = tx.hash();
                                    mempool_clone.write().insert(tx_hash, tx.clone());
                                    let _ = tx_tx.send(tx).await;
                                }
                            }
                        }
                        SwarmEvent::Behaviour(NodeBehaviourEvent::Autonat(autonat::Event::StatusChanged { old, new })) => {
                            tracing::info!("NAT status changed from {:?} to {:?}", old, new);
                        }
                        SwarmEvent::Behaviour(NodeBehaviourEvent::Dcutr(event)) => {
                            tracing::info!("DCUtR event: {:?}", event);
                        }
                        SwarmEvent::NewListenAddr { address, .. } => {
                            tracing::info!("DHT listening on {:?}", address);
                        }
                        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            tracing::debug!("Connection established with {}", peer_id);
                        }
                        SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                            tracing::debug!("Connection closed with {}: {:?}", peer_id, cause);
                        }
                        _ => {}
                    },
                    command = cmd_rx.recv() => match command {
                        Some(DhtCommand::StartDiscovery) => {
                            let _ = swarm.behaviour_mut().kademlia.bootstrap();
                        }
                        Some(DhtCommand::BroadcastBlock(data)) => {
                            if let Err(e) = swarm.behaviour_mut().gossipsub.publish(block_topic.clone(), data) {
                                tracing::error!("Failed to publish block via Gossipsub: {:?}", e);
                            }
                        }
                        Some(DhtCommand::BroadcastCompactBlock(data)) => {
                            if let Err(e) = swarm.behaviour_mut().gossipsub.publish(compact_block_topic.clone(), data) {
                                tracing::error!("Failed to publish compact block via Gossipsub: {:?}", e);
                            }
                        }
                        Some(DhtCommand::BroadcastTransaction(data)) => {
                            if let Err(e) = swarm.behaviour_mut().gossipsub.publish(tx_topic.clone(), data) {
                                tracing::error!("Failed to publish transaction via Gossipsub: {:?}", e);
                            }
                        }
                        Some(DhtCommand::RequestMissingTransactions(_short_ids)) => {
                            // TODO: Implement transaction request protocol
                            tracing::warn!("Missing transaction request not yet implemented");
                        }
                        None => break,
                    }
                }
            }
        });

        Ok(Self {
            cmd_tx,
            local_peer_id,
            mempool,
        })
    }
    
    /// Convert BitCell secret key to libp2p keypair
    fn bitcell_to_libp2p_keypair(secret_key: &bitcell_crypto::SecretKey) -> crate::Result<Keypair> {
        let sk_bytes = secret_key.to_bytes();
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&sk_bytes[..32]);
        let secret = ed25519::SecretKey::try_from_bytes(key_bytes)
            .map_err(|e| format!("Invalid secret key: {:?}", e))?;
        Ok(Keypair::from(ed25519::Keypair::from(secret)))
    }
    
    /// Extract peer ID from multiaddr
    fn extract_peer_id(addr: &Multiaddr) -> Option<PeerId> {
        addr.iter().find_map(|protocol| {
            if let libp2p::multiaddr::Protocol::P2p(peer_id) = protocol {
                Some(peer_id)
            } else {
                None
            }
        })
    }
    
    /// Start DHT discovery
    pub async fn start_discovery(&self) -> crate::Result<Vec<crate::dht::PeerInfo>> {
        self.cmd_tx.send(DhtCommand::StartDiscovery).await
            .map_err(|_| crate::Error::from("DHT service channel closed"))?;
        Ok(vec![]) // Return empty for now, discovery happens in background
    }
    
    /// Broadcast a full block
    pub async fn broadcast_block(&self, block: &Block) -> crate::Result<()> {
        let data = bincode::serialize(block).map_err(|e| format!("Serialization error: {}", e))?;
        self.cmd_tx.send(DhtCommand::BroadcastBlock(data)).await
            .map_err(|_| crate::Error::from("DHT service channel closed"))?;
        Ok(())
    }
    
    /// Broadcast a compact block (bandwidth-efficient)
    pub async fn broadcast_compact_block(&self, block: &Block) -> crate::Result<()> {
        let compact_block = CompactBlock::from_block(block);
        let data = bincode::serialize(&compact_block).map_err(|e| format!("Serialization error: {}", e))?;
        
        // Calculate bandwidth savings
        let full_size = bincode::serialize(block).map_err(|e| format!("Serialization error: {}", e))?.len();
        let compact_size = data.len();
        let savings_pct = (1.0 - (compact_size as f64 / full_size as f64)) * 100.0;
        
        tracing::info!(
            "Broadcasting compact block: {} bytes (full: {} bytes, {:.1}% savings)",
            compact_size, full_size, savings_pct
        );
        
        self.cmd_tx.send(DhtCommand::BroadcastCompactBlock(data)).await
            .map_err(|_| crate::Error::from("DHT service channel closed"))?;
        Ok(())
    }
    
    /// Broadcast a transaction
    pub async fn broadcast_transaction(&self, tx: &Transaction) -> crate::Result<()> {
        // Add to local mempool first
        let tx_hash = tx.hash();
        self.mempool.write().insert(tx_hash, tx.clone());
        
        let data = bincode::serialize(tx).map_err(|e| format!("Serialization error: {}", e))?;
        self.cmd_tx.send(DhtCommand::BroadcastTransaction(data)).await
            .map_err(|_| crate::Error::from("DHT service channel closed"))?;
        Ok(())
    }
    
    /// Get local peer ID
    pub fn local_peer_id(&self) -> &PeerId {
        &self.local_peer_id
    }
}

/// Information about a discovered peer
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub addresses: Vec<Multiaddr>,
}
