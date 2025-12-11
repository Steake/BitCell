//! Light client network protocol
//!
//! Defines messages and protocol for light client <-> full node communication.

use bitcell_consensus::BlockHeader;
use bitcell_crypto::Hash256;
use serde::{Deserialize, Serialize};

use crate::{StateProofRequest, StateProof, Checkpoint};

/// Light client protocol messages
#[derive(Clone, Serialize, Deserialize)]
pub enum LightClientMessage {
    /// Request headers in a range
    GetHeaders(GetHeadersRequest),
    
    /// Response with headers
    Headers(Vec<BlockHeader>),
    
    /// Request a state proof
    GetStateProof(StateProofRequest),
    
    /// Response with state proof
    StateProof(StateProof),
    
    /// Request the current chain tip
    GetChainTip,
    
    /// Response with chain tip info
    ChainTip(ChainTipInfo),
    
    /// Request a checkpoint
    GetCheckpoint(u64), // height
    
    /// Response with checkpoint
    Checkpoint(Option<Checkpoint>),
    
    /// Subscribe to new headers
    SubscribeHeaders,
    
    /// Notification of new header
    NewHeader(BlockHeader),
    
    /// Submit a transaction (light client -> full node)
    SubmitTransaction(Vec<u8>),
    
    /// Transaction submission result
    TransactionResult(TransactionResultResponse),
    
    /// Error response
    Error(String),
}

/// Request for headers in a range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHeadersRequest {
    /// Start height (inclusive)
    pub start_height: u64,
    
    /// End height (inclusive)  
    pub end_height: u64,
    
    /// Maximum number of headers to return
    pub max_count: usize,
}

impl GetHeadersRequest {
    /// Create a new get headers request
    pub fn new(start_height: u64, end_height: u64, max_count: usize) -> Self {
        Self {
            start_height,
            end_height,
            max_count,
        }
    }
}

/// Chain tip information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainTipInfo {
    /// Current height
    pub height: u64,
    
    /// Tip block hash
    pub hash: Hash256,
    
    /// Tip block header
    pub header: BlockHeader,
    
    /// Total chain work
    pub total_work: u64,
}

/// Transaction submission result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResultResponse {
    /// Transaction hash
    pub tx_hash: Hash256,
    
    /// Whether accepted into mempool
    pub accepted: bool,
    
    /// Error message if rejected
    pub error: Option<String>,
}

/// Light client protocol handler
///
/// Manages communication between light client and full nodes.
pub struct LightClientProtocol {
    /// Maximum headers per request
    max_headers_per_request: usize,
    
    /// Request timeout (milliseconds)
    request_timeout_ms: u64,
}

impl LightClientProtocol {
    /// Create a new protocol handler
    pub fn new() -> Self {
        Self {
            max_headers_per_request: 500,
            request_timeout_ms: 30_000,
        }
    }
    
    /// Create a get headers request
    pub fn create_get_headers(
        &self,
        start: u64,
        end: u64,
    ) -> GetHeadersRequest {
        if end < start {
            return GetHeadersRequest::new(start, start, 1);
        }
        
        let count = std::cmp::min(
            (end - start + 1) as usize,
            self.max_headers_per_request
        );
        
        GetHeadersRequest::new(start, end, count)
    }
    
    /// Create a state proof request
    pub fn create_state_proof_request(
        &self,
        request: StateProofRequest,
    ) -> LightClientMessage {
        LightClientMessage::GetStateProof(request)
    }
    
    /// Encode a message for transmission
    pub fn encode_message(&self, message: &LightClientMessage) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(message)
    }
    
    /// Decode a received message
    pub fn decode_message(&self, data: &[u8]) -> Result<LightClientMessage, bincode::Error> {
        bincode::deserialize(data)
    }
    
    /// Get request timeout
    pub fn timeout(&self) -> u64 {
        self.request_timeout_ms
    }
}

impl Default for LightClientProtocol {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcell_crypto::SecretKey;
    use crate::proofs::StateProofType;

    fn create_test_header(height: u64) -> BlockHeader {
        BlockHeader {
            height,
            prev_hash: Hash256::zero(),
            tx_root: Hash256::zero(),
            state_root: Hash256::zero(),
            timestamp: height * 10,
            proposer: SecretKey::generate().public_key(),
            vrf_output: [0u8; 32],
            vrf_proof: vec![],
            work: 100,
        }
    }

    #[test]
    fn test_get_headers_request() {
        let request = GetHeadersRequest::new(100, 200, 50);
        
        assert_eq!(request.start_height, 100);
        assert_eq!(request.end_height, 200);
        assert_eq!(request.max_count, 50);
    }

    #[test]
    fn test_protocol_encode_decode() {
        let protocol = LightClientProtocol::new();
        let header = create_test_header(100);
        
        let msg = LightClientMessage::NewHeader(header.clone());
        let encoded = protocol.encode_message(&msg).unwrap();
        let decoded = protocol.decode_message(&encoded).unwrap();
        
        match decoded {
            LightClientMessage::NewHeader(h) => {
                assert_eq!(h.height, header.height);
            },
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_create_get_headers() {
        let protocol = LightClientProtocol::new();
        let request = protocol.create_get_headers(100, 700);
        
        // Should cap at max_headers_per_request
        assert_eq!(request.max_count, 500);
    }

    #[test]
    fn test_state_proof_request_message() {
        let protocol = LightClientProtocol::new();
        let proof_req = StateProofRequest {
            proof_type: StateProofType::AccountBalance,
            block_height: 100,
            key: b"test_account".to_vec(),
            storage_slot: None,
        };
        
        let msg = protocol.create_state_proof_request(proof_req.clone());
        
        match msg {
            LightClientMessage::GetStateProof(req) => {
                assert_eq!(req.block_height, 100);
            },
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_chain_tip_info() {
        let header = create_test_header(500);
        let hash = header.hash();
        
        let tip_info = ChainTipInfo {
            height: 500,
            hash,
            header: header.clone(),
            total_work: 50000,
        };
        
        assert_eq!(tip_info.height, 500);
        assert_eq!(tip_info.hash, hash);
    }
}
