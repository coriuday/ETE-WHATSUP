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
