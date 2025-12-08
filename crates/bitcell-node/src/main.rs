//! BitCell node binary

use bitcell_node::{NodeConfig, ValidatorNode, MinerNode};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bitcell-node")]
#[command(about = "BitCell blockchain node", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run as validator
    Validator {
        #[arg(short, long, default_value_t = 30333)]
        port: u16,
        #[arg(long, default_value_t = 30334)]
        rpc_port: u16,
        #[arg(long)]
        data_dir: Option<PathBuf>,
        #[arg(long)]
        enable_dht: bool,
        #[arg(long)]
        bootstrap: Option<String>,
        #[arg(long)]
        key_seed: Option<String>,
        #[arg(long)]
        key_file: Option<PathBuf>,
        #[arg(long)]
        private_key: Option<String>,
    },
    /// Run as miner
    Miner {
        #[arg(short, long, default_value_t = 30333)]
        port: u16,
        #[arg(long, default_value_t = 30334)]
        rpc_port: u16,
        #[arg(long)]
        data_dir: Option<PathBuf>,
        #[arg(long)]
        enable_dht: bool,
        #[arg(long)]
        bootstrap: Option<String>,
        #[arg(long)]
        key_seed: Option<String>,
        #[arg(long)]
        key_file: Option<PathBuf>,
        #[arg(long)]
        private_key: Option<String>,
    },
    /// Run as full node
    FullNode {
        #[arg(short, long, default_value_t = 30333)]
        port: u16,
        #[arg(long, default_value_t = 30334)]
        rpc_port: u16,
        #[arg(long)]
        data_dir: Option<PathBuf>,
        #[arg(long)]
        enable_dht: bool,
        #[arg(long)]
        bootstrap: Option<String>,
        #[arg(long)]
        key_seed: Option<String>,
        #[arg(long)]
        key_file: Option<PathBuf>,
        #[arg(long)]
        private_key: Option<String>,
    },
    /// Show version
    Version,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validator { port, rpc_port, data_dir, enable_dht, bootstrap, key_seed, key_file, private_key } => {
            println!("🌌 BitCell Validator Node");
            println!("=========================");
            
            let mut config = NodeConfig::default();
            config.network_port = port;
            config.enable_dht = enable_dht;
            config.key_seed = key_seed.clone();
            config.data_dir = data_dir;
            if let Some(bootstrap_node) = bootstrap {
                config.bootstrap_nodes.push(bootstrap_node);
            }
            
            // Resolve secret key
            let secret_key = match bitcell_node::keys::resolve_secret_key(
                private_key.as_deref(),
                key_file.as_deref(),
                None, // Mnemonic not yet supported in CLI args
                key_seed.as_deref()
            ) {
                Ok(sk) => std::sync::Arc::new(sk),
                Err(e) => {
                    eprintln!("Error loading key: {}", e);
                    std::process::exit(1);
                }
            };
            
            tracing::debug!("Validator Public Key: {:?}", secret_key.public_key());
            
            // Initialize node with explicit secret key
            // Note: We need to modify ValidatorNode::new to accept an optional secret key or handle this differently
            // For now, we'll modify ValidatorNode to take the key in config or constructor
            // But since ValidatorNode::new takes config which has key_seed, we might need to update NodeConfig to hold the key
            // or update ValidatorNode::new.
            // Let's check ValidatorNode::new implementation again.
            
            // Actually, ValidatorNode::new derives key from config.key_seed. 
            // We should modify ValidatorNode::new to take the secret key directly.
            // Or we can modify NodeConfig to hold the secret key? No, NodeConfig is serializable.
            
            // Let's update ValidatorNode::new to take the secret key as an argument.
            let mut node = match ValidatorNode::with_key(config, secret_key.clone()) {
                Ok(node) => node,
                Err(e) => {
                    eprintln!("Error initializing validator node: {}", e);
                    std::process::exit(1);
                }
            };
            
            // Start metrics server on port + 2 to avoid conflict with P2P port (30333) and RPC port (30334)
            let metrics_port = port + 2;
            
            // Generate node_id from public key
            let node_id = hex::encode(secret_key.public_key().as_bytes());
            
            // Start RPC server
            let rpc_state = bitcell_node::rpc::RpcState {
                blockchain: node.blockchain.clone(),
                network: (*node.network).clone(),
                tx_pool: node.tx_pool.clone(),
                tournament_manager: Some(node.tournament_manager.clone()),
                config: node.config.clone(),
                node_type: "validator".to_string(),
                node_id,
            };
            
            tokio::spawn(async move {
                println!("RPC server listening on 0.0.0.0:{}", rpc_port);
                if let Err(e) = bitcell_node::rpc::run_server(rpc_state, rpc_port).await {
                    eprintln!("RPC server error: {}", e);
                }
            });

            if let Err(e) = node.start_with_metrics(metrics_port).await {
                eprintln!("Node error: {}", e);
                std::process::exit(1);
            }
            
            println!("Validator ready on port {}", port);
            println!("Metrics available at http://localhost:{}/metrics", metrics_port);
            println!("RPC server available at http://localhost:{}/rpc", rpc_port);
            println!("Press Ctrl+C to stop");
            
            // Keep running
            tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
            println!("\nShutting down...");
        }
        Commands::Miner { port, rpc_port, data_dir, enable_dht, bootstrap, key_seed, key_file, private_key } => {
            println!("⛏️  BitCell Miner Node");
            println!("======================");
            
            let mut config = NodeConfig::default();
            config.network_port = port;
            config.enable_dht = enable_dht;
            config.key_seed = key_seed.clone();
            config.data_dir = data_dir;
            if let Some(bootstrap_node) = bootstrap {
                config.bootstrap_nodes.push(bootstrap_node);
            }
            
            // Resolve secret key
            let secret_key = match bitcell_node::keys::resolve_secret_key(
                private_key.as_deref(),
                key_file.as_deref(),
                None,
                key_seed.as_deref()
            ) {
                Ok(sk) => std::sync::Arc::new(sk),
                Err(e) => {
                    eprintln!("Error loading key: {}", e);
                    std::process::exit(1);
                }
            };
            
            println!("Miner Public Key: {:?}", secret_key.public_key());
            
            let mut node = match MinerNode::with_key(config, secret_key.clone()) {
                Ok(node) => node,
                Err(e) => {
                    eprintln!("Error initializing miner node: {}", e);
                    std::process::exit(1);
                }
            };
            
            let metrics_port = port + 2;

            // Generate node_id from public key
            let node_id = hex::encode(secret_key.public_key().as_bytes());

            // Start RPC server
            let rpc_state = bitcell_node::rpc::RpcState {
                blockchain: node.blockchain.clone(),
                network: (*node.network).clone(),
                tx_pool: node.tx_pool.clone(),
                tournament_manager: None, // Miner doesn't have tournament manager yet
                config: node.config.clone(),
                node_type: "miner".to_string(),
                node_id,
            };
            
            tokio::spawn(async move {
                println!("RPC server listening on 0.0.0.0:{}", rpc_port);
                if let Err(e) = bitcell_node::rpc::run_server(rpc_state, rpc_port).await {
                    eprintln!("RPC server error: {}", e);
                }
            });

            if let Err(e) = node.start_with_metrics(metrics_port).await {
                eprintln!("Node error: {}", e);
                std::process::exit(1);
            }
            
            println!("Miner ready on port {}", port);
            println!("Metrics available at http://localhost:{}/metrics", metrics_port);
            println!("RPC server available at http://localhost:{}/rpc", rpc_port);
            println!("Press Ctrl+C to stop");
            
            tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
            println!("\nShutting down...");
        }
        Commands::FullNode { port, rpc_port, data_dir, enable_dht, bootstrap, key_seed, key_file, private_key } => {
            println!("🌍 BitCell Full Node");
            println!("====================");
            
            let mut config = NodeConfig::default();
            config.network_port = port;
            config.enable_dht = enable_dht;
            config.key_seed = key_seed.clone();
            config.data_dir = data_dir;
            if let Some(bootstrap_node) = bootstrap {
                config.bootstrap_nodes.push(bootstrap_node);
            }
            
            // Resolve secret key
            let secret_key = match bitcell_node::keys::resolve_secret_key(
                private_key.as_deref(),
                key_file.as_deref(),
                None,
                key_seed.as_deref()
            ) {
                Ok(sk) => std::sync::Arc::new(sk),
                Err(e) => {
                    eprintln!("Error loading key: {}", e);
                    std::process::exit(1);
                }
            };
            
            println!("Full Node Public Key: {:?}", secret_key.public_key());

            // Reuse ValidatorNode for now as FullNode logic is similar (just no voting)
            let mut node = match ValidatorNode::with_key(config, secret_key.clone()) {
                Ok(node) => node,
                Err(e) => {
                    eprintln!("Error initializing full node: {}", e);
                    std::process::exit(1);
                }
            };
            
            let metrics_port = port + 2;

            // Generate node_id from public key
            let node_id = hex::encode(secret_key.public_key().as_bytes());

            // Start RPC server
            let rpc_state = bitcell_node::rpc::RpcState {
                blockchain: node.blockchain.clone(),
                network: (*node.network).clone(),
                tx_pool: node.tx_pool.clone(),
                tournament_manager: Some(node.tournament_manager.clone()),
                config: node.config.clone(),
                node_type: "full_node".to_string(),
                node_id,
            };
            
            tokio::spawn(async move {
                println!("RPC server listening on 0.0.0.0:{}", rpc_port);
                if let Err(e) = bitcell_node::rpc::run_server(rpc_state, rpc_port).await {
                    eprintln!("RPC server error: {}", e);
                }
            });

            if let Err(e) = node.start_with_metrics(metrics_port).await {
                eprintln!("Error starting full node: {}", e);
                std::process::exit(1);
            }
            
            println!("Full node ready on port {}", port);
            println!("Metrics available at http://localhost:{}/metrics", metrics_port);
            println!("RPC server available at http://localhost:{}/rpc", rpc_port);
            println!("Press Ctrl+C to stop");
            
            tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
            println!("\nShutting down...");
        }
        Commands::Version => {
            println!("bitcell-node v0.1.0");
            println!("Cellular automaton tournament blockchain");
        }
    }
}
