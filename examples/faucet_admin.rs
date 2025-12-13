// Example: Running BitCell Admin Console with Testnet Faucet
//
// This example shows how to set up and run the admin console
// with the testnet faucet enabled.

use bitcell_admin::{AdminConsole, faucet::FaucetConfig};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // Configure the faucet
    // Load configuration from environment variables
    let faucet_config = FaucetConfig {
        // Amount per request (default: 1 CELL)
        amount_per_request: std::env::var("FAUCET_AMOUNT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1_000_000_000),
        
        // Rate limit in seconds (default: 1 hour)
        rate_limit_seconds: std::env::var("FAUCET_RATE_LIMIT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3600),
        
        // Max requests per day (default: 5)
        max_requests_per_day: std::env::var("FAUCET_MAX_REQUESTS_PER_DAY")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5),
        
        // Load private key from environment (NEVER hardcode!)
        private_key: std::env::var("FAUCET_PRIVATE_KEY")
            .expect("FAUCET_PRIVATE_KEY environment variable must be set"),
        
        // Node RPC endpoint
        node_rpc_host: std::env::var("FAUCET_NODE_RPC_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string()),
        node_rpc_port: std::env::var("FAUCET_NODE_RPC_PORT")
            .unwrap_or_else(|_| "8545".to_string())
            .parse()
            .expect("Invalid FAUCET_NODE_RPC_PORT"),
        
        // CAPTCHA verification (default: false - not implemented)
        require_captcha: std::env::var("FAUCET_REQUIRE_CAPTCHA")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(false),
        
        // Maximum recipient balance (default: 10 CELL)
        max_recipient_balance: std::env::var("FAUCET_MAX_RECIPIENT_BALANCE")
            .ok()
            .and_then(|s| s.parse().ok())
            .map(|v: u64| if v > 0 { Some(v) } else { None })
            .unwrap_or(Some(10_000_000_000)),
    };

    // Create admin console with faucet
    let addr: SocketAddr = std::env::var("ADMIN_CONSOLE_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_string())
        .parse()
        .expect("Invalid ADMIN_CONSOLE_ADDR");

    println!("Starting BitCell Admin Console with Faucet...");
    println!("  Address: http://{}", addr);
    println!("  Faucet UI: http://{}/faucet", addr);
    println!("  API Docs: http://{}/api/faucet/info", addr);
    println!();
    println!("Faucet Configuration:");
    println!("  Amount per request: {} CELL", faucet_config.amount_per_request as f64 / 1e9);
    println!("  Rate limit: {} seconds", faucet_config.rate_limit_seconds);
    println!("  Max requests/day: {}", faucet_config.max_requests_per_day);
    println!("  CAPTCHA required: {} (WARNING: not implemented - must be false)", faucet_config.require_captcha);

    let console = AdminConsole::new(addr)
        .with_faucet(faucet_config)
        .expect("Failed to initialize faucet");

    // Start the server
    console.serve().await
}

// Example environment setup:
//
// ```bash
// # Generate a new testnet wallet
// # (In production, use bitcell-wallet or similar)
// export FAUCET_PRIVATE_KEY="1234567890123456789012345678901234567890123456789012345678901234"
//
// # Configure node RPC endpoint
// export FAUCET_NODE_RPC_HOST="127.0.0.1"
// export FAUCET_NODE_RPC_PORT="8545"
//
// # Optional: customize faucet amount and limits
// export FAUCET_AMOUNT="1000000000"              # 1 CELL per request
// export FAUCET_RATE_LIMIT="3600"                # 1 hour between requests
// export FAUCET_MAX_REQUESTS_PER_DAY="5"         # 5 requests per day max
// export FAUCET_MAX_RECIPIENT_BALANCE="10000000000"  # 10 CELL max balance
//
// # CAPTCHA (keep disabled - not implemented)
// export FAUCET_REQUIRE_CAPTCHA="false"
//
// # Run the admin console
// cargo run --example faucet_admin
// ```
//
// Then visit http://localhost:8080/faucet in your browser.
