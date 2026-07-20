use anyhow::Result;
use deadpool_redis::{Config as RedisConfig, Pool, Runtime};

use crate::config::Config;

pub type RedisPool = Pool;

pub async fn create_redis_pool(config: &Config) -> Result<RedisPool> {
    let mut cfg = RedisConfig::from_url(&config.redis_url);
    cfg.pool = Some(deadpool_redis::PoolConfig {
        max_size: config.redis_max_connections.max(1) as usize,
        ..Default::default()
    });

    let pool = cfg.create_pool(Some(Runtime::Tokio1))?;

    let mut conn = pool.get().await?;
    redis::cmd("PING")
        .query_async::<_, String>(&mut conn)
        .await?;

    tracing::info!(
        "Redis pool created (max_connections={})",
        config.redis_max_connections
    );
    Ok(pool)
}
