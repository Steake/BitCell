//! BitCell Admin Console - Main Entry Point

use bitcell_admin::AdminConsole;
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

    tracing::info!("Admin console ready");
    tracing::info!("Dashboard available at http://{}", addr);

    console.serve().await?;

    Ok(())
}
