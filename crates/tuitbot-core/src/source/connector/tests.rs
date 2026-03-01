//! Integration tests for the connector module.

use super::crypto::{decrypt_credentials, encrypt_credentials, ensure_connector_key};
use super::google_drive::encrypt_refresh_token;

#[test]
fn full_encrypt_store_retrieve_decrypt_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let key = ensure_connector_key(dir.path()).unwrap();

    let refresh_token = "1//0eXample-refresh-token-value";
    let encrypted = encrypt_refresh_token(refresh_token, &key).unwrap();

    // Simulate storing and retrieving from "database" (just in-memory).
    let stored = encrypted.clone();
    let retrieved = stored;

    let decrypted = decrypt_credentials(&retrieved, &key).unwrap();
    let recovered = String::from_utf8(decrypted).unwrap();

    assert_eq!(recovered, refresh_token);
}

#[test]
fn different_keys_produce_different_ciphertext() {
    let dir1 = tempfile::tempdir().unwrap();
    let dir2 = tempfile::tempdir().unwrap();

    let key1 = ensure_connector_key(dir1.path()).unwrap();
    let key2 = ensure_connector_key(dir2.path()).unwrap();

    // Keys should be different (random).
    assert_ne!(key1, key2);

    let plaintext = b"shared-secret";
    let enc1 = encrypt_credentials(plaintext, &key1).unwrap();
    let enc2 = encrypt_credentials(plaintext, &key2).unwrap();

    // Ciphertexts should differ (different keys + nonces).
    assert_ne!(enc1, enc2);

    // Each decrypts only with its own key.
    assert_eq!(decrypt_credentials(&enc1, &key1).unwrap(), plaintext);
    assert_eq!(decrypt_credentials(&enc2, &key2).unwrap(), plaintext);
    assert!(decrypt_credentials(&enc1, &key2).is_err());
    assert!(decrypt_credentials(&enc2, &key1).is_err());
}
