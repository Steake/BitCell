//! BitCell Admin Console - Main Entry Point

use bitcell_admin::{AdminConsole, process::{ProcessManager, NodeConfig}, api::NodeType};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bitcell_admin=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🚀 Starting BitCell Admin Console");

    // Parse command line arguments
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string())
        .parse()?;

    let console = AdminConsole::new(addr);
    let process_mgr = console.process_manager();

    // Register some sample nodes for demonstration
    register_sample_nodes(&process_mgr);

    tracing::info!("Admin console ready - registered {} nodes", process_mgr.list_nodes().len());
    tracing::info!("Dashboard available at http://{}", addr);

    console.serve().await?;

    Ok(())
}

fn register_sample_nodes(process: &ProcessManager) {
    // Register sample validator nodes
    for i in 1..=3 {
        let config = NodeConfig {
            node_type: NodeType::Validator,
            data_dir: format!("/tmp/bitcell/validator-{}", i),
            port: 9000 + i as u16,
            rpc_port: 10000 + i as u16,
            log_level: "info".to_string(),
            network: "testnet".to_string(),
        };

        process.register_node(format!("validator-{}", i), config);
        tracing::info!("Registered validator-{}", i);
    }

    // Register sample miner nodes
    for i in 1..=2 {
        let config = NodeConfig {
            node_type: NodeType::Miner,
            data_dir: format!("/tmp/bitcell/miner-{}", i),
            port: 9100 + i as u16,
            rpc_port: 10100 + i as u16,
            log_level: "info".to_string(),
            network: "testnet".to_string(),
        };

        process.register_node(format!("miner-{}", i), config);
        tracing::info!("Registered miner-{}", i);
    }
}
