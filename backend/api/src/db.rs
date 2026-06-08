use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

use crate::config::Config;

pub async fn create_pool(config: &Config) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .min_connections(config.database_min_connections)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(&config.database_url)
        .await?;

    tracing::info!("Database pool created successfully");
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("../../db/migrations")
        .run(pool)
        .await?;
    tracing::info!("Database migrations applied");
    Ok(())
}
