//! Governance RPC endpoints

use axum::{
    extract::{State, Json},
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use parking_lot::RwLock;
use bitcell_governance::*;

use super::{RpcState, JsonRpcError};

/// Governance RPC state (shared across requests)
pub struct GovernanceRpcState {
    pub manager: Arc<RwLock<GovernanceManager>>,
}

impl GovernanceRpcState {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(RwLock::new(GovernanceManager::new())),
        }
    }
    
    pub fn with_config(config: GovernanceConfig, guardians: GuardianSet) -> Self {
        Self {
            manager: Arc::new(RwLock::new(GovernanceManager::with_config(config, guardians))),
        }
    }
}

/// Submit a governance proposal
pub async fn submit_proposal(
    state: &RpcState,
    gov_state: &GovernanceRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    #[derive(Deserialize)]
    struct SubmitProposalParams {
        proposer: String,
        proposal_type: ProposalTypeJson,
        description: String,
    }
    
    #[derive(Deserialize)]
    #[serde(tag = "type")]
    enum ProposalTypeJson {
        ParameterChange { parameter: String, new_value: String },
        TreasurySpending { recipient: String, amount: u64, reason: String },
        ProtocolUpgrade { version: String, code_hash: String, description: String },
    }
    
    let params: SubmitProposalParams = serde_json::from_value(params.ok_or(JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?)
    .map_err(|e| JsonRpcError {
        code: -32602,
        message: format!("Invalid params: {}", e),
        data: None,
    })?;
    
    // Parse proposer address
    let proposer_bytes = hex::decode(&params.proposer.trim_start_matches("0x"))
        .map_err(|e| JsonRpcError {
            code: -32602,
            message: format!("Invalid proposer address: {}", e),
            data: None,
        })?;
    
    if proposer_bytes.len() != 33 {
        return Err(JsonRpcError {
            code: -32602,
            message: "Proposer address must be 33 bytes".to_string(),
            data: None,
        });
    }
    
    let mut proposer = [0u8; 33];
    proposer.copy_from_slice(&proposer_bytes);
    
    // Convert proposal type
    let proposal_type = match params.proposal_type {
        ProposalTypeJson::ParameterChange { parameter, new_value } => {
            ProposalType::ParameterChange { parameter, new_value }
        }
        ProposalTypeJson::TreasurySpending { recipient, amount, reason } => {
            let recipient_bytes = hex::decode(&recipient.trim_start_matches("0x"))
                .map_err(|e| JsonRpcError {
                    code: -32602,
                    message: format!("Invalid recipient address: {}", e),
                    data: None,
                })?;
            
            if recipient_bytes.len() != 33 {
                return Err(JsonRpcError {
                    code: -32602,
                    message: "Recipient address must be 33 bytes".to_string(),
                    data: None,
                });
            }
            
            let mut recipient_addr = [0u8; 33];
            recipient_addr.copy_from_slice(&recipient_bytes);
            
            ProposalType::TreasurySpending {
                recipient: recipient_addr,
                amount,
                reason,
            }
        }
        ProposalTypeJson::ProtocolUpgrade { version, code_hash, description } => {
            let hash_bytes = hex::decode(&code_hash.trim_start_matches("0x"))
                .map_err(|e| JsonRpcError {
                    code: -32602,
                    message: format!("Invalid code hash: {}", e),
                    data: None,
                })?;
            
            if hash_bytes.len() != 32 {
                return Err(JsonRpcError {
                    code: -32602,
                    message: "Code hash must be 32 bytes".to_string(),
                    data: None,
                });
            }
            
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&hash_bytes);
            
            ProposalType::ProtocolUpgrade {
                version,
                code_hash: hash,
                description,
            }
        }
    };
    
    // Get current timestamp (in production, use actual blockchain time)
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Submit proposal
    let mut gov = gov_state.manager.write();
    let proposal_id = gov.submit_proposal(proposer, proposal_type, params.description, timestamp)
        .map_err(|e| JsonRpcError {
            code: -32000,
            message: format!("Failed to submit proposal: {}", e),
            data: None,
        })?;
    
    Ok(json!({
        "proposal_id": format!("0x{}", hex::encode(&proposal_id.0)),
        "status": "submitted"
    }))
}

/// Vote on a proposal
pub async fn vote_on_proposal(
    state: &RpcState,
    gov_state: &GovernanceRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    #[derive(Deserialize)]
    struct VoteParams {
        proposal_id: String,
        voter: String,
        support: bool,
        voting_power: u64,
    }
    
    let params: VoteParams = serde_json::from_value(params.ok_or(JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?)
    .map_err(|e| JsonRpcError {
        code: -32602,
        message: format!("Invalid params: {}", e),
        data: None,
    })?;
    
    // Parse proposal ID
    let proposal_id_bytes = hex::decode(&params.proposal_id.trim_start_matches("0x"))
        .map_err(|e| JsonRpcError {
            code: -32602,
            message: format!("Invalid proposal ID: {}", e),
            data: None,
        })?;
    
    if proposal_id_bytes.len() != 32 {
        return Err(JsonRpcError {
            code: -32602,
            message: "Proposal ID must be 32 bytes".to_string(),
            data: None,
        });
    }
    
    let mut proposal_id_arr = [0u8; 32];
    proposal_id_arr.copy_from_slice(&proposal_id_bytes);
    let proposal_id = ProposalId(proposal_id_arr);
    
    // Parse voter address
    let voter_bytes = hex::decode(&params.voter.trim_start_matches("0x"))
        .map_err(|e| JsonRpcError {
            code: -32602,
            message: format!("Invalid voter address: {}", e),
            data: None,
        })?;
    
    if voter_bytes.len() != 33 {
        return Err(JsonRpcError {
            code: -32602,
            message: "Voter address must be 33 bytes".to_string(),
            data: None,
        });
    }
    
    let mut voter = [0u8; 33];
    voter.copy_from_slice(&voter_bytes);
    
    // Get current timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Cast vote
    let mut gov = gov_state.manager.write();
    gov.vote(proposal_id, voter, params.support, params.voting_power, timestamp)
        .map_err(|e| JsonRpcError {
            code: -32000,
            message: format!("Failed to vote: {}", e),
            data: None,
        })?;
    
    Ok(json!({
        "status": "vote_cast",
        "support": params.support
    }))
}

/// Get proposal details
pub async fn get_proposal(
    state: &RpcState,
    gov_state: &GovernanceRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    #[derive(Deserialize)]
    struct GetProposalParams {
        proposal_id: String,
    }
    
    let params: GetProposalParams = serde_json::from_value(params.ok_or(JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?)
    .map_err(|e| JsonRpcError {
        code: -32602,
        message: format!("Invalid params: {}", e),
        data: None,
    })?;
    
    // Parse proposal ID
    let proposal_id_bytes = hex::decode(&params.proposal_id.trim_start_matches("0x"))
        .map_err(|e| JsonRpcError {
            code: -32602,
            message: format!("Invalid proposal ID: {}", e),
            data: None,
        })?;
    
    if proposal_id_bytes.len() != 32 {
        return Err(JsonRpcError {
            code: -32602,
            message: "Proposal ID must be 32 bytes".to_string(),
            data: None,
        });
    }
    
    let mut proposal_id_arr = [0u8; 32];
    proposal_id_arr.copy_from_slice(&proposal_id_bytes);
    let proposal_id = ProposalId(proposal_id_arr);
    
    // Get proposal
    let gov = gov_state.manager.read();
    let proposal = gov.get_proposal(&proposal_id)
        .ok_or(JsonRpcError {
            code: -32000,
            message: "Proposal not found".to_string(),
            data: None,
        })?;
    
    // Convert to JSON
    Ok(json!({
        "id": format!("0x{}", hex::encode(&proposal.id.0)),
        "proposer": format!("0x{}", hex::encode(&proposal.proposer)),
        "description": proposal.description,
        "created_at": proposal.created_at,
        "status": format!("{:?}", proposal.status),
        "votes_for": proposal.votes_for,
        "votes_against": proposal.votes_against,
        "vote_percentage_for": proposal.vote_percentage_for(),
        "executed_at": proposal.executed_at,
    }))
}

/// Finalize a proposal
pub async fn finalize_proposal(
    state: &RpcState,
    gov_state: &GovernanceRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    #[derive(Deserialize)]
    struct FinalizeParams {
        proposal_id: String,
    }
    
    let params: FinalizeParams = serde_json::from_value(params.ok_or(JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?)
    .map_err(|e| JsonRpcError {
        code: -32602,
        message: format!("Invalid params: {}", e),
        data: None,
    })?;
    
    // Parse proposal ID
    let proposal_id_bytes = hex::decode(&params.proposal_id.trim_start_matches("0x"))
        .map_err(|e| JsonRpcError {
            code: -32602,
            message: format!("Invalid proposal ID: {}", e),
            data: None,
        })?;
    
    if proposal_id_bytes.len() != 32 {
        return Err(JsonRpcError {
            code: -32602,
            message: "Proposal ID must be 32 bytes".to_string(),
            data: None,
        });
    }
    
    let mut proposal_id_arr = [0u8; 32];
    proposal_id_arr.copy_from_slice(&proposal_id_bytes);
    let proposal_id = ProposalId(proposal_id_arr);
    
    // Get current timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Finalize proposal
    let mut gov = gov_state.manager.write();
    let passed = gov.finalize_proposal(proposal_id, timestamp)
        .map_err(|e| JsonRpcError {
            code: -32000,
            message: format!("Failed to finalize proposal: {}", e),
            data: None,
        })?;
    
    Ok(json!({
        "passed": passed,
        "status": if passed { "executed" } else { "rejected" }
    }))
}

/// Delegate voting power
pub async fn delegate_voting_power(
    state: &RpcState,
    gov_state: &GovernanceRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    #[derive(Deserialize)]
    struct DelegateParams {
        delegator: String,
        delegatee: String,
        amount: u64,
    }
    
    let params: DelegateParams = serde_json::from_value(params.ok_or(JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?)
    .map_err(|e| JsonRpcError {
        code: -32602,
        message: format!("Invalid params: {}", e),
        data: None,
    })?;
    
    // Parse addresses
    let delegator_bytes = hex::decode(&params.delegator.trim_start_matches("0x"))
        .map_err(|e| JsonRpcError {
            code: -32602,
            message: format!("Invalid delegator address: {}", e),
            data: None,
        })?;
    
    let delegatee_bytes = hex::decode(&params.delegatee.trim_start_matches("0x"))
        .map_err(|e| JsonRpcError {
            code: -32602,
            message: format!("Invalid delegatee address: {}", e),
            data: None,
        })?;
    
    if delegator_bytes.len() != 33 || delegatee_bytes.len() != 33 {
        return Err(JsonRpcError {
            code: -32602,
            message: "Addresses must be 33 bytes".to_string(),
            data: None,
        });
    }
    
    let mut delegator = [0u8; 33];
    let mut delegatee = [0u8; 33];
    delegator.copy_from_slice(&delegator_bytes);
    delegatee.copy_from_slice(&delegatee_bytes);
    
    // Delegate
    let mut gov = gov_state.manager.write();
    gov.delegate(delegator, delegatee, params.amount)
        .map_err(|e| JsonRpcError {
            code: -32000,
            message: format!("Failed to delegate: {}", e),
            data: None,
        })?;
    
    Ok(json!({
        "status": "delegated",
        "amount": params.amount
    }))
}

/// Get voting power (including delegations)
pub async fn get_voting_power(
    state: &RpcState,
    gov_state: &GovernanceRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    #[derive(Deserialize)]
    struct GetPowerParams {
        address: String,
        base_power: u64,
    }
    
    let params: GetPowerParams = serde_json::from_value(params.ok_or(JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?)
    .map_err(|e| JsonRpcError {
        code: -32602,
        message: format!("Invalid params: {}", e),
        data: None,
    })?;
    
    // Parse address
    let address_bytes = hex::decode(&params.address.trim_start_matches("0x"))
        .map_err(|e| JsonRpcError {
            code: -32602,
            message: format!("Invalid address: {}", e),
            data: None,
        })?;
    
    if address_bytes.len() != 33 {
        return Err(JsonRpcError {
            code: -32602,
            message: "Address must be 33 bytes".to_string(),
            data: None,
        });
    }
    
    let mut address = [0u8; 33];
    address.copy_from_slice(&address_bytes);
    
    // Get voting power
    let gov = gov_state.manager.read();
    let total_power = gov.get_voting_power(&address, params.base_power);
    
    Ok(json!({
        "total_voting_power": total_power,
        "base_power": params.base_power,
        "delegated_power": total_power - params.base_power
    }))
}
