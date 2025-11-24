//! BitCell node binary

use bitcell_node::{NodeConfig, ValidatorNode, MinerNode};
use bitcell_crypto::SecretKey;
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
    },
    /// Run as miner
    Miner {
        #[arg(short, long, default_value_t = 30333)]
        port: u16,
        #[arg(long, default_value_t = 30334)]
        rpc_port: u16,
        #[arg(long)]
        data_dir: Option<PathBuf>,
    },
    /// Run as full node
    FullNode {
        #[arg(short, long, default_value_t = 30333)]
        port: u16,
        #[arg(long, default_value_t = 30334)]
        rpc_port: u16,
        #[arg(long)]
        data_dir: Option<PathBuf>,
    },
    /// Show version
    Version,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validator { port, rpc_port: _, data_dir: _ } => {
            println!("🌌 BitCell Validator Node");
            println!("=========================");
            
            let mut config = NodeConfig::default();
            config.network_port = port;
            // TODO: Use rpc_port and data_dir
            
            let mut node = ValidatorNode::new(config);
            
            // Start metrics server on port + 1 to avoid conflict with P2P port
            let metrics_port = port + 1;
            
            // We need to pass the metrics port to the node start
            if let Err(e) = node.start_with_metrics(metrics_port).await {
                eprintln!("Error starting validator: {}", e);
                std::process::exit(1);
            }
            
            println!("Validator ready on port {}", port);
            println!("Metrics available at http://localhost:{}/metrics", metrics_port);
            println!("Press Ctrl+C to stop");
            
            // Keep running
            tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
            println!("\nShutting down...");
        }
        Commands::Miner { port, rpc_port: _, data_dir: _ } => {
            println!("🎮 BitCell Miner Node");
            println!("=====================");
            
            let mut config = NodeConfig::default();
            config.network_port = port;
            
            let sk = SecretKey::generate();
            println!("Public key: {:?}", sk.public_key());
            
            let mut node = MinerNode::new(config, sk);
            
            let metrics_port = port + 1;

            if let Err(e) = node.start_with_metrics(metrics_port).await {
                eprintln!("Error starting miner: {}", e);
                std::process::exit(1);
            }
            
            println!("Miner ready on port {}", port);
            println!("Metrics available at http://localhost:{}/metrics", metrics_port);
            println!("Press Ctrl+C to stop");
            
            tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
            println!("\nShutting down...");
        }
        Commands::FullNode { port, rpc_port: _, data_dir: _ } => {
            println!("🌍 BitCell Full Node");
            println!("====================");
            
            let mut config = NodeConfig::default();
            config.network_port = port;
            
            // Reuse ValidatorNode for now as FullNode logic is similar (just no voting)
            let mut node = ValidatorNode::new(config);
            
            let metrics_port = port + 1;

            if let Err(e) = node.start_with_metrics(metrics_port).await {
                eprintln!("Error starting full node: {}", e);
                std::process::exit(1);
            }
            
            println!("Full node ready on port {}", port);
            println!("Metrics available at http://localhost:{}/metrics", metrics_port);
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
