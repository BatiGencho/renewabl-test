use anyhow::Context;
use axum::{http::StatusCode, response::Json};
use serde_json::json;
use std::sync::Arc;
use telemetry::metrics::Telemetry;
use tower_http::{
    catch_panic::CatchPanicLayer, compression::CompressionLayer,
    trace::TraceLayer,
};
use wire_api::metrics::ServerMetrics;
use wire_api::shutdown::{ShutdownCoordinator, listen_for_shutdown_signals};

use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;

const VERSION: Option<&'static str> = option_env!("VERSION");
const MIGRATIONS: diesel_migrations::EmbeddedMigrations =
    diesel_migrations::embed_migrations!("./../../../db/migrations");

async fn fallback_handler() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": "Not Found",
            "message": "The requested endpoint does not exist",
            "status": 404
        })),
    )
}

fn main() {
    let version = VERSION.unwrap_or("unknown").to_string();
    let config = wire_api::Config::load().expect("Failed to load config");

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build tokio runtime")
        .block_on(async {
            if let Err(e) = setup(config, version).await {
                tracing::error!("Fatal error during setup: {e:#}");
                std::process::exit(1);
            }
        });
}

async fn setup(
    config: wire_api::Config,
    _version: String,
) -> anyhow::Result<()> {
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .map_err(|e| anyhow::anyhow!(e))
        .context("Failed to initialize tracing filter")?;

    let use_json = config.log_format != "pretty";

    if use_json {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_timer(UtcTime::rfc_3339())
            .with_target(true)
            .with_level(true)
            .json();
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .init();
    } else {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_level(true)
            .with_ansi(true)
            .pretty();
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .init();
    };

    let addr: String = format!("0.0.0.0:{}", config.api_service_port);
    tracing::info!("Starting wire-api service at: {addr}");

    let db_creds = config.database_credentials();
    let db_username = db_creds.username;
    let db_password = db_creds.password;
    let db_rw_endpoint = config.database_rw_endpoint.clone();
    let db_ro_endpoint = config.database_ro_endpoint.clone();

    let db_rw_url = format!(
        "postgresql://{db_username}:{db_password}@{db_rw_endpoint}:5432/wire"
    );
    let db_ro_url = format!(
        "postgresql://{db_username}:{db_password}@{db_ro_endpoint}:5432/wire"
    );

    let db_pool = postgres_models::connection::establish_connection(db_rw_url)
        .await
        .context("Failed to connect to Postgres (read-write)")?;

    let db_pool_conn = db_pool
        .get_owned()
        .await
        .context("Failed to get connection from pool for migrations")?;

    postgres_models::connection::run_migrations(db_pool_conn, MIGRATIONS)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("Failed to run database migrations")?;

    // Load energy readings from Excel into the database
    wire_api::data_loader::load_energy_readings(
        &config.energy_readings_xls_file_path,
        &db_pool,
    )
    .await
    .context("Failed to load energy readings")?;

    let read_only_pool =
        postgres_models::connection::establish_connection(db_ro_url)
            .await
            .context("Failed to connect to Postgres (read-only)")?;

    let redis_pool =
        redis_cache::connection::establish_connection(config.redis_url.clone())
            .await
            .context("Failed to connect to Redis")?;

    let shutdown = Arc::new(ShutdownCoordinator::new(
        db_pool.clone(),
        redis_pool.clone(),
    ));

    // Initialize global prom telemetry
    let metrics =
        ServerMetrics::new(None).context("Failed to create server metrics")?;
    let telemetry = Telemetry::new(Some(metrics))
        .await
        .context("Failed to create telemetry")?;
    telemetry
        .start()
        .await
        .context("Failed to start telemetry")?;
    tracing::info!("Initialized telemetry");

    let app_state = wire_api::AppState {
        telemetry,
        pool: db_pool,
        read_only_pool,
        cache_pool: redis_pool,
        config: Arc::new(config),
        shutdown: shutdown.clone(),
    };
    let app = axum::Router::new()
        .without_v07_checks()
        .route("/health", {
            let state = app_state.clone();
            axum::routing::get(move || {
                let state = state.clone();
                async move { wire_api::health::handler(state).await }
            })
        })
        .route(
            "/version",
            axum::routing::get(|| async { VERSION.unwrap_or("unknown") }),
        )
        .route("/metrics", {
            let telemetry = app_state.telemetry.clone();
            axum::routing::get(move || {
                let telemetry = telemetry.clone();
                async move {
                    (
                        axum::http::StatusCode::OK,
                        [(
                            axum::http::header::CONTENT_TYPE,
                            "text/plain; charset=utf-8",
                        )],
                        telemetry.get_metrics().await,
                    )
                }
            })
        })
        .nest(
            "/api/wire/v1",
            wire_api::get_wire_api_v1_routes(app_state.clone()),
        )
        .fallback(fallback_handler)
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(CatchPanicLayer::new())
        .merge(wire_api::get_openapi_routes());

    // Spawn shutdown signal handler
    let shutdown_handle = shutdown.clone();
    tokio::spawn(async move {
        listen_for_shutdown_signals().await;
        shutdown_handle.shutdown().await;
    });

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind to {addr}"))?;
    let shutdown_for_serve = shutdown.clone();
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_for_serve.wait_for_shutdown().await
        })
        .await
        .context("Server exited with error")?;

    Ok(())
}
