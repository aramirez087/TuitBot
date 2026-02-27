//! Passphrase generation and verification.
//!
//! Generates 4-word passphrases from the EFF short wordlist (1,296 words).
//! Passphrases are hashed with bcrypt before storage.

use std::path::Path;

use rand::seq::SliceRandom;

use super::error::AuthError;

/// EFF short wordlist (1,296 words), one per line.
const WORDLIST: &str = include_str!("../../assets/eff_short_wordlist.txt");

/// Number of words in the generated passphrase.
const PASSPHRASE_WORD_COUNT: usize = 4;

/// Bcrypt cost factor (12 = ~250ms on modern hardware).
const BCRYPT_COST: u32 = 12;

/// Generate a random 4-word passphrase from the EFF short wordlist.
pub fn generate_passphrase() -> String {
    let words: Vec<&str> = WORDLIST.lines().filter(|l| !l.is_empty()).collect();
    let mut rng = rand::thread_rng();
    let selected: Vec<&&str> = words
        .choose_multiple(&mut rng, PASSPHRASE_WORD_COUNT)
        .collect();
    selected
        .into_iter()
        .copied()
        .collect::<Vec<&str>>()
        .join(" ")
}

/// Hash a passphrase with bcrypt.
pub fn hash_passphrase(passphrase: &str) -> Result<String, AuthError> {
    bcrypt::hash(passphrase, BCRYPT_COST).map_err(|e| AuthError::HashError {
        message: e.to_string(),
    })
}

/// Verify a passphrase against a bcrypt hash.
pub fn verify_passphrase(passphrase: &str, hash: &str) -> Result<bool, AuthError> {
    bcrypt::verify(passphrase, hash).map_err(|e| AuthError::HashError {
        message: e.to_string(),
    })
}

/// Ensure a passphrase hash file exists in the data directory.
///
/// If the file doesn't exist, generates a new passphrase, hashes it, writes
/// the hash to `passphrase_hash`, and returns `Ok(Some(plaintext))` so the
/// caller can print it to the terminal.
///
/// If the file already exists, returns `Ok(None)`.
pub fn ensure_passphrase(data_dir: &Path) -> Result<Option<String>, AuthError> {
    let hash_path = data_dir.join("passphrase_hash");

    if hash_path.exists() {
        let existing = std::fs::read_to_string(&hash_path).map_err(|e| AuthError::Storage {
            message: format!("failed to read passphrase hash: {e}"),
        })?;
        if !existing.trim().is_empty() {
            return Ok(None);
        }
    }

    let passphrase = generate_passphrase();
    let hash = hash_passphrase(&passphrase)?;

    std::fs::create_dir_all(data_dir).map_err(|e| AuthError::Storage {
        message: format!("failed to create data directory: {e}"),
    })?;
    std::fs::write(&hash_path, &hash).map_err(|e| AuthError::Storage {
        message: format!("failed to write passphrase hash: {e}"),
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&hash_path, std::fs::Permissions::from_mode(0o600));
    }

    Ok(Some(passphrase))
}

/// Load the passphrase hash from disk. Returns `None` if the file doesn't exist.
pub fn load_passphrase_hash(data_dir: &Path) -> Result<Option<String>, AuthError> {
    let hash_path = data_dir.join("passphrase_hash");
    if !hash_path.exists() {
        return Ok(None);
    }
    let hash = std::fs::read_to_string(&hash_path).map_err(|e| AuthError::Storage {
        message: format!("failed to read passphrase hash: {e}"),
    })?;
    let trimmed = hash.trim().to_string();
    if trimmed.is_empty() {
        return Ok(None);
    }
    Ok(Some(trimmed))
}

/// Re-generate a passphrase (for `--reset-passphrase`).
///
/// Overwrites the existing hash file and returns the new plaintext passphrase.
pub fn reset_passphrase(data_dir: &Path) -> Result<String, AuthError> {
    let hash_path = data_dir.join("passphrase_hash");
    let passphrase = generate_passphrase();
    let hash = hash_passphrase(&passphrase)?;

    std::fs::write(&hash_path, &hash).map_err(|e| AuthError::Storage {
        message: format!("failed to write passphrase hash: {e}"),
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&hash_path, std::fs::Permissions::from_mode(0o600));
    }

    Ok(passphrase)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_passphrase_has_four_words() {
        let passphrase = generate_passphrase();
        let words: Vec<&str> = passphrase.split_whitespace().collect();
        assert_eq!(words.len(), 4);
    }

    #[test]
    fn generate_passphrase_uses_wordlist_words() {
        let valid_words: Vec<&str> = WORDLIST.lines().filter(|l| !l.is_empty()).collect();
        let passphrase = generate_passphrase();
        for word in passphrase.split_whitespace() {
            assert!(valid_words.contains(&word), "word not in wordlist: {word}");
        }
    }

    #[test]
    fn hash_and_verify_roundtrip() {
        let passphrase = "alpha bravo charlie delta";
        let hash = hash_passphrase(passphrase).unwrap();
        assert!(verify_passphrase(passphrase, &hash).unwrap());
        assert!(!verify_passphrase("wrong passphrase here", &hash).unwrap());
    }

    #[test]
    fn ensure_passphrase_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let result = ensure_passphrase(dir.path()).unwrap();
        assert!(result.is_some());
        let passphrase = result.unwrap();
        assert_eq!(passphrase.split_whitespace().count(), 4);

        // Second call should return None (already exists)
        let result2 = ensure_passphrase(dir.path()).unwrap();
        assert!(result2.is_none());
    }

    #[test]
    fn ensure_passphrase_verifies_against_hash() {
        let dir = tempfile::tempdir().unwrap();
        let passphrase = ensure_passphrase(dir.path()).unwrap().unwrap();
        let hash = load_passphrase_hash(dir.path()).unwrap().unwrap();
        assert!(verify_passphrase(&passphrase, &hash).unwrap());
    }

    #[test]
    fn reset_passphrase_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        let first = ensure_passphrase(dir.path()).unwrap().unwrap();
        let second = reset_passphrase(dir.path()).unwrap();
        assert_ne!(first, second);
        let hash = load_passphrase_hash(dir.path()).unwrap().unwrap();
        assert!(verify_passphrase(&second, &hash).unwrap());
        assert!(!verify_passphrase(&first, &hash).unwrap());
    }
}
