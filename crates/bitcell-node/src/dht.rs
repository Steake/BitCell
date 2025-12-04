//! DHT-based peer discovery and Gossipsub using libp2p
//!
//! Provides decentralized peer discovery and message propagation.

use libp2p::{
    gossipsub,
    kad::{store::MemoryStore, Behaviour as Kademlia, Config as KademliaConfig, Event as KademliaEvent},
    swarm::{NetworkBehaviour, SwarmEvent},
    identify, noise, tcp, yamux, PeerId, Multiaddr, StreamProtocol,
    identity::{Keypair, ed25519},
    SwarmBuilder,
};
use futures::prelude::*;
use std::time::Duration;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tokio::sync::mpsc;
use bitcell_consensus::{Block, Transaction};

/// Network behaviour combining Kademlia, Identify, and Gossipsub
#[derive(NetworkBehaviour)]
struct NodeBehaviour {
    kademlia: Kademlia<MemoryStore>,
    identify: identify::Behaviour,
    gossipsub: gossipsub::Behaviour,
}

/// Commands for the DHT service
enum DhtCommand {
    StartDiscovery,
    BroadcastBlock(Vec<u8>),
    BroadcastTransaction(Vec<u8>),
}

/// DHT manager (client interface)
#[derive(Clone)]
pub struct DhtManager {
    cmd_tx: mpsc::Sender<DhtCommand>,
    local_peer_id: PeerId,
}

impl DhtManager {
    /// Create a new DHT manager and spawn the swarm
    pub fn new(
        secret_key: &bitcell_crypto::SecretKey, 
        bootstrap: Vec<String>,
        block_tx: mpsc::Sender<Block>,
        tx_tx: mpsc::Sender<Transaction>,
    ) -> crate::Result<Self> {
        // 1. Create libp2p keypair
        let keypair = Self::bitcell_to_libp2p_keypair(secret_key)?;
        let local_peer_id = PeerId::from(keypair.public());
        println!("Local Peer ID: {}", local_peer_id);

        // 2. Create transport
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
            .with_behaviour(|key| {
                // Kademlia
                let store = MemoryStore::new(key.public().to_peer_id());
                let kad_config = KademliaConfig::default();
                let kademlia = Kademlia::with_config(key.public().to_peer_id(), store, kad_config);

                // Identify
                let identify = identify::Behaviour::new(identify::Config::new(
                    "/bitcell/1.0.0".to_string(),
                    key.public(),
                ));

                // Gossipsub
                let message_id_fn = |message: &gossipsub::Message| {
                    let mut s = DefaultHasher::new();
                    message.data.hash(&mut s);
                    gossipsub::MessageId::from(s.finish().to_string())
                };
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(1))
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .message_id_fn(message_id_fn)
                    .build()
                    .map_err(|msg| std::io::Error::new(std::io::ErrorKind::Other, msg))?;

                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )?;

                Ok(NodeBehaviour {
                    kademlia,
                    identify,
                    gossipsub,
                })
            })
            .map_err(|e| crate::Error::Network(format!("Behaviour error: {:?}", e)))?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        // 3. Subscribe to topics
        let block_topic = gossipsub::IdentTopic::new("bitcell-blocks");
        let tx_topic = gossipsub::IdentTopic::new("bitcell-transactions");
        
        swarm.behaviour_mut().gossipsub.subscribe(&block_topic)?;
        swarm.behaviour_mut().gossipsub.subscribe(&tx_topic)?;

        // 4. Listen on a random port (or fixed if configured)
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

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
                                    println!("Received block via Gossipsub from {}", peer_id);
                                    let _ = block_tx.send(block).await;
                                }
                            } else if message.topic == tx_topic.hash() {
                                if let Ok(tx) = bincode::deserialize::<Transaction>(&message.data) {
                                    println!("Received tx via Gossipsub from {}", peer_id);
                                    let _ = tx_tx.send(tx).await;
                                }
                            }
                        }
                        SwarmEvent::NewListenAddr { address, .. } => {
                            println!("DHT listening on {:?}", address);
                        }
                        _ => {}
                    },
                    command = cmd_rx.recv() => match command {
                        Some(DhtCommand::StartDiscovery) => {
                            let _ = swarm.behaviour_mut().kademlia.bootstrap();
                        }
                        Some(DhtCommand::BroadcastBlock(data)) => {
                            if let Err(e) = swarm.behaviour_mut().gossipsub.publish(block_topic.clone(), data) {
                                eprintln!("Failed to publish block: {:?}", e);
                            }
                        }
                        Some(DhtCommand::BroadcastTransaction(data)) => {
                            if let Err(e) = swarm.behaviour_mut().gossipsub.publish(tx_topic.clone(), data) {
                                eprintln!("Failed to publish tx: {:?}", e);
                            }
                        }
                        None => break,
                    }
                }
            }
        });

        Ok(Self {
            cmd_tx,
            local_peer_id,
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
    
    pub async fn start_discovery(&self) -> crate::Result<Vec<crate::dht::PeerInfo>> {
        self.cmd_tx.send(DhtCommand::StartDiscovery).await
            .map_err(|_| crate::Error::from("DHT service channel closed"))?;
        Ok(vec![]) // Return empty for now, discovery happens in background
    }
    
    pub async fn broadcast_block(&self, block: &Block) -> crate::Result<()> {
        let data = bincode::serialize(block).map_err(|e| format!("Serialization error: {}", e))?;
        self.cmd_tx.send(DhtCommand::BroadcastBlock(data)).await
            .map_err(|_| crate::Error::from("DHT service channel closed"))?;
        Ok(())
    }
    
    pub async fn broadcast_transaction(&self, tx: &Transaction) -> crate::Result<()> {
        let data = bincode::serialize(tx).map_err(|e| format!("Serialization error: {}", e))?;
        self.cmd_tx.send(DhtCommand::BroadcastTransaction(data)).await
            .map_err(|_| crate::Error::from("DHT service channel closed"))?;
        Ok(())
    }
}

/// Information about a discovered peer
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub addresses: Vec<Multiaddr>,
}
