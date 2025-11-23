//! BitCell Administrative Console
//!
//! Provides a web-based administrative interface for:
//! - Node deployment and management
//! - System monitoring and metrics
//! - Configuration management
//! - Testing utilities
//! - Log aggregation and viewing

pub mod api;
pub mod web;
pub mod deployment;
pub mod config;
pub mod metrics;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;

pub use api::AdminApi;
pub use deployment::DeploymentManager;
pub use config::ConfigManager;

/// Administrative console server
pub struct AdminConsole {
    addr: SocketAddr,
    api: Arc<AdminApi>,
    deployment: Arc<DeploymentManager>,
    config: Arc<ConfigManager>,
}

impl AdminConsole {
    /// Create a new admin console
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            api: Arc::new(AdminApi::new()),
            deployment: Arc::new(DeploymentManager::new()),
            config: Arc::new(ConfigManager::new()),
        }
    }

    /// Build the application router
    fn build_router(&self) -> Router {
        Router::new()
            // Dashboard
            .route("/", get(web::dashboard::index))
            .route("/dashboard", get(web::dashboard::index))

            // API endpoints
            .route("/api/nodes", get(api::nodes::list_nodes))
            .route("/api/nodes/:id", get(api::nodes::get_node))
            .route("/api/nodes/:id/start", post(api::nodes::start_node))
            .route("/api/nodes/:id/stop", post(api::nodes::stop_node))

            .route("/api/metrics", get(api::metrics::get_metrics))
            .route("/api/metrics/chain", get(api::metrics::chain_metrics))
            .route("/api/metrics/network", get(api::metrics::network_metrics))

            .route("/api/deployment/deploy", post(api::deployment::deploy_node))
            .route("/api/deployment/status", get(api::deployment::deployment_status))

            .route("/api/config", get(api::config::get_config))
            .route("/api/config", post(api::config::update_config))

            .route("/api/test/battle", post(api::test::run_battle_test))
            .route("/api/test/transaction", post(api::test::send_test_transaction))

            // Static files
            .nest_service("/static", ServeDir::new("static"))

            // CORS
            .layer(CorsLayer::permissive())

            // State
            .with_state(Arc::new(AppState {
                api: self.api.clone(),
                deployment: self.deployment.clone(),
                config: self.config.clone(),
            }))
    }

    /// Start the admin console server
    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Starting BitCell Admin Console on {}", self.addr);

        let app = self.build_router();

        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub api: Arc<AdminApi>,
    pub deployment: Arc<DeploymentManager>,
    pub config: Arc<ConfigManager>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_console_creation() {
        let addr = "127.0.0.1:8080".parse().unwrap();
        let console = AdminConsole::new(addr);
        assert_eq!(console.addr, addr);
    }
}
