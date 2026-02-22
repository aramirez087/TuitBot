//! Error types for the ReplyGuy core library.
//!
//! Each module has its own error enum to provide clear error boundaries.
//! The library uses `thiserror` for structured, typed errors.

/// Errors related to configuration loading, parsing, and validation.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// A required configuration field is absent.
    #[error("missing required config field: {field}")]
    MissingField {
        /// The name of the missing field.
        field: String,
    },

    /// A configuration field has an unacceptable value.
    #[error("invalid value for config field '{field}': {message}")]
    InvalidValue {
        /// The name of the invalid field.
        field: String,
        /// A description of why the value is invalid.
        message: String,
    },

    /// The configuration file does not exist at the specified path.
    #[error("config file not found: {path}")]
    FileNotFound {
        /// The path that was searched.
        path: String,
    },

    /// TOML deserialization failed.
    #[error("failed to parse config file: {source}")]
    ParseError {
        /// The underlying TOML parse error.
        #[source]
        source: toml::de::Error,
    },
}

/// Errors from interacting with the X (Twitter) API.
#[derive(Debug, thiserror::Error)]
pub enum XApiError {
    /// X API returned HTTP 429 (rate limited).
    #[error("X API rate limited{}", match .retry_after {
        Some(secs) => format!(", retry after {secs}s"),
        None => String::new(),
    })]
    RateLimited {
        /// Seconds to wait before retrying, if provided by the API.
        retry_after: Option<u64>,
    },

    /// OAuth token is expired and refresh failed.
    #[error("X API authentication expired, re-authentication required")]
    AuthExpired,

    /// Account is suspended or limited.
    #[error("X API account restricted: {message}")]
    AccountRestricted {
        /// Details about the restriction.
        message: String,
    },

    /// Network-level failure communicating with X API.
    #[error("X API network error: {source}")]
    Network {
        /// The underlying HTTP client error.
        #[source]
        source: reqwest::Error,
    },

    /// Any other X API error response.
    #[error("X API error (HTTP {status}): {message}")]
    ApiError {
        /// The HTTP status code.
        status: u16,
        /// The error message from the API.
        message: String,
    },
}

/// Errors from interacting with LLM providers (OpenAI, Anthropic, Ollama).
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    /// Cannot reach the LLM endpoint.
    #[error("LLM provider '{provider}' unreachable: {source}")]
    ProviderUnreachable {
        /// The name of the LLM provider.
        provider: String,
        /// The underlying HTTP client error.
        #[source]
        source: reqwest::Error,
    },

    /// LLM provider rate limit hit.
    #[error("LLM provider '{provider}' rate limited")]
    RateLimited {
        /// The name of the LLM provider.
        provider: String,
    },

    /// LLM response could not be parsed.
    #[error("failed to parse LLM response: {message}")]
    ParseFailure {
        /// Details about the parse failure.
        message: String,
    },

    /// No LLM provider configured.
    #[error("no LLM provider configured")]
    NotConfigured,
}

/// Errors from SQLite storage operations.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// Failed to connect to SQLite database.
    #[error("database connection error: {source}")]
    Connection {
        /// The underlying SQLx error.
        #[source]
        source: sqlx::Error,
    },

    /// Database migration failed.
    #[error("database migration error: {source}")]
    Migration {
        /// The underlying migration error.
        #[source]
        source: sqlx::migrate::MigrateError,
    },

    /// A database query failed.
    #[error("database query error: {source}")]
    Query {
        /// The underlying SQLx error.
        #[source]
        source: sqlx::Error,
    },
}

/// Errors from the tweet scoring engine.
#[derive(Debug, thiserror::Error)]
pub enum ScoringError {
    /// Tweet data is missing or malformed for scoring.
    #[error("invalid tweet data for scoring: {message}")]
    InvalidTweetData {
        /// Details about what is missing or malformed.
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_error_missing_field_message() {
        let err = ConfigError::MissingField {
            field: "business.product_name".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "missing required config field: business.product_name"
        );
    }

    #[test]
    fn config_error_invalid_value_message() {
        let err = ConfigError::InvalidValue {
            field: "llm.provider".to_string(),
            message: "must be openai, anthropic, or ollama".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "invalid value for config field 'llm.provider': must be openai, anthropic, or ollama"
        );
    }

    #[test]
    fn config_error_file_not_found_message() {
        let err = ConfigError::FileNotFound {
            path: "/home/user/.replyguy/config.toml".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "config file not found: /home/user/.replyguy/config.toml"
        );
    }

    #[test]
    fn x_api_error_rate_limited_with_retry() {
        let err = XApiError::RateLimited {
            retry_after: Some(30),
        };
        assert_eq!(err.to_string(), "X API rate limited, retry after 30s");
    }

    #[test]
    fn x_api_error_rate_limited_without_retry() {
        let err = XApiError::RateLimited { retry_after: None };
        assert_eq!(err.to_string(), "X API rate limited");
    }

    #[test]
    fn x_api_error_auth_expired_message() {
        let err = XApiError::AuthExpired;
        assert_eq!(
            err.to_string(),
            "X API authentication expired, re-authentication required"
        );
    }

    #[test]
    fn x_api_error_api_error_message() {
        let err = XApiError::ApiError {
            status: 403,
            message: "Forbidden".to_string(),
        };
        assert_eq!(err.to_string(), "X API error (HTTP 403): Forbidden");
    }

    #[test]
    fn llm_error_not_configured_message() {
        let err = LlmError::NotConfigured;
        assert_eq!(err.to_string(), "no LLM provider configured");
    }

    #[test]
    fn llm_error_rate_limited_message() {
        let err = LlmError::RateLimited {
            provider: "openai".to_string(),
        };
        assert_eq!(err.to_string(), "LLM provider 'openai' rate limited");
    }

    #[test]
    fn llm_error_parse_failure_message() {
        let err = LlmError::ParseFailure {
            message: "unexpected JSON structure".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "failed to parse LLM response: unexpected JSON structure"
        );
    }

    #[test]
    fn scoring_error_invalid_tweet_data_message() {
        let err = ScoringError::InvalidTweetData {
            message: "missing author_id".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "invalid tweet data for scoring: missing author_id"
        );
    }
}
