//! Authentication error types.

/// Errors from authentication operations.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// The passphrase provided does not match the stored hash.
    #[error("invalid passphrase")]
    InvalidPassphrase,

    /// The session token is expired or not found.
    #[error("session expired or not found")]
    SessionExpired,

    /// Too many login attempts from this source.
    #[error("rate limited: too many login attempts")]
    RateLimited,

    /// Passphrase hash file could not be read or written.
    #[error("passphrase storage error: {message}")]
    Storage { message: String },

    /// Database error during session operations.
    #[error("session database error: {source}")]
    Database {
        #[source]
        source: sqlx::Error,
    },

    /// Bcrypt hashing failed.
    #[error("hashing error: {message}")]
    HashError { message: String },

    /// Attempted to claim an instance that already has a passphrase.
    #[error("instance already claimed")]
    AlreadyClaimed,
}
