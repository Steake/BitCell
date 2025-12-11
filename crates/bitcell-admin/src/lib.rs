//! BitCell Administrative Console
//!
//! Provides a web-based administrative interface for:
//! - Node deployment and management
//! - System monitoring and metrics
//! - Configuration management
//! - Testing utilities
//! - Log aggregation and viewing
//! - HSM integration for secure key management
//! - Testnet faucet service

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
pub mod faucet;
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
pub use faucet::FaucetService;

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
    faucet: Option<Arc<FaucetService>>,
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
        // TODO: SECURITY: Load JWT secret from environment variable or secure config
        // Current hardcoded secret is for development only and MUST be changed for production
        let jwt_secret = std::env::var("BITCELL_JWT_SECRET")
            .unwrap_or_else(|_| {
                tracing::warn!("BITCELL_JWT_SECRET not set, using default (INSECURE for production!)");
                "bitcell-admin-jwt-secret-change-in-production".to_string()
            });
        let auth = Arc::new(auth::AuthManager::new(&jwt_secret));
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
            faucet: None,
            auth,
            audit,
        }
    }

    /// Enable faucet with configuration
    pub fn with_faucet(mut self, faucet_config: faucet::FaucetConfig) -> Result<Self, String> {
        match FaucetService::new(faucet_config) {
            Ok(service) => {
                self.faucet = Some(Arc::new(service));
                Ok(self)
            }
            Err(e) => Err(format!("Failed to initialize faucet: {}", e)),
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
            .route("/api/auth/refresh", post(api::auth::refresh))
            // Faucet routes are intentionally public for testnet use
            .route("/faucet", get(web::faucet::faucet_page))
            .route("/api/faucet/request", post(api::faucet::request_tokens))
            .route("/api/faucet/info", get(api::faucet::get_info))
            .route("/api/faucet/check", post(api::faucet::check_eligibility));

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
            // Faucet history and stats require authentication (contain operational data)
            .route("/api/faucet/history", get(api::faucet::get_history))
            .route("/api/faucet/stats", get(api::faucet::get_stats))
            
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
                faucet: self.faucet.clone(),
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
    pub faucet: Option<Arc<FaucetService>>,
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
