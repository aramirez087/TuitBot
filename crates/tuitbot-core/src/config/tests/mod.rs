//! Config test suite split by domain.
//!
//! - `defaults`   — load, default values, roundtrip, basic validation
//! - `sources`    — BusinessProfile, draft_context, content sources, Google Drive
//! - `deployment` — deployment mode, capabilities, preferred source, connection_id, connector
//! - `migrations` — backward-compat regression, content source enabled/change_detection
//! - `validation` — validate_minimum() and validate() edge cases
//! - `env_types`  — env override paths, serde/Default/method coverage

mod defaults;
mod deployment;
mod env_types;
mod migrations;
mod sources;
mod validation;

// Shared fixtures and helpers available to all submodules.

use super::env_overrides::{parse_env_bool, parse_env_u32};
use super::*;
use crate::config::types::{DeploymentCapabilities, DeploymentMode};
use std::env;
use std::ffi::OsString;
use std::sync::{Mutex, OnceLock};

// Environment variables are process-global — tests that mutate them must not run concurrently.
pub(super) fn with_locked_env(test: impl FnOnce()) {
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let _guard = ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("env lock poisoned");
    test();
}

/// Create a minimal config that passes all validation checks.
pub(super) fn valid_test_config() -> Config {
    let mut config = Config::default();
    config.business.product_name = "TestProduct".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.business.product_description = "A test product".to_string();
    config.business.industry_topics = vec!["testing".to_string()];
    config.llm.provider = "ollama".to_string();
    config.x_api.client_id = "test-id".to_string();
    config
}

pub(super) struct ScopedEnvVar {
    key: &'static str,
    previous: Option<OsString>,
}

impl ScopedEnvVar {
    pub fn set(key: &'static str, value: &str) -> Self {
        let previous = env::var_os(key);
        env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for ScopedEnvVar {
    fn drop(&mut self) {
        match self.previous.take() {
            Some(previous) => env::set_var(self.key, previous),
            None => env::remove_var(self.key),
        }
    }
}
