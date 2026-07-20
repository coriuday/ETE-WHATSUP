use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use whatsup_api::{
    cache, db, services,
    AppState, Config, build_app,
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "whatsup_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::load()?;
    tracing::info!("Starting {} in {} mode", config.app_name, config.app_env);

    let db = db::create_pool(&config).await?;
    tracing::info!("Connected to PostgreSQL");
    // Migrations are applied externally (Compose init / CI / ops). See RUNBOOK.md.

    let redis = cache::create_redis_pool(&config).await?;
    tracing::info!("Connected to Redis");

    let port = config.app_port;
    let state = AppState::new(db, redis, config);

    let recovery_state = state.clone();
    tokio::spawn(async move {
        services::worker::recover_running_campaigns(recovery_state).await;
    });

    let reclaim_state = state.clone();
    tokio::spawn(async move {
        services::worker::run_job_reclaimer(reclaim_state).await;
    });

    let scheduler_state = state.clone();
    tokio::spawn(async move {
        services::scheduler::run_scheduler(scheduler_state).await;
    });
    tracing::info!("Campaign scheduler daemon started");

    let router = build_app(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("WhatsUp API listening on http://{}", addr);

    axum::serve(listener, router.into_make_service()).await?;

    Ok(())
}
