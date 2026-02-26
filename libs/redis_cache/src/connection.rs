use deadpool_redis::Runtime;
use deadpool_redis::redis::AsyncCommands;
use std::{sync::Arc, time::Duration};
use tracing::{info, instrument};

pub type Pool = Arc<deadpool_redis::Pool>;
pub type PooledConnection = deadpool_redis::Connection;

pub async fn establish_connection(
    redis_url: String,
) -> Result<Pool, anyhow::Error> {
    let mut cfg = deadpool_redis::Config::from_url(redis_url);
    cfg.pool = Some(deadpool_redis::PoolConfig {
        max_size: 50,
        ..Default::default()
    });
    let pool = cfg.create_pool(Some(Runtime::Tokio1))?;

    // Verify the pool works by running a test query
    let mut conn = pool.get().await?;
    let _: () = conn.ping().await?;

    Ok(Arc::new(pool))
}

#[instrument(skip(pool))]
pub async fn shutdown_pool_with_timeout(pool: Pool, timeout: Duration) {
    info!("Starting graceful Redis connection pool shutdown");

    // Wait for in-flight operations to complete
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Close the pool - this will close all idle connections
    // Active connections will be closed when they are returned to the pool
    pool.close();

    info!("Redis connection pool closed");
}
