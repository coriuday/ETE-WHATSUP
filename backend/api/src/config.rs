use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // Application
    pub app_env: String,
    pub app_name: String,
    pub app_port: u16,
    pub app_url: String,
    pub frontend_url: String,

    // Database
    pub database_url: String,
    pub database_max_connections: u32,
    pub database_min_connections: u32,

    // Redis
    pub redis_url: String,
    pub redis_max_connections: u32,

    // JWT
    pub jwt_secret: String,
    pub jwt_refresh_secret: String,
    pub jwt_access_expires_secs: u64,
    pub jwt_refresh_expires_secs: u64,

    // Encryption
    pub encryption_key: String,

    // Meta WhatsApp
    pub meta_wa_token: String,
    pub meta_wa_phone_number_id: String,
    pub meta_wa_waba_id: String,
    pub meta_wa_verify_token: String,
    pub meta_api_version: String,
    pub meta_api_base_url: String,

    // Object Storage
    pub s3_endpoint: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub s3_bucket: String,
    pub s3_region: String,
    pub s3_force_path_style: bool,

    // SMTP
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_password: String,
    pub smtp_from_name: String,
    pub smtp_from_email: String,

    // n8n
    pub n8n_webhook_base_url: String,
    pub n8n_api_key: Option<String>,

    // Rate limiting
    pub rate_limit_requests_per_min: u64,
    pub rate_limit_burst: u64,

    // WhatsApp throttle
    pub wa_messages_per_minute: u64,
    pub wa_messages_per_second: u64,

    // CORS
    pub allowed_origins: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();

        let config = config::Config::builder()
            .set_default("app_env", "development")?
            .set_default("app_name", "WhatsUp API")?
            .set_default("app_port", 8080)?
            .set_default("app_url", "http://localhost:8080")?
            .set_default("frontend_url", "http://localhost:3000")?
            .set_default("database_max_connections", 10)?
            .set_default("database_min_connections", 2)?
            .set_default("redis_max_connections", 20)?
            .set_default("jwt_access_expires_secs", 900)?
            .set_default("jwt_refresh_expires_secs", 2592000)?
            .set_default("s3_force_path_style", true)?
            .set_default("smtp_port", 587)?
            .set_default("rate_limit_requests_per_min", 100)?
            .set_default("rate_limit_burst", 20)?
            .set_default("wa_messages_per_minute", 60)?
            .set_default("wa_messages_per_second", 1)?
            .set_default("meta_api_version", "v19.0")?
            .set_default("meta_api_base_url", "https://graph.facebook.com")?
            .set_default("allowed_origins", "http://localhost:3000")?
            .add_source(config::Environment::default())
            .build()
            .context("Failed to build config")?;

        config
            .try_deserialize()
            .context("Failed to deserialize config")
    }

    pub fn is_production(&self) -> bool {
        self.app_env == "production"
    }

    pub fn is_development(&self) -> bool {
        self.app_env == "development"
    }

    pub fn allowed_origins_list(&self) -> Vec<String> {
        self.allowed_origins
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    }

    pub fn meta_api_url(&self) -> String {
        format!("{}/{}", self.meta_api_base_url, self.meta_api_version)
    }
}
