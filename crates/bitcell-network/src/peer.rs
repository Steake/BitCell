//! Peer management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,
    pub address: String,
    pub reputation: f64,
}

impl Peer {
    pub fn new(id: String, address: String) -> Self {
        Self {
            id,
            address,
            reputation: 1.0,
        }
    }
}

/// Peer manager
pub struct PeerManager {
    peers: HashMap<String, Peer>,
}

impl PeerManager {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
        }
    }

    pub fn add_peer(&mut self, peer: Peer) {
        self.peers.insert(peer.id.clone(), peer);
    }

    pub fn get_peer(&self, id: &str) -> Option<&Peer> {
        self.peers.get(id)
    }

    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }
}

impl Default for PeerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_manager() {
        let mut pm = PeerManager::new();
        let peer = Peer::new("peer1".to_string(), "127.0.0.1:8080".to_string());
        
        pm.add_peer(peer);
        assert_eq!(pm.peer_count(), 1);
        
        let retrieved = pm.get_peer("peer1").unwrap();
        assert_eq!(retrieved.id, "peer1");
    }
}
