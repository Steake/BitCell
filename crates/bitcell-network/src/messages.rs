//! Network message types

use bitcell_consensus::{Block, Transaction, GliderCommitment, GliderReveal};
use bitcell_crypto::Hash256;
use serde::{Deserialize, Serialize};

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Block(Block),
    Transaction(Transaction),
    GliderCommit(GliderCommitment),
    GliderReveal(GliderReveal),
    GetBlock(Hash256),
    GetPeers,
}

/// Network message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub message_type: MessageType,
    pub timestamp: u64,
}

impl Message {
    pub fn new(message_type: MessageType) -> Self {
        Self {
            message_type,
            timestamp: 0, // Would use system time
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::new(MessageType::GetPeers);
        assert!(matches!(msg.message_type, MessageType::GetPeers));
    }
}
