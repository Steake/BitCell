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
    // In production, load these from environment variables
    let faucet_config = FaucetConfig {
        // Amount per request: 1 CELL (1_000_000_000 smallest units)
        amount_per_request: 1_000_000_000,
        
        // Rate limit: 1 hour between requests
        rate_limit_seconds: 3600,
        
        // Max 5 requests per day per address
        max_requests_per_day: 5,
        
        // Load private key from environment (NEVER hardcode!)
        private_key: std::env::var("FAUCET_PRIVATE_KEY")
            .expect("FAUCET_PRIVATE_KEY environment variable must be set"),
        
        // Node RPC endpoint
        node_rpc_host: std::env::var("NODE_RPC_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string()),
        node_rpc_port: std::env::var("NODE_RPC_PORT")
            .unwrap_or_else(|_| "8545".to_string())
            .parse()
            .expect("Invalid NODE_RPC_PORT"),
        
        // Enable CAPTCHA (recommended for public testnets)
        require_captcha: std::env::var("FAUCET_REQUIRE_CAPTCHA")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true),
        
        // Maximum recipient balance: 10 CELL
        max_recipient_balance: Some(10_000_000_000),
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
// export FAUCET_PRIVATE_KEY="0x1234567890abcdef..."
//
// # Configure node RPC
// export NODE_RPC_HOST="127.0.0.1"
// export NODE_RPC_PORT="8545"
//
// # Optional: customize faucet settings
// export FAUCET_REQUIRE_CAPTCHA="true"
//
// # Run the admin console
// cargo run --example faucet_admin
// ```
//
// Then visit http://localhost:8080/faucet in your browser.
