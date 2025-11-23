/// libp2p transport integration for P2P networking
/// Provides full networking layer with gossipsub and peer discovery

use libp2p::{
    core::upgrade,
    gossipsub, identity, mdns, noise,
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm, Transport,
};
use std::collections::HashSet;
use std::error::Error;
use tokio::sync::mpsc;

use crate::messages::{Block, GliderCommit, GliderReveal, Transaction};
use crate::peer::{PeerInfo, PeerReputation};

/// Network behavior combining gossipsub and mDNS
#[derive(NetworkBehaviour)]
pub struct BitCellBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

/// P2P network manager
pub struct NetworkManager {
    swarm: Swarm<BitCellBehaviour>,
    known_peers: HashSet<PeerId>,
    peer_reputations: std::collections::HashMap<PeerId, PeerReputation>,
    block_tx: mpsc::Sender<Block>,
    tx_tx: mpsc::Sender<Transaction>,
}

impl NetworkManager {
    /// Create a new network manager
    pub async fn new(
        listen_addr: Multiaddr,
        block_tx: mpsc::Sender<Block>,
        tx_tx: mpsc::Sender<Transaction>,
    ) -> Result<Self, Box<dyn Error>> {
        // Generate identity
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        println!("Local peer id: {local_peer_id}");

        // Create transport
        let transport = tcp::tokio::Transport::default()
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::Config::new(&local_key).unwrap())
            .multiplex(yamux::Config::default())
            .boxed();

        // Create gossipsub
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(std::time::Duration::from_secs(1))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()
            .expect("Valid gossipsub config");

        let mut gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )
        .expect("Valid gossipsub behaviour");

        // Subscribe to topics
        gossipsub.subscribe(&gossipsub::IdentTopic::new("blocks"))?;
        gossipsub.subscribe(&gossipsub::IdentTopic::new("transactions"))?;
        gossipsub.subscribe(&gossipsub::IdentTopic::new("commits"))?;
        gossipsub.subscribe(&gossipsub::IdentTopic::new("reveals"))?;

        // Create mDNS
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?;

        // Create swarm
        let behaviour = BitCellBehaviour { gossipsub, mdns };
        let mut swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, local_peer_id).build();

        // Listen on address
        swarm.listen_on(listen_addr)?;

        Ok(Self {
            swarm,
            known_peers: HashSet::new(),
            peer_reputations: std::collections::HashMap::new(),
            block_tx,
            tx_tx,
        })
    }

    /// Broadcast a block to the network
    pub fn broadcast_block(&mut self, block: &Block) -> Result<(), Box<dyn Error>> {
        let topic = gossipsub::IdentTopic::new("blocks");
        let data = bincode::serialize(block)?;
        self.swarm.behaviour_mut().gossipsub.publish(topic, data)?;
        Ok(())
    }

    /// Broadcast a transaction to the network
    pub fn broadcast_transaction(&mut self, tx: &Transaction) -> Result<(), Box<dyn Error>> {
        let topic = gossipsub::IdentTopic::new("transactions");
        let data = bincode::serialize(tx)?;
        self.swarm.behaviour_mut().gossipsub.publish(topic, data)?;
        Ok(())
    }

    /// Broadcast a glider commit
    pub fn broadcast_commit(&mut self, commit: &GliderCommit) -> Result<(), Box<dyn Error>> {
        let topic = gossipsub::IdentTopic::new("commits");
        let data = bincode::serialize(commit)?;
        self.swarm.behaviour_mut().gossipsub.publish(topic, data)?;
        Ok(())
    }

    /// Broadcast a glider reveal
    pub fn broadcast_reveal(&mut self, reveal: &GliderReveal) -> Result<(), Box<dyn Error>> {
        let topic = gossipsub::IdentTopic::new("reveals");
        let data = bincode::serialize(reveal)?;
        self.swarm.behaviour_mut().gossipsub.publish(topic, data)?;
        Ok(())
    }

    /// Run the network event loop
    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::Behaviour(BitCellBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, addr) in list {
                        println!("Discovered peer: {peer_id} at {addr}");
                        self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        self.known_peers.insert(peer_id);
                        self.peer_reputations.insert(peer_id, PeerReputation::new());
                    }
                }
                SwarmEvent::Behaviour(BitCellBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, addr) in list {
                        println!("Peer expired: {peer_id} at {addr}");
                        self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        self.known_peers.remove(&peer_id);
                    }
                }
                SwarmEvent::Behaviour(BitCellBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source,
                    message,
                    ..
                })) => {
                    self.handle_gossipsub_message(propagation_source, message).await?;
                }
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Listening on {address}");
                }
                _ => {}
            }
        }
    }

    /// Handle incoming gossipsub messages
    async fn handle_gossipsub_message(
        &mut self,
        _source: PeerId,
        message: gossipsub::Message,
    ) -> Result<(), Box<dyn Error>> {
        let topic = message.topic.as_str();
        
        match topic {
            "blocks" => {
                if let Ok(block) = bincode::deserialize::<Block>(&message.data) {
                    self.block_tx.send(block).await?;
                }
            }
            "transactions" => {
                if let Ok(tx) = bincode::deserialize::<Transaction>(&message.data) {
                    self.tx_tx.send(tx).await?;
                }
            }
            "commits" | "reveals" => {
                // Handle tournament messages (to be implemented)
            }
            _ => {}
        }
        
        Ok(())
    }

    /// Get peer count
    pub fn peer_count(&self) -> usize {
        self.known_peers.len()
    }

    /// Get peer info
    pub fn get_peers(&self) -> Vec<PeerInfo> {
        self.known_peers
            .iter()
            .map(|peer_id| PeerInfo {
                peer_id: peer_id.to_string(),
                reputation: self.peer_reputations.get(peer_id).cloned().unwrap_or_default(),
                connected: true,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_manager_creation() {
        let (block_tx, _) = mpsc::channel(100);
        let (tx_tx, _) = mpsc::channel(100);
        
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
        let result = NetworkManager::new(addr, block_tx, tx_tx).await;
        
        assert!(result.is_ok());
    }
}
