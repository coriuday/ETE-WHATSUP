use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::RngCore;

/// Encrypt plaintext using AES-256-GCM
/// Returns: base64(nonce || ciphertext)
pub fn encrypt(plaintext: &str, key_hex: &str) -> Result<String> {
    let key_bytes = hex::decode(key_hex).context("Invalid encryption key hex")?;
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(combined))
}

/// Decrypt ciphertext encrypted by `encrypt()`
pub fn decrypt(encrypted: &str, key_hex: &str) -> Result<String> {
    let key_bytes = hex::decode(key_hex).context("Invalid encryption key hex")?;
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let combined = BASE64.decode(encrypted).context("Invalid base64 ciphertext")?;
    if combined.len() < 12 {
        anyhow::bail!("Ciphertext too short");
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).context("Decrypted data is not valid UTF-8")
}

/// Hash password using bcrypt
pub fn hash_password(password: &str) -> Result<String> {
    bcrypt::hash(password, 12).context("Failed to hash password")
}

/// Verify password against bcrypt hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    bcrypt::verify(password, hash).context("Failed to verify password")
}

/// Compute SHA-256 hex digest of a string.
/// Used to hash refresh tokens before storing in the DB.
pub fn sha256_hex(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

/// Compute HMAC-SHA256 of `data` keyed with `key`, returning lowercase hex.
/// Used for Meta webhook signature verification (X-Hub-Signature-256).
pub fn hmac_sha256_hex(key: &[u8], data: &[u8]) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut mac = <Hmac<Sha256> as hmac::Mac>::new_from_slice(key)
        .expect("HMAC accepts keys of any length");
    mac.update(data);
    hex::encode(mac.finalize().into_bytes())
}
