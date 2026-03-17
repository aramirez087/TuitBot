//! Retry helper for transient X API / scraper errors.
//!
//! Provides `retry_with_backoff` — an async wrapper that retries a fallible
//! async operation using exponential backoff with full jitter.
//!
//! Use for scraper mutations and queries where transient network or 5xx
//! errors are expected.  Never retry non-retryable errors (401, 403, etc.).

use std::time::Duration;

use rand::Rng;

use crate::error::XApiError;

/// Configuration for the retry policy.
#[derive(Debug, Clone, Copy)]
pub struct RetryConfig {
    /// Maximum number of attempts (including the first).  Default: 3.
    pub max_attempts: u32,
    /// Base delay before the first retry.  Default: 500 ms.
    pub base_delay: Duration,
    /// Maximum delay cap (jitter stays within `[0, capped_delay]`).  Default: 8 s.
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(8),
        }
    }
}

/// Retry `op` up to `cfg.max_attempts` times on retryable errors.
///
/// Delay between attempts uses exponential backoff with full jitter:
/// `sleep(rand(0, min(base * 2^attempt, max_delay)))`.
///
/// Returns the last error unchanged if all attempts are exhausted or
/// the error is non-retryable.
pub async fn retry_with_backoff<F, Fut, T>(cfg: RetryConfig, mut op: F) -> Result<T, XApiError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, XApiError>>,
{
    let mut attempt = 0u32;
    loop {
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) if !e.is_retryable() => return Err(e),
            Err(e) => {
                attempt += 1;
                if attempt >= cfg.max_attempts {
                    return Err(e);
                }

                // Exponential backoff with full jitter.
                let cap_ms = cfg
                    .max_delay
                    .min(cfg.base_delay * 2u32.saturating_pow(attempt))
                    .as_millis() as u64;
                let jitter_ms = rand::rng().random_range(0..=cap_ms);
                let delay = Duration::from_millis(jitter_ms);

                tracing::debug!(
                    attempt,
                    delay_ms = jitter_ms,
                    error = %e,
                    "Retryable scraper error — backing off before retry"
                );

                tokio::time::sleep(delay).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = RetryConfig::default();
        assert_eq!(cfg.max_attempts, 3);
        assert_eq!(cfg.base_delay, Duration::from_millis(500));
        assert_eq!(cfg.max_delay, Duration::from_secs(8));
    }

    #[tokio::test]
    async fn succeeds_on_first_attempt() {
        let mut calls = 0u32;
        let result = retry_with_backoff(RetryConfig::default(), || {
            calls += 1;
            async { Ok::<_, XApiError>(42u32) }
        })
        .await;
        assert_eq!(result.unwrap(), 42);
        assert_eq!(calls, 1);
    }

    #[tokio::test]
    async fn does_not_retry_non_retryable_error() {
        let mut calls = 0u32;
        let cfg = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(2),
        };
        let result = retry_with_backoff(cfg, || {
            calls += 1;
            async { Err::<u32, _>(XApiError::AuthExpired) }
        })
        .await;
        assert!(matches!(result, Err(XApiError::AuthExpired)));
        // Must not retry on non-retryable.
        assert_eq!(calls, 1);
    }

    #[tokio::test]
    async fn retries_retryable_error_up_to_max() {
        let mut calls = 0u32;
        let cfg = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(2),
        };
        let result = retry_with_backoff(cfg, || {
            calls += 1;
            async {
                Err::<u32, _>(XApiError::ScraperTransportUnavailable {
                    message: "timeout".to_string(),
                })
            }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(calls, 3, "should attempt exactly max_attempts times");
    }

    #[tokio::test]
    async fn succeeds_on_retry_after_transient_failure() {
        use std::sync::{Arc, Mutex};
        let calls = Arc::new(Mutex::new(0u32));
        let cfg = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(2),
        };
        let calls_clone = calls.clone();
        let result = retry_with_backoff(cfg, move || {
            let c = calls_clone.clone();
            async move {
                let mut n = c.lock().unwrap();
                *n += 1;
                if *n < 2 {
                    Err(XApiError::ScraperTransportUnavailable {
                        message: "transient".to_string(),
                    })
                } else {
                    Ok(99u32)
                }
            }
        })
        .await;
        assert_eq!(result.unwrap(), 99);
        assert_eq!(*calls.lock().unwrap(), 2);
    }
}
