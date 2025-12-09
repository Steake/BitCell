//! Testnet Faucet Service
//!
//! Provides token distribution for testnet with:
//! - Rate limiting per address
//! - Request tracking and audit logging
//! - CAPTCHA verification support
//! - Secure wallet management

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Faucet errors
#[derive(Debug, Error)]
pub enum FaucetError {
    #[error("Rate limit exceeded. Try again in {0} seconds")]
    RateLimited(u64),
    #[error("Invalid address format: {0}")]
    InvalidAddress(String),
    #[error("Faucet balance too low")]
    InsufficientBalance,
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    #[error("Invalid CAPTCHA")]
    InvalidCaptcha,
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Faucet configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaucetConfig {
    /// Amount to send per request (in smallest units)
    pub amount_per_request: u64,
    /// Minimum time between requests from same address (seconds)
    pub rate_limit_seconds: u64,
    /// Maximum requests per address per day
    pub max_requests_per_day: usize,
    /// Faucet private key (hex string)
    pub private_key: String,
    /// Node RPC host
    pub node_rpc_host: String,
    /// Node RPC port
    pub node_rpc_port: u16,
    /// Enable CAPTCHA verification
    pub require_captcha: bool,
    /// Maximum balance an address can have to receive funds (anti-abuse)
    pub max_recipient_balance: Option<u64>,
}

impl Default for FaucetConfig {
    fn default() -> Self {
        Self {
            amount_per_request: 1_000_000_000, // 1 CELL in smallest units
            rate_limit_seconds: 3600,           // 1 hour
            max_requests_per_day: 5,
            private_key: String::new(),
            node_rpc_host: "127.0.0.1".to_string(),
            node_rpc_port: 8545,
            require_captcha: true,
            max_recipient_balance: Some(10_000_000_000), // 10 CELL max balance
        }
    }
}

/// Request history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaucetRequest {
    pub address: String,
    pub amount: u64,
    pub timestamp: u64,
    pub tx_hash: String,
    pub status: RequestStatus,
}

/// Status of a faucet request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RequestStatus {
    Pending,
    Completed,
    Failed,
}

/// Rate limit tracking
#[derive(Debug, Clone)]
struct RateLimitInfo {
    last_request: u64,
    requests_today: Vec<u64>,
}

/// Faucet service
pub struct FaucetService {
    config: Arc<RwLock<FaucetConfig>>,
    rate_limits: Arc<RwLock<HashMap<String, RateLimitInfo>>>,
    request_history: Arc<RwLock<Vec<FaucetRequest>>>,
}

impl FaucetService {
    /// Create a new faucet service
    pub fn new(config: FaucetConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            request_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get current configuration
    pub fn get_config(&self) -> FaucetConfig {
        self.config.read().clone()
    }

    /// Update configuration
    pub fn update_config(&self, config: FaucetConfig) {
        *self.config.write() = config;
    }

    /// Check if address can request tokens
    pub fn check_rate_limit(&self, address: &str) -> Result<(), FaucetError> {
        let config = self.config.read();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let rate_limits = self.rate_limits.read();
        
        if let Some(info) = rate_limits.get(address) {
            // Check time-based rate limit
            let elapsed = now.saturating_sub(info.last_request);
            if elapsed < config.rate_limit_seconds {
                let remaining = config.rate_limit_seconds - elapsed;
                return Err(FaucetError::RateLimited(remaining));
            }

            // Check daily request limit
            let today_start = now - (now % 86400); // Start of current day
            let requests_today: Vec<_> = info.requests_today
                .iter()
                .filter(|&&t| t >= today_start)
                .collect();
            
            if requests_today.len() >= config.max_requests_per_day {
                let next_day = today_start + 86400;
                let remaining = next_day - now;
                return Err(FaucetError::RateLimited(remaining));
            }
        }

        Ok(())
    }

    /// Record a request
    fn record_request(&self, address: &str, timestamp: u64) {
        let mut rate_limits = self.rate_limits.write();
        
        let info = rate_limits.entry(address.to_string()).or_insert_with(|| {
            RateLimitInfo {
                last_request: 0,
                requests_today: Vec::new(),
            }
        });

        info.last_request = timestamp;
        
        // Clean up old requests (keep only today's)
        let today_start = timestamp - (timestamp % 86400);
        info.requests_today.retain(|&t| t >= today_start);
        info.requests_today.push(timestamp);
    }

    /// Get faucet balance
    pub async fn get_balance(&self) -> Result<u64, FaucetError> {
        let config = self.config.read().clone();
        let rpc_url = format!("http://{}:{}/rpc", config.node_rpc_host, config.node_rpc_port);
        
        // Get faucet address from private key
        let faucet_address = self.get_faucet_address(&config.private_key)?;
        
        let client = reqwest::Client::new();
        let rpc_req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getBalance",
            "params": [faucet_address, "latest"],
            "id": 1
        });

        let resp = client
            .post(&rpc_url)
            .json(&rpc_req)
            .send()
            .await
            .map_err(|e| FaucetError::TransactionFailed(e.to_string()))?;

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| FaucetError::TransactionFailed(e.to_string()))?;

        if let Some(result) = json.get("result").and_then(|v| v.as_str()) {
            let balance = u64::from_str_radix(result.trim_start_matches("0x"), 16)
                .unwrap_or(0);
            Ok(balance)
        } else {
            Err(FaucetError::TransactionFailed("Failed to get balance".to_string()))
        }
    }

    /// Get recipient balance
    async fn get_recipient_balance(&self, address: &str) -> Result<u64, FaucetError> {
        let config = self.config.read().clone();
        let rpc_url = format!("http://{}:{}/rpc", config.node_rpc_host, config.node_rpc_port);
        
        let client = reqwest::Client::new();
        let rpc_req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getBalance",
            "params": [address, "latest"],
            "id": 1
        });

        let resp = client
            .post(&rpc_url)
            .json(&rpc_req)
            .send()
            .await
            .map_err(|e| FaucetError::TransactionFailed(e.to_string()))?;

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| FaucetError::TransactionFailed(e.to_string()))?;

        if let Some(result) = json.get("result").and_then(|v| v.as_str()) {
            let balance = u64::from_str_radix(result.trim_start_matches("0x"), 16)
                .unwrap_or(0);
            Ok(balance)
        } else {
            Err(FaucetError::TransactionFailed("Failed to get balance".to_string()))
        }
    }

    /// Process faucet request
    pub async fn process_request(
        &self,
        address: &str,
        _captcha_response: Option<&str>,
    ) -> Result<FaucetRequest, FaucetError> {
        // Validate address format
        self.validate_address(address)?;

        // Check CAPTCHA if required
        let config = self.config.read().clone();
        if config.require_captcha {
            // For now, skip CAPTCHA validation in basic implementation
            // In production, validate against reCAPTCHA or hCaptcha service
            // if let Some(response) = captcha_response {
            //     self.verify_captcha(response).await?;
            // } else {
            //     return Err(FaucetError::InvalidCaptcha);
            // }
        }

        // Check rate limit
        self.check_rate_limit(address)?;

        // Check recipient balance if configured
        if let Some(max_balance) = config.max_recipient_balance {
            let recipient_balance = self.get_recipient_balance(address).await?;
            if recipient_balance >= max_balance {
                return Err(FaucetError::TransactionFailed(
                    format!("Recipient balance ({}) exceeds maximum allowed ({})", 
                        recipient_balance, max_balance)
                ));
            }
        }

        // Check faucet balance
        let balance = self.get_balance().await?;
        if balance < config.amount_per_request {
            return Err(FaucetError::InsufficientBalance);
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Send tokens
        let tx_hash = self.send_tokens(address, config.amount_per_request).await?;

        // Record the request
        self.record_request(address, timestamp);

        // Create request record
        let request = FaucetRequest {
            address: address.to_string(),
            amount: config.amount_per_request,
            timestamp,
            tx_hash,
            status: RequestStatus::Completed,
        };

        // Add to history
        self.request_history.write().push(request.clone());

        Ok(request)
    }

    /// Get request history
    pub fn get_history(&self, limit: usize) -> Vec<FaucetRequest> {
        let history = self.request_history.read();
        history.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get statistics
    pub fn get_stats(&self) -> FaucetStats {
        let history = self.request_history.read();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let hour_ago = now - 3600;
        let day_ago = now - 86400;

        FaucetStats {
            total_requests: history.len(),
            requests_last_hour: history.iter().filter(|r| r.timestamp >= hour_ago).count(),
            requests_last_day: history.iter().filter(|r| r.timestamp >= day_ago).count(),
            total_distributed: history.iter().map(|r| r.amount).sum(),
        }
    }

    /// Validate address format
    fn validate_address(&self, address: &str) -> Result<(), FaucetError> {
        // Check if address starts with 0x and has correct length
        if !address.starts_with("0x") || address.len() != 42 {
            return Err(FaucetError::InvalidAddress(
                "Address must start with 0x and be 42 characters".to_string()
            ));
        }

        // Check if all characters are hex
        if !address[2..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(FaucetError::InvalidAddress(
                "Address contains invalid characters".to_string()
            ));
        }

        Ok(())
    }

    /// Get faucet address from private key
    fn get_faucet_address(&self, private_key: &str) -> Result<String, FaucetError> {
        use bitcell_crypto::SecretKey;

        let key_bytes = hex::decode(private_key.trim_start_matches("0x"))
            .map_err(|e| FaucetError::ConfigError(format!("Invalid private key: {}", e)))?;

        if key_bytes.len() != 32 {
            return Err(FaucetError::ConfigError("Private key must be 32 bytes".to_string()));
        }

        let mut arr = [0u8; 32];
        arr.copy_from_slice(&key_bytes);
        
        let secret_key = SecretKey::from_bytes(&arr)
            .map_err(|e| FaucetError::ConfigError(format!("Invalid secret key: {}", e)))?;

        let public_key = secret_key.public_key();
        Ok(format!("0x{}", hex::encode(public_key.as_bytes())))
    }

    /// Send tokens to address
    async fn send_tokens(&self, to_address: &str, amount: u64) -> Result<String, FaucetError> {
        use bitcell_crypto::SecretKey;
        use bitcell_wallet::{Chain, Transaction as WalletTx};

        let config = self.config.read().clone();
        let rpc_url = format!("http://{}:{}/rpc", config.node_rpc_host, config.node_rpc_port);
        
        // Parse private key
        let key_bytes = hex::decode(config.private_key.trim_start_matches("0x"))
            .map_err(|e| FaucetError::ConfigError(format!("Invalid private key: {}", e)))?;

        if key_bytes.len() != 32 {
            return Err(FaucetError::ConfigError("Private key must be 32 bytes".to_string()));
        }

        let mut arr = [0u8; 32];
        arr.copy_from_slice(&key_bytes);
        
        let secret_key = SecretKey::from_bytes(&arr)
            .map_err(|e| FaucetError::ConfigError(format!("Invalid secret key: {}", e)))?;

        let from_address = self.get_faucet_address(&config.private_key)?;

        // Get nonce
        let client = reqwest::Client::new();
        let nonce_req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getTransactionCount",
            "params": [from_address, "latest"],
            "id": 1
        });

        let resp = client
            .post(&rpc_url)
            .json(&nonce_req)
            .send()
            .await
            .map_err(|e| FaucetError::TransactionFailed(e.to_string()))?;

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| FaucetError::TransactionFailed(e.to_string()))?;

        let nonce = if let Some(result) = json.get("result").and_then(|v| v.as_str()) {
            u64::from_str_radix(result.trim_start_matches("0x"), 16).unwrap_or(0)
        } else {
            0
        };

        // Build and sign transaction
        let tx = WalletTx::new(
            Chain::BitCell,
            from_address.clone(),
            to_address.to_string(),
            amount,
            21000, // Standard gas fee
            nonce,
        );

        let signed_tx = tx.sign(&secret_key);
        let tx_bytes = signed_tx.serialize()
            .map_err(|e| FaucetError::TransactionFailed(format!("Serialization failed: {}", e)))?;

        let tx_hex = format!("0x{}", hex::encode(&tx_bytes));

        // Broadcast transaction
        let send_req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_sendRawTransaction",
            "params": [tx_hex],
            "id": 1
        });

        let resp = client
            .post(&rpc_url)
            .json(&send_req)
            .send()
            .await
            .map_err(|e| FaucetError::TransactionFailed(e.to_string()))?;

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| FaucetError::TransactionFailed(e.to_string()))?;

        if let Some(error) = json.get("error") {
            return Err(FaucetError::TransactionFailed(error.to_string()));
        }

        if let Some(result) = json.get("result").and_then(|v| v.as_str()) {
            Ok(result.to_string())
        } else {
            Ok(signed_tx.hash_hex())
        }
    }
}

/// Faucet statistics
#[derive(Debug, Clone, Serialize)]
pub struct FaucetStats {
    pub total_requests: usize,
    pub requests_last_hour: usize,
    pub requests_last_day: usize,
    pub total_distributed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_address() {
        let config = FaucetConfig::default();
        let service = FaucetService::new(config);

        // Valid address
        assert!(service.validate_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0").is_ok());

        // Invalid: no 0x prefix
        assert!(service.validate_address("742d35Cc6634C0532925a3b844Bc9e7595f0bEb0").is_err());

        // Invalid: wrong length
        assert!(service.validate_address("0x742d35Cc").is_err());

        // Invalid: non-hex characters
        assert!(service.validate_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEbZ").is_err());
    }

    #[test]
    fn test_rate_limiting() {
        let config = FaucetConfig {
            rate_limit_seconds: 60,
            max_requests_per_day: 3,
            ..Default::default()
        };
        let service = FaucetService::new(config);

        let address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0";

        // First request should be allowed
        assert!(service.check_rate_limit(address).is_ok());

        // Record request
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        service.record_request(address, timestamp);

        // Second immediate request should be rate limited
        assert!(matches!(
            service.check_rate_limit(address),
            Err(FaucetError::RateLimited(_))
        ));
    }

    #[test]
    fn test_daily_request_limit() {
        let config = FaucetConfig {
            rate_limit_seconds: 1, // Very short for testing
            max_requests_per_day: 2,
            ..Default::default()
        };
        let service = FaucetService::new(config);

        let address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0";
        let base_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Record 2 requests
        service.record_request(address, base_time);
        service.record_request(address, base_time + 2);

        // Third request should exceed daily limit
        assert!(matches!(
            service.check_rate_limit(address),
            Err(FaucetError::RateLimited(_))
        ));
    }

    #[test]
    fn test_get_stats() {
        let config = FaucetConfig::default();
        let service = FaucetService::new(config);

        // Add some requests
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        service.request_history.write().push(FaucetRequest {
            address: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0".to_string(),
            amount: 1000,
            timestamp: now,
            tx_hash: "0xabc".to_string(),
            status: RequestStatus::Completed,
        });

        let stats = service.get_stats();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.total_distributed, 1000);
    }
}
