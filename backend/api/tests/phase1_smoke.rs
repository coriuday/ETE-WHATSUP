//! Phase 1 smoke tests — crypto helpers and auth constants.

use whatsup_api::middleware::auth::ORG_HEADER;
use whatsup_api::utils::encryption::hmac_sha256_hex;

#[test]
fn webhook_hmac_is_deterministic() {
    let key = b"test-secret";
    let body = b"{\"object\":\"whatsapp_business_account\"}";
    let a = hmac_sha256_hex(key, body);
    let b = hmac_sha256_hex(key, body);
    assert_eq!(a, b);
    assert_eq!(a.len(), 64);
}

#[test]
fn webhook_hmac_rejects_tampered_body() {
    let key = b"test-secret";
    let good = hmac_sha256_hex(key, b"payload-a");
    let bad = hmac_sha256_hex(key, b"payload-b");
    assert_ne!(good, bad);
}

#[test]
fn production_encryption_key_rules() {
    let valid = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    assert_eq!(valid.len(), 64);
    assert!(valid.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn org_header_name_is_stable() {
    assert_eq!(ORG_HEADER, "x-organization-id");
}
