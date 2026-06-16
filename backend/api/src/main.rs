use std::sync::Arc;

use anyhow::Result;
use axum::{
    http::{header, Method},
    Router,
};
use sqlx::PgPool;
use tower_http::{
    cors::{AllowHeaders, AllowOrigin, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cache;
mod config;
mod db;
mod errors;
mod middleware;
mod models;
mod routes;
mod services;
mod utils;

pub use config::Config;
pub use cache::RedisPool;

/// Global application state shared across all request handlers
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: Arc<RedisPool>,
    pub config: Arc<Config>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // ── Initialize tracing ─────────────────────────────────────
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "whatsup_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // ── Load config ────────────────────────────────────────────
    let config = Config::load()?;
    tracing::info!("Starting {} in {} mode", config.app_name, config.app_env);

    // ── Connect to database ────────────────────────────────────
    let db = db::create_pool(&config).await?;
    tracing::info!("Connected to Supabase PostgreSQL");
    // db::run_migrations(&db).await?;

    // ── Connect to Redis ───────────────────────────────────────
    let redis = cache::create_redis_pool(&config).await?;
    tracing::info!("Connected to Redis");

    // ── Build app state ────────────────────────────────────────
    let state = AppState {
        db,
        redis: Arc::new(redis),
        config: Arc::new(config.clone()),
    };

    // ── Build CORS layer ───────────────────────────────────────
    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(AllowHeaders::list([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
        ]))
        .allow_credentials(true)
        .allow_origin(AllowOrigin::list(
            config
                .allowed_origins_list()
                .iter()
                .filter_map(|o| o.parse().ok()),
        ));

    // ── Build router ───────────────────────────────────────────
    let router = routes::create_router(state.clone())
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    // ── Start scheduler daemon ─────────────────────────────────
    let scheduler_state = state.clone();
    tokio::spawn(async move {
        services::scheduler::run_scheduler(scheduler_state).await;
    });
    tracing::info!("Campaign scheduler daemon started");

    // ── Start server ───────────────────────────────────────────
    let addr = format!("0.0.0.0:{}", config.app_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("🚀 WhatsUp API listening on http://{}", addr);

    axum::serve(listener, router.into_make_service()).await?;

    Ok(())
}
