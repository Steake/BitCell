//! BitCell Admin Console - Main Entry Point

use bitcell_admin::{AdminConsole, AdminApi, api::{NodeInfo, NodeType, NodeStatus}};
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

    // Register some sample nodes for demonstration
    register_sample_nodes(&console);

    console.serve().await?;

    Ok(())
}

fn register_sample_nodes(_console: &AdminConsole) {
    let api = AdminApi::new();

    // Register sample validator nodes
    for i in 1..=3 {
        api.register_node(NodeInfo {
            id: format!("validator-{}", i),
            node_type: NodeType::Validator,
            status: if i == 1 { NodeStatus::Running } else { NodeStatus::Stopped },
            address: "127.0.0.1".to_string(),
            port: 9000 + i as u16,
            started_at: if i == 1 {
                Some(chrono::Utc::now() - chrono::Duration::hours(2))
            } else {
                None
            },
        });
    }

    // Register sample miner nodes
    for i in 1..=2 {
        api.register_node(NodeInfo {
            id: format!("miner-{}", i),
            node_type: NodeType::Miner,
            status: NodeStatus::Running,
            address: "127.0.0.1".to_string(),
            port: 9100 + i as u16,
            started_at: Some(chrono::Utc::now() - chrono::Duration::minutes(30)),
        });
    }

    tracing::info!("Registered {} sample nodes", api.list_nodes().len());
}
