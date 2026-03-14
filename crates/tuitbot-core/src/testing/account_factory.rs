//! Factory for building [`Account`] instances with realistic test data.
//!
//! # Example
//! ```rust
//! use tuitbot_core::testing::AccountFactory;
//!
//! let account = AccountFactory::new().build();
//! let multi = AccountFactory::new()
//!     .with_label("brand-account")
//!     .with_username("mybrand")
//!     .build();
//! let accounts = AccountFactory::build_many(3);
//! ```

use crate::storage::accounts::Account;

static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn next_id() -> String {
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!("test-account-{n:04}")
}

/// Builder for [`Account`] test instances.
pub struct AccountFactory {
    id: Option<String>,
    label: String,
    x_user_id: Option<String>,
    x_username: Option<String>,
    x_display_name: Option<String>,
    x_avatar_url: Option<String>,
    status: String,
}

impl Default for AccountFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl AccountFactory {
    pub fn new() -> Self {
        Self {
            id: None,
            label: "test-account".to_string(),
            x_user_id: Some("987654321".to_string()),
            x_username: Some("test_user".to_string()),
            x_display_name: Some("Test User".to_string()),
            x_avatar_url: None,
            status: "active".to_string(),
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.x_username = Some(username.into());
        self
    }

    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.x_display_name = Some(name.into());
        self
    }

    pub fn with_avatar_url(mut self, url: impl Into<String>) -> Self {
        self.x_avatar_url = Some(url.into());
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    pub fn inactive(mut self) -> Self {
        self.status = "inactive".to_string();
        self
    }

    pub fn unauthenticated(mut self) -> Self {
        self.x_user_id = None;
        self.x_username = None;
        self.x_display_name = None;
        self.status = "pending_auth".to_string();
        self
    }

    pub fn build(self) -> Account {
        let now = "2026-03-14T00:00:00.000Z".to_string();
        Account {
            id: self.id.unwrap_or_else(next_id),
            label: self.label,
            x_user_id: self.x_user_id,
            x_username: self.x_username,
            x_display_name: self.x_display_name,
            x_avatar_url: self.x_avatar_url,
            config_overrides: "{}".to_string(),
            token_path: None,
            status: self.status,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn build_many(count: usize) -> Vec<Account> {
        (0..count)
            .map(|i| {
                AccountFactory::new()
                    .with_label(format!("account-{i}"))
                    .with_username(format!("user_{i}"))
                    .with_display_name(format!("User {i}"))
                    .build()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_account_with_defaults() {
        let acct = AccountFactory::new().build();
        assert!(!acct.id.is_empty());
        assert_eq!(acct.status, "active");
        assert!(acct.x_username.is_some());
    }

    #[test]
    fn inactive_sets_status() {
        let acct = AccountFactory::new().inactive().build();
        assert_eq!(acct.status, "inactive");
    }

    #[test]
    fn unauthenticated_clears_x_fields() {
        let acct = AccountFactory::new().unauthenticated().build();
        assert!(acct.x_user_id.is_none());
        assert!(acct.x_username.is_none());
        assert_eq!(acct.status, "pending_auth");
    }

    #[test]
    fn build_many_produces_unique_ids() {
        let accounts = AccountFactory::build_many(4);
        assert_eq!(accounts.len(), 4);
        let ids: std::collections::HashSet<_> = accounts.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), 4);
    }
}
