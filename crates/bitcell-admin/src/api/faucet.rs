//! Faucet API endpoints

use axum::{
    extract::{State, Json},
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::{AppState, faucet::{FaucetError, FaucetRequest as ServiceRequest}};

/// Faucet request
#[derive(Debug, Deserialize)]
pub struct FaucetRequest {
    /// Recipient address
    pub address: String,
    /// CAPTCHA response token
    pub captcha_response: Option<String>,
}

/// Faucet response
#[derive(Debug, Serialize)]
pub struct FaucetResponse {
    pub success: bool,
    pub message: String,
    pub tx_hash: Option<String>,
    pub amount: Option<u64>,
}

/// Faucet info response
#[derive(Debug, Serialize)]
pub struct FaucetInfoResponse {
    pub balance: u64,
    pub amount_per_request: u64,
    pub rate_limit_seconds: u64,
    pub max_requests_per_day: usize,
    pub require_captcha: bool,
}

/// Request testnet tokens
pub async fn request_tokens(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FaucetRequest>,
) -> impl IntoResponse {
    let faucet = match &state.faucet {
        Some(f) => f,
        None => return (
            StatusCode::NOT_FOUND,
            Json(FaucetResponse {
                success: false,
                message: "Faucet not enabled".to_string(),
                tx_hash: None,
                amount: None,
            })
        ).into_response(),
    };

    match faucet.process_request(
        &req.address,
        req.captcha_response.as_deref(),
    ).await {
        Ok(request) => {
            Json(FaucetResponse {
                success: true,
                message: format!(
                    "Successfully sent {} tokens to {}",
                    request.amount, request.address
                ),
                tx_hash: Some(request.tx_hash),
                amount: Some(request.amount),
            }).into_response()
        }
        Err(e) => {
            let (status, message) = match e {
                FaucetError::RateLimited(seconds) => (
                    StatusCode::TOO_MANY_REQUESTS,
                    format!("Rate limit exceeded. Try again in {} seconds", seconds)
                ),
                FaucetError::InvalidAddress(msg) => (
                    StatusCode::BAD_REQUEST,
                    msg
                ),
                FaucetError::InvalidCaptcha => (
                    StatusCode::BAD_REQUEST,
                    "Invalid CAPTCHA response".to_string()
                ),
                FaucetError::InsufficientBalance => (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Faucet balance too low. Please contact administrator.".to_string()
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to process request: {}", e)
                ),
            };

            (
                status,
                Json(FaucetResponse {
                    success: false,
                    message,
                    tx_hash: None,
                    amount: None,
                })
            ).into_response()
        }
    }
}

/// Get faucet information
pub async fn get_info(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let faucet = match &state.faucet {
        Some(f) => f,
        None => return (StatusCode::NOT_FOUND, "Faucet not enabled").into_response(),
    };

    let config = faucet.get_config();
    
    let balance = match faucet.get_balance().await {
        Ok(b) => b,
        Err(_) => 0,
    };

    Json(FaucetInfoResponse {
        balance,
        amount_per_request: config.amount_per_request,
        rate_limit_seconds: config.rate_limit_seconds,
        max_requests_per_day: config.max_requests_per_day,
        require_captcha: config.require_captcha,
    }).into_response()
}

/// Get recent faucet requests
pub async fn get_history(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let faucet = match &state.faucet {
        Some(f) => f,
        None => return (StatusCode::NOT_FOUND, "Faucet not enabled").into_response(),
    };

    let history = faucet.get_history(50);
    Json(history).into_response()
}

/// Get faucet statistics
pub async fn get_stats(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let faucet = match &state.faucet {
        Some(f) => f,
        None => return (StatusCode::NOT_FOUND, "Faucet not enabled").into_response(),
    };

    let stats = faucet.get_stats();
    Json(stats).into_response()
}

/// Check if address can request tokens
#[derive(Debug, Deserialize)]
pub struct CheckEligibilityRequest {
    pub address: String,
}

#[derive(Debug, Serialize)]
pub struct CheckEligibilityResponse {
    pub eligible: bool,
    pub message: String,
    pub retry_after_seconds: Option<u64>,
}

pub async fn check_eligibility(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CheckEligibilityRequest>,
) -> impl IntoResponse {
    let faucet = match &state.faucet {
        Some(f) => f,
        None => return (StatusCode::NOT_FOUND, "Faucet not enabled").into_response(),
    };

    match faucet.check_rate_limit(&req.address) {
        Ok(_) => Json(CheckEligibilityResponse {
            eligible: true,
            message: "Address is eligible for faucet request".to_string(),
            retry_after_seconds: None,
        }).into_response(),
        Err(FaucetError::RateLimited(seconds)) => Json(CheckEligibilityResponse {
            eligible: false,
            message: format!("Rate limit active. Try again in {} seconds", seconds),
            retry_after_seconds: Some(seconds),
        }).into_response(),
        Err(e) => Json(CheckEligibilityResponse {
            eligible: false,
            message: e.to_string(),
            retry_after_seconds: None,
        }).into_response(),
    }
}
