//! Block Explorer API endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;

#[derive(Debug, Serialize)]
pub struct TransactionDetail {
    pub hash: String,
    pub block_height: u64,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
    pub timestamp: u64,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct AccountInfo {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
    pub transaction_count: usize,
    pub trust_score: f64,
    pub is_miner: bool,
}

#[derive(Debug, Serialize)]
pub struct TransactionHistoryResponse {
    pub transactions: Vec<TransactionDetail>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(Debug, Serialize)]
pub enum SearchResult {
    Block { height: u64, hash: String },
    Transaction { hash: String },
    Account { address: String },
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_per_page")]
    pub per_page: usize,
}

fn default_page() -> usize {
    1
}

fn default_per_page() -> usize {
    20
}

/// Get transaction details by hash
pub async fn get_transaction(
    State(_state): State<Arc<AppState>>,
    Path(hash): Path<String>,
) -> Result<Json<TransactionDetail>, (StatusCode, Json<String>)> {
    // In a real implementation, this would query the blockchain
    // For now, return mock data
    
    // Validate hash format
    if !hash.starts_with("0x") || hash.len() != 66 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json("Invalid transaction hash format".to_string()),
        ));
    }
    
    Ok(Json(TransactionDetail {
        hash: hash.clone(),
        block_height: 12345,
        from: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb8".to_string(),
        to: "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed".to_string(),
        amount: 1000000, // 1 CELL in micro-units
        fee: 21000,
        nonce: 5,
        timestamp: 1700000000,
        status: "confirmed".to_string(),
    }))
}

/// Get account information and balance
pub async fn get_account(
    State(_state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> Result<Json<AccountInfo>, (StatusCode, Json<String>)> {
    // In a real implementation, this would query the state manager
    // For now, return mock data
    
    // Validate address format
    if !address.starts_with("0x") || address.len() != 42 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json("Invalid address format".to_string()),
        ));
    }
    
    // Mock trust score calculation
    let trust_score = 0.85;
    
    Ok(Json(AccountInfo {
        address: address.clone(),
        balance: 5000000, // 5 CELL
        nonce: 10,
        transaction_count: 25,
        trust_score,
        is_miner: trust_score > 0.75,
    }))
}

/// Get transaction history for an account
pub async fn get_account_transactions(
    State(_state): State<Arc<AppState>>,
    Path(address): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<TransactionHistoryResponse>, (StatusCode, Json<String>)> {
    // Validate address format
    if !address.starts_with("0x") || address.len() != 42 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json("Invalid address format".to_string()),
        ));
    }
    
    // In a real implementation, this would query the transaction index
    // For now, return mock data
    let mut transactions = Vec::new();
    
    // Generate some mock transactions
    for i in 0..10 {
        transactions.push(TransactionDetail {
            hash: format!("0x{:064x}", i * 12345),
            block_height: 12340 + i,
            from: if i % 2 == 0 { address.clone() } else { "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb8".to_string() },
            to: if i % 2 == 0 { "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb8".to_string() } else { address.clone() },
            amount: 100000 * (i + 1),
            fee: 21000,
            nonce: i,
            timestamp: 1700000000 + (i * 600),
            status: "confirmed".to_string(),
        });
    }
    
    // Apply pagination
    let start = (pagination.page - 1) * pagination.per_page;
    let end = start + pagination.per_page;
    let paginated = transactions.into_iter().skip(start).take(pagination.per_page).collect::<Vec<_>>();
    
    Ok(Json(TransactionHistoryResponse {
        transactions: paginated,
        total: 10,
        page: pagination.page,
        per_page: pagination.per_page,
    }))
}

/// Search for blocks, transactions, or accounts
pub async fn search(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<String>)> {
    let q = query.q.trim();
    let mut results = Vec::new();
    
    if q.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json("Empty search query".to_string()),
        ));
    }
    
    // Check if it's a block height (numeric)
    if let Ok(height) = q.parse::<u64>() {
        results.push(SearchResult::Block {
            height,
            hash: format!("0x{:016x}", height * 12345),
        });
    }
    
    // Check if it's a transaction hash (0x + 64 hex chars)
    if q.starts_with("0x") && q.len() == 66 {
        results.push(SearchResult::Transaction {
            hash: q.to_string(),
        });
    }
    
    // Check if it's a block hash (0x + 16 hex chars for our simplified mock)
    if q.starts_with("0x") && q.len() == 18 {
        // Try to find block by hash
        results.push(SearchResult::Block {
            height: 12345, // Mock value
            hash: q.to_string(),
        });
    }
    
    // Check if it's an account address (0x + 40 hex chars)
    if q.starts_with("0x") && q.len() == 42 {
        results.push(SearchResult::Account {
            address: q.to_string(),
        });
    }
    
    if results.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            Json("No results found".to_string()),
        ));
    }
    
    Ok(Json(SearchResponse { results }))
}

/// Get trust score for an account
pub async fn get_trust_score(
    State(_state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> Result<Json<TrustScoreResponse>, (StatusCode, Json<String>)> {
    // Validate address format
    if !address.starts_with("0x") || address.len() != 42 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json("Invalid address format".to_string()),
        ));
    }
    
    // In a real implementation, this would query the EBSL system
    // For now, return mock data
    Ok(Json(TrustScoreResponse {
        address: address.clone(),
        trust_score: 0.85,
        belief: 0.70,
        disbelief: 0.15,
        uncertainty: 0.15,
        positive_evidence: 100,
        negative_evidence: 5,
        total_blocks_proposed: 150,
        slashing_events: 0,
        status: "active".to_string(),
    }))
}

#[derive(Debug, Serialize)]
pub struct TrustScoreResponse {
    pub address: String,
    pub trust_score: f64,
    pub belief: f64,
    pub disbelief: f64,
    pub uncertainty: f64,
    pub positive_evidence: u64,
    pub negative_evidence: u64,
    pub total_blocks_proposed: u64,
    pub slashing_events: u64,
    pub status: String,
}
