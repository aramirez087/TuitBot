//! Secret redaction and masking helpers for logs and user-facing diagnostics.

use std::fmt;
use std::sync::OnceLock;

use regex::{Captures, Regex};

const REDACTED: &str = "***REDACTED***";

fn bearer_token_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\bBearer\s+[^\s,;]+").expect("bearer token regex must compile")
    })
}

fn secret_kv_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(access_token|refresh_token|client_secret)\s*=\s*([^\s,&]+)")
            .expect("secret key-value regex must compile")
    })
}

/// Redact token/secret values from a string.
pub fn redact_secrets(input: &str) -> String {
    let with_bearer_redacted = bearer_token_re().replace_all(input, format!("Bearer {REDACTED}"));

    secret_kv_re()
        .replace_all(&with_bearer_redacted, |caps: &Captures<'_>| {
            format!("{}={REDACTED}", &caps[1])
        })
        .into_owned()
}

/// Wrapper that redacts secrets from anything implementing `Display`.
pub struct Redacted<T: fmt::Display>(pub T);

impl<T: fmt::Display> fmt::Display for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", redact_secrets(&self.0.to_string()))
    }
}

/// Mask a secret string for user display.
pub fn mask_secret(secret: &str) -> String {
    let char_count = secret.chars().count();

    if char_count > 8 {
        let prefix: String = secret.chars().take(4).collect();
        let suffix: String = secret
            .chars()
            .rev()
            .take(4)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        format!("{prefix}...{suffix}")
    } else if !secret.is_empty() {
        "****".to_string()
    } else {
        "(empty)".to_string()
    }
}

/// Mask an optional secret string for user display.
pub fn mask_optional_secret(secret: &Option<String>) -> String {
    match secret {
        Some(s) => mask_secret(s),
        None => "(not set)".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_bearer_token() {
        let input = "Authorization: Bearer abc123xyz";
        let output = redact_secrets(input);
        assert_eq!(output, "Authorization: Bearer ***REDACTED***");
    }

    #[test]
    fn redact_access_token_kv() {
        let input = "access_token=abc123";
        let output = redact_secrets(input);
        assert_eq!(output, "access_token=***REDACTED***");
    }

    #[test]
    fn redact_normal_text_unchanged() {
        let input = "normal text";
        let output = redact_secrets(input);
        assert_eq!(output, "normal text");
    }

    #[test]
    fn redacted_wrapper_display() {
        let wrapped = Redacted("client_secret=supersecret");
        assert_eq!(wrapped.to_string(), "client_secret=***REDACTED***");
    }

    #[test]
    fn mask_secret_long_short_and_empty() {
        assert_eq!(mask_secret("sk-1234567890abcdef"), "sk-1...cdef");
        assert_eq!(mask_secret("abc"), "****");
        assert_eq!(mask_secret(""), "(empty)");
    }

    #[test]
    fn mask_optional_secret_none() {
        assert_eq!(mask_optional_secret(&None), "(not set)");
    }

    // -----------------------------------------------------------------------
    // Additional redact coverage tests
    // -----------------------------------------------------------------------

    #[test]
    fn redact_refresh_token_kv() {
        let input = "refresh_token=xyz789abc";
        let output = redact_secrets(input);
        assert_eq!(output, "refresh_token=***REDACTED***");
    }

    #[test]
    fn redact_client_secret_kv() {
        let input = "client_secret=s3cret_value";
        let output = redact_secrets(input);
        assert_eq!(output, "client_secret=***REDACTED***");
    }

    #[test]
    fn redact_multiple_secrets_in_one_string() {
        let input = "access_token=abc123&refresh_token=xyz789";
        let output = redact_secrets(input);
        assert!(output.contains("access_token=***REDACTED***"));
        assert!(output.contains("refresh_token=***REDACTED***"));
    }

    #[test]
    fn redact_bearer_case_insensitive() {
        let input = "bearer myToken123";
        let output = redact_secrets(input);
        assert_eq!(output, "Bearer ***REDACTED***");
    }

    #[test]
    fn redact_bearer_with_prefix_text() {
        let input = "Got error with Bearer abc.def.ghi in header";
        let output = redact_secrets(input);
        assert!(output.contains("Bearer ***REDACTED***"));
        assert!(output.contains("Got error with"));
    }

    #[test]
    fn redact_kv_with_spaces_around_equals() {
        let input = "access_token = some_value";
        let output = redact_secrets(input);
        assert_eq!(output, "access_token=***REDACTED***");
    }

    #[test]
    fn redact_empty_string() {
        assert_eq!(redact_secrets(""), "");
    }

    #[test]
    fn redacted_wrapper_with_normal_text() {
        let wrapped = Redacted("just normal text");
        assert_eq!(wrapped.to_string(), "just normal text");
    }

    #[test]
    fn redacted_wrapper_with_error() {
        let err = std::io::Error::new(std::io::ErrorKind::Other, "Bearer sk-test123 failed");
        let wrapped = Redacted(err);
        let output = wrapped.to_string();
        assert!(output.contains("***REDACTED***"));
        assert!(!output.contains("sk-test123"));
    }

    #[test]
    fn mask_secret_exactly_8_chars() {
        // 8 chars: "abcdefgh" — not > 8, so returns "****"
        assert_eq!(mask_secret("abcdefgh"), "****");
    }

    #[test]
    fn mask_secret_9_chars() {
        // > 8 chars: shows prefix...suffix
        assert_eq!(mask_secret("abcdefghi"), "abcd...fghi");
    }

    #[test]
    fn mask_secret_unicode() {
        // Unicode string > 8 chars
        let secret = "sk-12345678901";
        let masked = mask_secret(secret);
        assert!(masked.starts_with("sk-1"));
        assert!(masked.ends_with("8901"));
        assert!(masked.contains("..."));
    }

    #[test]
    fn mask_optional_secret_some_short() {
        assert_eq!(mask_optional_secret(&Some("abc".to_string())), "****");
    }

    #[test]
    fn mask_optional_secret_some_long() {
        let result = mask_optional_secret(&Some("sk-1234567890".to_string()));
        assert!(result.starts_with("sk-1"));
        assert!(result.contains("..."));
    }

    #[test]
    fn mask_optional_secret_some_empty() {
        assert_eq!(mask_optional_secret(&Some(String::new())), "(empty)");
    }
}
