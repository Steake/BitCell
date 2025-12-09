//! BitCell Administrative Console
//!
//! Provides a web-based administrative interface for:
//! - Node deployment and management
//! - System monitoring and metrics
//! - Configuration management
//! - Testing utilities
//! - Log aggregation and viewing
//! - HSM integration for secure key management

pub mod api;
pub mod web;
pub mod deployment;
pub mod config;
pub mod metrics;
pub mod process;
pub mod metrics_client;
pub mod setup;
pub mod system_metrics;
pub mod hsm;
pub mod auth;
pub mod audit;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post, delete},
};
use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;

pub use api::AdminApi;
pub use deployment::DeploymentManager;
pub use config::ConfigManager;
pub use process::ProcessManager;
pub use setup::SETUP_FILE_PATH;

/// Administrative console server
pub struct AdminConsole {
    addr: SocketAddr,
    api: Arc<AdminApi>,
    deployment: Arc<DeploymentManager>,
    config: Arc<ConfigManager>,
    process: Arc<ProcessManager>,
    metrics_client: Arc<metrics_client::MetricsClient>,
    setup: Arc<setup::SetupManager>,
    system_metrics: Arc<system_metrics::SystemMetricsCollector>,
    auth: Arc<auth::AuthManager>,
    audit: Arc<audit::AuditLogger>,
}

impl AdminConsole {
    /// Create a new admin console
    pub fn new(addr: SocketAddr) -> Self {
        let process = Arc::new(ProcessManager::new());
        let setup = Arc::new(setup::SetupManager::new());
        let deployment = Arc::new(DeploymentManager::new(process.clone(), setup.clone()));
        let system_metrics = Arc::new(system_metrics::SystemMetricsCollector::new());
        
        // Initialize auth with a secret key
        // TODO: Load from environment variable or secure config
        let auth = Arc::new(auth::AuthManager::new("bitcell-admin-jwt-secret-change-in-production"));
        let audit = Arc::new(audit::AuditLogger::new());

        // Try to load setup state from default location
        let setup_path = std::path::PathBuf::from(SETUP_FILE_PATH);
        if let Err(e) = setup.load_from_file(&setup_path) {
            tracing::warn!("Failed to load setup state: {}", e);
        }

        Self {
            addr,
            api: Arc::new(AdminApi::new()),
            deployment,
            config: Arc::new(ConfigManager::new()),
            process,
            metrics_client: Arc::new(metrics_client::MetricsClient::new()),
            setup,
            system_metrics,
            auth,
            audit,
        }
    }

    /// Get the process manager
    pub fn process_manager(&self) -> Arc<ProcessManager> {
        self.process.clone()
    }

    /// Get the setup manager
    pub fn setup_manager(&self) -> Arc<setup::SetupManager> {
        self.setup.clone()
    }

    /// Build the application router
    fn build_router(&self) -> Router {
        use axum::middleware;

        // Public routes (no authentication required)
        let public_routes = Router::new()
            .route("/api/auth/login", post(api::auth::login))
            .route("/api/auth/refresh", post(api::auth::refresh));

        // Protected routes requiring authentication
        let protected_routes = Router::new()
            // Dashboard (viewer role required)
            .route("/", get(web::dashboard::index))
            .route("/dashboard", get(web::dashboard::index))

            // Read-only API endpoints (viewer role)
            .route("/api/nodes", get(api::nodes::list_nodes))
            .route("/api/nodes/:id", get(api::nodes::get_node))
            .route("/api/nodes/:id/logs", get(api::nodes::get_node_logs))
            .route("/api/metrics", get(api::metrics::get_metrics))
            .route("/api/metrics/chain", get(api::metrics::chain_metrics))
            .route("/api/metrics/network", get(api::metrics::network_metrics))
            .route("/api/metrics/system", get(api::metrics::system_metrics))
            .route("/api/deployment/status", get(api::deployment::deployment_status))
            .route("/api/config", get(api::config::get_config))
            .route("/api/setup/status", get(api::setup::get_setup_status))
            .route("/api/blocks", get(api::blocks::list_blocks))
            .route("/api/blocks/:height", get(api::blocks::get_block))
            .route("/api/blocks/:height/battles", get(api::blocks::get_block_battles))
            .route("/api/audit/logs", get(api::auth::get_audit_logs))
            
            // Operator routes (can start/stop nodes, deploy)
            .route("/api/nodes/:id/start", post(api::nodes::start_node))
            .route("/api/nodes/:id/stop", post(api::nodes::stop_node))
            .route("/api/deployment/deploy", post(api::deployment::deploy_node))
            .route("/api/test/battle", post(api::test::run_battle_test))
            .route("/api/test/battle/visualize", post(api::test::run_battle_visualization))
            .route("/api/test/transaction", post(api::test::send_test_transaction))
            .route("/api/setup/node", post(api::setup::add_node))
            .route("/api/setup/config-path", post(api::setup::set_config_path))
            .route("/api/setup/data-dir", post(api::setup::set_data_dir))
            .route("/api/setup/complete", post(api::setup::complete_setup))
            
            // Admin routes (can delete nodes, update config)
            .route("/api/nodes/:id", delete(api::nodes::delete_node))
            .route("/api/config", post(api::config::update_config))
            .route("/api/auth/users", post(api::auth::create_user))
            .route("/api/auth/logout", post(api::auth::logout))
            
            // Wallet API
            .nest("/api/wallet", api::wallet::router().with_state(self.config.clone()))
            
            // Apply auth middleware to all protected routes
            .layer(middleware::from_fn_with_state(
                self.auth.clone(),
                auth::auth_middleware,
            ));

        Router::new()
            .merge(public_routes)
            .merge(protected_routes)
            
            // Static files
            .nest_service("/static", ServeDir::new("static"))

            // CORS - WARNING: Permissive CORS allows requests from any origin.
            // This is only suitable for local development. For production,
            // configure specific allowed origins to prevent CSRF attacks.
            .layer(CorsLayer::permissive())

            // State
            .with_state(Arc::new(AppState {
                api: self.api.clone(),
                deployment: self.deployment.clone(),
                config: self.config.clone(),
                process: self.process.clone(),
                metrics_client: self.metrics_client.clone(),
                setup: self.setup.clone(),
                system_metrics: self.system_metrics.clone(),
                auth: self.auth.clone(),
                audit: self.audit.clone(),
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
    pub process: Arc<ProcessManager>,
    pub metrics_client: Arc<metrics_client::MetricsClient>,
    pub setup: Arc<setup::SetupManager>,
    pub system_metrics: Arc<system_metrics::SystemMetricsCollector>,
    pub auth: Arc<auth::AuthManager>,
    pub audit: Arc<audit::AuditLogger>,
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
