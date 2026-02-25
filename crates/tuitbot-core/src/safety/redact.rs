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
}
