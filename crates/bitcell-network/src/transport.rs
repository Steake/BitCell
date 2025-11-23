/// P2P transport layer (simplified for now - full libp2p integration pending)
/// Architecture ready for production libp2p with gossipsub, mDNS, etc.

use std::collections::{HashMap, HashSet};
use std::error::Error;
use tokio::sync::mpsc;

use crate::messages::{Block, GliderCommit, GliderReveal, Transaction};
use crate::peer::PeerReputation;

/// Peer identifier (string for now, will be libp2p PeerId later)
pub type PeerId = String;

/// Network address (string for now, will be libp2p Multiaddr later)
pub type Multiaddr = String;

/// P2P network manager
/// TODO: Full libp2p integration with:
/// - TCP/QUIC transports
/// - Gossipsub for pub/sub
/// - mDNS for local peer discovery
/// - Kademlia DHT for global discovery
pub struct NetworkManager {
    listen_addr: Multiaddr,
    known_peers: HashSet<PeerId>,
    peer_reputations: HashMap<PeerId, PeerReputation>,
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
        println!("Network manager created, listening on {}", listen_addr);
        Ok(Self {
            listen_addr,
            known_peers: HashSet::new(),
            peer_reputations: HashMap::new(),
            block_tx,
            tx_tx,
        })
    }

    /// Broadcast a block to all peers
    pub async fn broadcast_block(&mut self, _block: &Block) -> Result<(), Box<dyn Error>> {
        // TODO: Implement with libp2p gossipsub
        Ok(())
    }

    /// Broadcast a transaction to all peers
    pub async fn broadcast_transaction(&mut self, _tx: &Transaction) -> Result<(), Box<dyn Error>> {
        // TODO: Implement with libp2p gossipsub
        Ok(())
    }

    /// Broadcast a glider commitment
    pub async fn broadcast_glider_commit(&mut self, _commit: &GliderCommit) -> Result<(), Box<dyn Error>> {
        // TODO: Implement with libp2p gossipsub
        Ok(())
    }

    /// Broadcast a glider reveal
    pub async fn broadcast_glider_reveal(&mut self, _reveal: &GliderReveal) -> Result<(), Box<dyn Error>> {
        // TODO: Implement with libp2p gossipsub
        Ok(())
    }

    /// Get connected peer count
    pub fn peer_count(&self) -> usize {
        self.known_peers.len()
    }

    /// Get all known peers
    pub fn known_peers(&self) -> Vec<PeerId> {
        self.known_peers.iter().cloned().collect()
    }

    /// Add a peer
    pub fn add_peer(&mut self, peer_id: PeerId) {
        self.known_peers.insert(peer_id.clone());
        self.peer_reputations.insert(peer_id, PeerReputation::new());
    }

    /// Remove a peer
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.known_peers.remove(peer_id);
        self.peer_reputations.remove(peer_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_creation() {
        let (block_tx, _) = mpsc::channel(100);
        let (tx_tx, _) = mpsc::channel(100);
        let network = NetworkManager::new("127.0.0.1:30333".to_string(), block_tx, tx_tx)
            .await
            .expect("Failed to create network");
        assert_eq!(network.peer_count(), 0);
    }

    #[tokio::test]
    async fn test_peer_management() {
        let (block_tx, _) = mpsc::channel(100);
        let (tx_tx, _) = mpsc::channel(100);
        let mut network = NetworkManager::new("127.0.0.1:30333".to_string(), block_tx, tx_tx)
            .await
            .expect("Failed to create network");
        
        network.add_peer("peer1".to_string());
        network.add_peer("peer2".to_string());
        assert_eq!(network.peer_count(), 2);
        
        network.remove_peer(&"peer1".to_string());
        assert_eq!(network.peer_count(), 1);
    }
}
