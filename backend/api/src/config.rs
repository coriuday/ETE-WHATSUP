use anyhow::{bail, Context, Result};
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
    pub meta_wa_app_secret: String,
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
    #[serde(default = "default_auth_rate_limit")]
    pub rate_limit_auth_per_min: u64,
    #[serde(default = "default_webhook_rate_limit")]
    pub rate_limit_webhook_per_min: u64,

    // WhatsApp throttle
    pub wa_messages_per_minute: u64,
    pub wa_messages_per_second: u64,

    // Worker
    #[serde(default = "default_job_reclaim_mins")]
    pub job_reclaim_minutes: u64,

    // CORS
    pub allowed_origins: String,

    #[serde(default = "default_messaging_provider")]
    pub messaging_provider: String,
    #[serde(default = "default_enable_mock")]
    pub enable_mock_provider: bool,
    #[serde(default)]
    pub mock_failure_rate: f64,
}

fn default_auth_rate_limit() -> u64 {
    5
}
fn default_webhook_rate_limit() -> u64 {
    1000
}
fn default_job_reclaim_mins() -> u64 {
    10
}
fn default_messaging_provider() -> String {
    "mock".into()
}
fn default_enable_mock() -> bool {
    true
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
            .set_default("rate_limit_requests_per_min", 300)?
            .set_default("rate_limit_burst", 20)?
            .set_default("rate_limit_auth_per_min", 5)?
            .set_default("rate_limit_webhook_per_min", 1000)?
            .set_default("wa_messages_per_minute", 60)?
            .set_default("wa_messages_per_second", 1)?
            .set_default("job_reclaim_minutes", 10)?
            .set_default("meta_wa_app_secret", "")?
            .set_default("meta_api_version", "v19.0")?
            .set_default("meta_api_base_url", "https://graph.facebook.com")?
            .set_default("allowed_origins", "http://localhost:3000")?
            .set_default("messaging_provider", "mock")?
            .set_default("enable_mock_provider", true)?
            .set_default("mock_failure_rate", 0.0)?
            .add_source(config::Environment::default())
            .build()
            .context("Failed to build config")?;

        let cfg: Self = config
            .try_deserialize()
            .context("Failed to deserialize config")?;

        cfg.validate()?;
        Ok(cfg)
    }

    /// Fail closed in production; warn in development for optional integrations.
    pub fn validate(&self) -> Result<()> {
        if self.database_url.is_empty() {
            bail!("DATABASE_URL is required");
        }
        if self.redis_url.is_empty() {
            bail!("REDIS_URL is required");
        }

        if self.is_production() {
            if self.jwt_secret.len() < 32 {
                bail!("JWT_SECRET must be at least 32 characters in production");
            }
            if self.jwt_refresh_secret.len() < 32 {
                bail!("JWT_REFRESH_SECRET must be at least 32 characters in production");
            }
            if self.jwt_secret == self.jwt_refresh_secret {
                bail!("JWT_SECRET and JWT_REFRESH_SECRET must be distinct in production");
            }
            if !is_valid_encryption_key(&self.encryption_key) {
                bail!("ENCRYPTION_KEY must be a 64-char hex string (32 bytes) in production");
            }
            if self.meta_wa_verify_token.is_empty() {
                bail!("META_WA_VERIFY_TOKEN is required in production");
            }
        } else {
            if self.jwt_secret.len() < 16 {
                tracing::warn!("JWT_SECRET is short; use >= 32 chars before production");
            }
            if self.jwt_secret == self.jwt_refresh_secret {
                tracing::warn!("JWT_SECRET and JWT_REFRESH_SECRET should be distinct");
            }
            if !is_valid_encryption_key(&self.encryption_key) {
                tracing::warn!(
                    "ENCRYPTION_KEY should be 64-char hex (32 bytes); current length={}",
                    self.encryption_key.len()
                );
            }
            if self.meta_wa_token.is_empty() {
                tracing::warn!("META_WA_TOKEN is empty — live Meta sends disabled; mock provider will be used");
            }
            if self.smtp_host.is_empty() {
                tracing::warn!("SMTP_HOST is empty — email delivery disabled");
            }
            if self.s3_endpoint.is_empty() {
                tracing::warn!("S3_ENDPOINT is empty — object storage disabled");
            }
        }

        Ok(())
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

fn is_valid_encryption_key(key: &str) -> bool {
    key.len() == 64 && key.chars().all(|c| c.is_ascii_hexdigit())
}
