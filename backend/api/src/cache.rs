use anyhow::Result;
use deadpool_redis::{Config as RedisConfig, Pool, Runtime};

use crate::config::Config;

pub type RedisPool = Pool;

pub async fn create_redis_pool(config: &Config) -> Result<RedisPool> {
    let cfg = RedisConfig::from_url(&config.redis_url);
    let pool = cfg.create_pool(Some(Runtime::Tokio1))?;

    // Test connection
    let mut conn = pool.get().await?;
    redis::cmd("PING")
        .query_async::<_, String>(&mut conn)
        .await?;

    tracing::info!("Redis pool created successfully");
    Ok(pool)
}
