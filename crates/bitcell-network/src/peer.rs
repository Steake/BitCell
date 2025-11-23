//! Peer management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub address: String,
    pub reputation: f64,
}

impl PeerInfo {
    pub fn new(id: String, address: String) -> Self {
        Self {
            id,
            address,
            reputation: 1.0,
        }
    }
}

/// Peer reputation tracker
#[derive(Debug, Clone)]
pub struct PeerReputation {
    pub score: f64,
    pub good_messages: u64,
    pub bad_messages: u64,
}

impl PeerReputation {
    pub fn new() -> Self {
        Self {
            score: 1.0,
            good_messages: 0,
            bad_messages: 0,
        }
    }

    pub fn record_good_message(&mut self) {
        self.good_messages += 1;
        self.score = (self.score * 0.9) + 0.1;
        if self.score > 1.0 {
            self.score = 1.0;
        }
    }

    pub fn record_bad_message(&mut self) {
        self.bad_messages += 1;
        self.score = (self.score * 0.9) - 0.2;
        if self.score < 0.0 {
            self.score = 0.0;
        }
    }
}

impl Default for PeerReputation {
    fn default() -> Self {
        Self::new()
    }
}

/// Peer manager
pub struct PeerManager {
    peers: HashMap<String, PeerInfo>,
}

impl PeerManager {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
        }
    }

    pub fn add_peer(&mut self, peer: PeerInfo) {
        self.peers.insert(peer.id.clone(), peer);
    }

    pub fn get_peer(&self, id: &str) -> Option<&PeerInfo> {
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
        let peer = PeerInfo::new("peer1".to_string(), "127.0.0.1:8080".to_string());
        
        pm.add_peer(peer);
        assert_eq!(pm.peer_count(), 1);
        
        let retrieved = pm.get_peer("peer1").unwrap();
        assert_eq!(retrieved.id, "peer1");
    }

    #[test]
    fn test_peer_reputation() {
        let mut rep = PeerReputation::new();
        assert_eq!(rep.score, 1.0);
        
        rep.record_good_message();
        assert!(rep.good_messages == 1);
        
        rep.record_bad_message();
        assert!(rep.bad_messages == 1);
        assert!(rep.score < 1.0);
    }
}
