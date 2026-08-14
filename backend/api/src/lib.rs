//! WhatsUp API library — shared types and modules for the binary and tests.

pub mod cache;
pub mod config;
pub mod db;
pub mod errors;
pub mod middleware;
pub mod models;
pub mod providers;
pub mod routes;
pub mod services;
pub mod utils;

pub use cache::RedisPool;
pub use config::Config;
pub use errors::{AppError, AppResult};

use std::sync::Arc;

use axum::{
    http::{header, HeaderName, Method},
    Router,
};
use sqlx::PgPool;
use tower_http::{
    cors::{AllowHeaders, AllowOrigin, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};

/// Global application state shared across all request handlers
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: Arc<RedisPool>,
    pub config: Arc<Config>,
    pub http: reqwest::Client,
    pub s3: aws_sdk_s3::Client,
}

impl AppState {
    pub fn new(db: PgPool, redis: RedisPool, config: Config) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        let s3 = build_s3_client(&config);

        Self {
            db,
            redis: Arc::new(redis),
            config: Arc::new(config),
            http,
            s3,
        }
    }
}

fn build_s3_client(config: &Config) -> aws_sdk_s3::Client {
    use aws_credential_types::Credentials;
    use aws_sdk_s3::config::Region;

    let creds = Credentials::new(
        &config.s3_access_key,
        &config.s3_secret_key,
        None,
        None,
        "whatsup-static",
    );

    let s3_config = aws_sdk_s3::Config::builder()
        .credentials_provider(creds)
        .region(Region::new(config.s3_region.clone()))
        .endpoint_url(&config.s3_endpoint)
        .force_path_style(config.s3_force_path_style)
        .build();

    aws_sdk_s3::Client::from_conf(s3_config)
}

pub fn build_app(state: AppState) -> Router {
    let request_id_header = HeaderName::from_static("x-request-id");

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
            HeaderName::from_static("x-organization-id"),
            request_id_header.clone(),
        ]))
        .allow_credentials(true)
        .allow_origin(AllowOrigin::list(
            state
                .config
                .allowed_origins_list()
                .iter()
                .filter_map(|o| o.parse().ok()),
        ));

    routes::create_router(state)
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::new(request_id_header.clone()))
        .layer(SetRequestIdLayer::new(
            request_id_header,
            MakeRequestUuid,
        ))
        .layer(cors)
}

#[cfg(test)]
mod phase1_unit_tests {
    use crate::middleware::auth::ORG_HEADER;
    use crate::utils::encryption::hmac_sha256_hex;

    #[test]
    fn org_header_constant() {
        assert_eq!(ORG_HEADER, "x-organization-id");
    }

    #[test]
    fn webhook_hmac_deterministic() {
        let a = hmac_sha256_hex(b"secret", b"body");
        let b = hmac_sha256_hex(b"secret", b"body");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }
}
