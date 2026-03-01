//! AES-256-GCM encryption for connector credentials.
//!
//! Each Tuitbot instance has a random 32-byte key stored at
//! `<data_dir>/connector_key` with 0600 permissions. The key is
//! generated once and reused across all connections.
//!
//! Ciphertext format: `nonce(12) || ciphertext(N) || tag(16)`.

use std::path::Path;

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm};

use super::ConnectorError;

const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;
const KEY_FILENAME: &str = "connector_key";

/// Ensure a connector encryption key exists at `<data_dir>/connector_key`.
///
/// Reads the key if it exists, otherwise generates 32 random bytes and
/// writes them with 0600 permissions. Returns the key bytes.
pub fn ensure_connector_key(data_dir: &Path) -> Result<Vec<u8>, ConnectorError> {
    let key_path = data_dir.join(KEY_FILENAME);

    if key_path.exists() {
        let key = std::fs::read(&key_path).map_err(|e| {
            ConnectorError::Encryption(format!("failed to read connector key: {e}"))
        })?;
        if key.len() != KEY_LEN {
            return Err(ConnectorError::Encryption(format!(
                "connector key has invalid length {} (expected {KEY_LEN})",
                key.len()
            )));
        }
        return Ok(key);
    }

    // Generate a new random key.
    let key: Vec<u8> = (0..KEY_LEN).map(|_| rand::random::<u8>()).collect();

    // Write with restricted permissions.
    write_key_file(&key_path, &key)?;

    Ok(key)
}

/// Write key bytes to a file with 0600 permissions (Unix) or default (other).
fn write_key_file(path: &Path, key: &[u8]) -> Result<(), ConnectorError> {
    std::fs::write(path, key)
        .map_err(|e| ConnectorError::Encryption(format!("failed to write connector key: {e}")))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(path, perms).map_err(|e| {
            ConnectorError::Encryption(format!("failed to set key permissions: {e}"))
        })?;
    }

    Ok(())
}

/// Encrypt plaintext with AES-256-GCM.
///
/// Returns `nonce(12) || ciphertext_with_tag`.
pub fn encrypt_credentials(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, ConnectorError> {
    if key.len() != KEY_LEN {
        return Err(ConnectorError::Encryption(format!(
            "key length {} != {KEY_LEN}",
            key.len()
        )));
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| ConnectorError::Encryption(format!("cipher init failed: {e}")))?;

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| ConnectorError::Encryption(format!("encryption failed: {e}")))?;

    let mut blob = nonce.to_vec();
    blob.extend_from_slice(&ciphertext);
    Ok(blob)
}

/// Decrypt a blob produced by `encrypt_credentials`.
///
/// Expects `nonce(12) || ciphertext_with_tag`.
pub fn decrypt_credentials(blob: &[u8], key: &[u8]) -> Result<Vec<u8>, ConnectorError> {
    if key.len() != KEY_LEN {
        return Err(ConnectorError::Encryption(format!(
            "key length {} != {KEY_LEN}",
            key.len()
        )));
    }
    if blob.len() < NONCE_LEN + 16 {
        return Err(ConnectorError::Encryption(
            "ciphertext too short".to_string(),
        ));
    }

    let (nonce_bytes, ciphertext) = blob.split_at(NONCE_LEN);
    let nonce = aes_gcm::Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| ConnectorError::Encryption(format!("cipher init failed: {e}")))?;

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| ConnectorError::Encryption(format!("decryption failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_encrypt_decrypt() {
        let key: Vec<u8> = (0..32).collect();
        let plaintext = b"my-secret-refresh-token";

        let blob = encrypt_credentials(plaintext, &key).unwrap();
        let decrypted = decrypt_credentials(&blob, &key).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn decrypt_with_wrong_key_fails() {
        let key1: Vec<u8> = (0..32).collect();
        let key2: Vec<u8> = (32..64).collect();
        let plaintext = b"secret-token";

        let blob = encrypt_credentials(plaintext, &key1).unwrap();
        let result = decrypt_credentials(&blob, &key2);

        assert!(result.is_err());
    }

    #[test]
    fn corrupt_ciphertext_fails() {
        let key: Vec<u8> = (0..32).collect();
        let plaintext = b"secret-token";

        let mut blob = encrypt_credentials(plaintext, &key).unwrap();
        // Flip a byte in the ciphertext region.
        let last = blob.len() - 1;
        blob[last] ^= 0xFF;

        let result = decrypt_credentials(&blob, &key);
        assert!(result.is_err());
    }

    #[test]
    fn short_blob_fails() {
        let key: Vec<u8> = (0..32).collect();
        let result = decrypt_credentials(&[0u8; 10], &key);
        assert!(result.is_err());
    }

    #[test]
    fn wrong_key_length_fails() {
        let result = encrypt_credentials(b"data", &[0u8; 16]);
        assert!(result.is_err());

        let result = decrypt_credentials(&[0u8; 30], &[0u8; 16]);
        assert!(result.is_err());
    }

    #[test]
    fn ensure_connector_key_creates_and_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let key1 = ensure_connector_key(dir.path()).unwrap();
        assert_eq!(key1.len(), KEY_LEN);

        let key2 = ensure_connector_key(dir.path()).unwrap();
        assert_eq!(key1, key2, "key should be idempotent");
    }

    #[cfg(unix)]
    #[test]
    fn connector_key_has_restricted_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let _ = ensure_connector_key(dir.path()).unwrap();

        let key_path = dir.path().join(KEY_FILENAME);
        let perms = std::fs::metadata(&key_path).unwrap().permissions();
        assert_eq!(perms.mode() & 0o777, 0o600);
    }
}
