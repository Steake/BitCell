//! BitCell node binary

use bitcell_node::{NodeConfig, ValidatorNode, MinerNode};
use bitcell_crypto::SecretKey;
use clap::{Parser, Subcommand};

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
    },
    /// Run as miner
    Miner {
        #[arg(short, long, default_value_t = 30333)]
        port: u16,
    },
    /// Show version
    Version,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validator { port } => {
            println!("🌌 BitCell Validator Node");
            println!("=========================");
            
            let mut config = NodeConfig::default();
            config.network_port = port;
            
            let mut node = ValidatorNode::new(config);
            
            if let Err(e) = node.start().await {
                eprintln!("Error starting validator: {}", e);
                std::process::exit(1);
            }
            
            println!("Validator ready on port {}", port);
            println!("Press Ctrl+C to stop");
            
            // Keep running
            tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
            println!("\nShutting down...");
        }
        Commands::Miner { port } => {
            println!("🎮 BitCell Miner Node");
            println!("=====================");
            
            let mut config = NodeConfig::default();
            config.network_port = port;
            
            let sk = SecretKey::generate();
            println!("Public key: {:?}", sk.public_key());
            
            let mut node = MinerNode::new(config, sk);
            
            if let Err(e) = node.start().await {
                eprintln!("Error starting miner: {}", e);
                std::process::exit(1);
            }
            
            println!("Miner ready on port {}", port);
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
