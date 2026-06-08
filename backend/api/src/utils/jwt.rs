use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,           // user ID
    pub email: String,
    pub role: String,
    pub token_version: i32,
    pub exp: i64,              // expiry timestamp
    pub iat: i64,              // issued at
    pub token_type: TokenType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
}

pub fn generate_access_token(
    user_id: Uuid,
    email: &str,
    role: &str,
    token_version: i32,
    secret: &str,
    expires_secs: u64,
) -> Result<String> {
    let now = Utc::now();
    let exp = (now + Duration::seconds(expires_secs as i64)).timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        token_version,
        exp,
        iat: now.timestamp(),
        token_type: TokenType::Access,
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .context("Failed to generate access token")
}

pub fn generate_refresh_token(
    user_id: Uuid,
    email: &str,
    role: &str,
    token_version: i32,
    secret: &str,
    expires_secs: u64,
) -> Result<String> {
    let now = Utc::now();
    let exp = (now + Duration::seconds(expires_secs as i64)).timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        token_version,
        exp,
        iat: now.timestamp(),
        token_type: TokenType::Refresh,
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .context("Failed to generate refresh token")
}

pub fn verify_access_token(token: &str, secret: &str) -> Result<Claims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    let token_data: TokenData<Claims> = decode(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .context("Failed to verify access token")?;

    if token_data.claims.token_type != TokenType::Access {
        anyhow::bail!("Invalid token type");
    }

    Ok(token_data.claims)
}

pub fn verify_refresh_token(token: &str, secret: &str) -> Result<Claims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    let token_data: TokenData<Claims> = decode(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .context("Failed to verify refresh token")?;

    if token_data.claims.token_type != TokenType::Refresh {
        anyhow::bail!("Invalid token type");
    }

    Ok(token_data.claims)
}

/// Generate a cryptographically secure random token (hex string)
pub fn generate_secure_token(bytes: usize) -> String {
    use rand::Rng;
    let random_bytes: Vec<u8> = (0..bytes).map(|_| rand::thread_rng().gen()).collect();
    hex::encode(random_bytes)
}
