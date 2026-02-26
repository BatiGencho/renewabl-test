//! # Wire API Server
//!
use crate::metrics::ServerMetrics;
use crate::shutdown::ShutdownCoordinator;
use std::sync::Arc;
use telemetry::metrics::Telemetry;
// Private API modules - internal implementation details
pub mod data_loader;
pub mod shutdown;
mod wire_api;

// OpenAPI documentation module
pub mod openapi;

// Public modules - shared utilities and middleware
// These provide common functionality that can be used across the application
pub mod health;
pub mod metrics;
pub mod shared;

// Public API surface - only expose route registration functions
// This provides a clean API boundary where external code can only access
// the route registration functions without depending on internal module structure

pub use wire_api::core::v1::get_routes as get_wire_api_v1_routes;

/// Returns the OpenAPI documentation routes for Wire v1 API
/// Includes Swagger UI and OpenAPI JSON spec with OpenAPI 3.0 compatibility fixes
pub fn get_openapi_routes() -> axum::Router {
    use axum::Json;
    use axum::routing::get;
    use utoipa_swagger_ui::SwaggerUi;

    // Custom handler that serves OpenAPI 3.0 compatible JSON (for Mintlify)
    // Converts type: ["array", "null"] -> type: "array", nullable: true
    async fn openapi_3_0_handler() -> Json<serde_json::Value> {
        Json(openapi::WireV1ApiDoc::openapi_json())
    }

    axum::Router::new()
        .without_v07_checks()
        // OpenAPI 3.0 format with nullable array fixes
        // Keep as /api-docs/openapi.json for backward compatibility
        .route("/api-docs/openapi.json", get(openapi_3_0_handler))
        // SwaggerUI: Creates /api-docs/openapi-3.1.json serving native OpenAPI 3.1 spec
        // SwaggerUI handles type: ["array", "null"] correctly, so no conversion needed
        .merge(SwaggerUi::new("/swagger-ui").url(
            "/api-docs/openapi-3.1.json",
            openapi::WireV1ApiDoc::openapi(),
        ))
}

#[derive(Clone)]
pub struct AppState {
    pub telemetry: Arc<Telemetry<ServerMetrics>>,
    pub pool: postgres_models::connection::Pool,
    pub read_only_pool: postgres_models::connection::Pool,
    pub cache_pool: redis_cache::connection::Pool,
    pub config: Arc<Config>,
    pub shutdown: Arc<ShutdownCoordinator>,
}

impl AppState {}

impl axum::extract::FromRef<AppState> for postgres_models::connection::Pool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

#[derive(serde::Deserialize)]
pub struct Config {
    // Service port
    pub api_service_port: String,

    // Loggers
    pub rust_log: String,
    #[serde(default)]
    pub log_format: String,

    // Db configs
    pub database_credentials: String,
    pub database_rw_endpoint: String,
    pub database_ro_endpoint: String,

    // Redis configs
    pub redis_url: String,

    // Energy readings Excel file path
    pub energy_readings_xls_file_path: String,
}

impl Config {
    pub fn load() -> Result<Self, envy::Error> {
        // Load .env file if present (useful when running outside docker-compose)
        match dotenv::dotenv() {
            Ok(path) => eprintln!("Loaded .env from: {}", path.display()),
            Err(e) => eprintln!("dotenv warning: {e}"),
        }

        envy::from_env::<Config>()
    }

    pub fn database_credentials(
        &self,
    ) -> postgres_models::connection::Credentials {
        serde_json::from_str::<postgres_models::connection::Credentials>(
            self.database_credentials.as_str(),
        )
        .expect("creds must be valid")
    }
}
