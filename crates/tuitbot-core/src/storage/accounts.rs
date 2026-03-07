//! CRUD operations for the `accounts` and `account_roles` tables.
//!
//! Provides multi-account registry, per-account configuration overrides,
//! and role-based access control (admin/approver/viewer).

use std::path::{Path, PathBuf};

use super::DbPool;
use crate::error::StorageError;

/// Well-known sentinel ID for the default (backward-compatible) account.
pub const DEFAULT_ACCOUNT_ID: &str = "00000000-0000-0000-0000-000000000000";

/// An account in the registry.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct Account {
    pub id: String,
    pub label: String,
    pub x_user_id: Option<String>,
    pub x_username: Option<String>,
    pub x_display_name: Option<String>,
    pub x_avatar_url: Option<String>,
    pub config_overrides: String,
    pub token_path: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// A role assignment for an actor on an account.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct AccountRole {
    pub account_id: String,
    pub actor: String,
    pub role: String,
    pub created_at: String,
}

/// Ensure the default account and its roles exist.
///
/// This is idempotent — safe to call on every startup. It re-creates the
/// default account row and admin roles if they were deleted (e.g. by
/// factory reset).
pub async fn ensure_default_account(pool: &DbPool) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT OR IGNORE INTO accounts (id, label, status) \
         VALUES (?, 'Default', 'active')",
    )
    .bind(DEFAULT_ACCOUNT_ID)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    sqlx::query(
        "INSERT OR IGNORE INTO account_roles (account_id, actor, role) \
         VALUES (?, 'dashboard', 'admin')",
    )
    .bind(DEFAULT_ACCOUNT_ID)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    sqlx::query(
        "INSERT OR IGNORE INTO account_roles (account_id, actor, role) \
         VALUES (?, 'mcp', 'admin')",
    )
    .bind(DEFAULT_ACCOUNT_ID)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// List all active accounts, ordered by creation date.
pub async fn list_accounts(pool: &DbPool) -> Result<Vec<Account>, StorageError> {
    sqlx::query_as::<_, Account>(
        "SELECT * FROM accounts WHERE status = 'active' ORDER BY created_at",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// Get a single account by ID.
pub async fn get_account(pool: &DbPool, id: &str) -> Result<Option<Account>, StorageError> {
    sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })
}

/// Default config overrides for new accounts.
///
/// Blanks out identity fields so a new account doesn't inherit the default
/// account's persona, business profile, or targets from `config.toml`.
const NEW_ACCOUNT_OVERRIDES: &str = r#"{
    "business": {
        "product_name": "",
        "product_keywords": [],
        "product_description": "",
        "product_url": null,
        "target_audience": "",
        "competitor_keywords": [],
        "industry_topics": [],
        "brand_voice": null,
        "reply_style": null,
        "content_style": null,
        "persona_opinions": [],
        "persona_experiences": [],
        "content_pillars": []
    },
    "targets": []
}"#;

/// Create a new account. Returns the account ID.
pub async fn create_account(pool: &DbPool, id: &str, label: &str) -> Result<String, StorageError> {
    sqlx::query(
        "INSERT INTO accounts (id, label, config_overrides, updated_at) \
         VALUES (?, ?, ?, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
    )
    .bind(id)
    .bind(label)
    .bind(NEW_ACCOUNT_OVERRIDES)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    // Auto-grant admin to dashboard actor for new accounts.
    set_role(pool, id, "dashboard", "admin").await?;

    Ok(id.to_string())
}

/// Parameters for updating an account's mutable fields.
#[derive(Debug, Default)]
pub struct UpdateAccountParams<'a> {
    pub label: Option<&'a str>,
    pub x_user_id: Option<&'a str>,
    pub x_username: Option<&'a str>,
    pub x_display_name: Option<&'a str>,
    pub x_avatar_url: Option<&'a str>,
    pub config_overrides: Option<&'a str>,
    pub token_path: Option<&'a str>,
    pub status: Option<&'a str>,
}

/// Update an account's mutable fields.
pub async fn update_account(
    pool: &DbPool,
    id: &str,
    params: UpdateAccountParams<'_>,
) -> Result<(), StorageError> {
    // Build SET clauses dynamically to only update provided fields.
    let mut sets = vec!["updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')".to_string()];
    let mut binds: Vec<String> = Vec::new();

    if let Some(v) = params.label {
        sets.push(format!("label = ?{}", binds.len() + 1));
        binds.push(v.to_string());
    }
    if let Some(v) = params.x_user_id {
        sets.push(format!("x_user_id = ?{}", binds.len() + 1));
        binds.push(v.to_string());
    }
    if let Some(v) = params.x_username {
        sets.push(format!("x_username = ?{}", binds.len() + 1));
        binds.push(v.to_string());
    }
    if let Some(v) = params.x_display_name {
        sets.push(format!("x_display_name = ?{}", binds.len() + 1));
        binds.push(v.to_string());
    }
    if let Some(v) = params.x_avatar_url {
        sets.push(format!("x_avatar_url = ?{}", binds.len() + 1));
        binds.push(v.to_string());
    }
    if let Some(v) = params.config_overrides {
        sets.push(format!("config_overrides = ?{}", binds.len() + 1));
        binds.push(v.to_string());
    }
    if let Some(v) = params.token_path {
        sets.push(format!("token_path = ?{}", binds.len() + 1));
        binds.push(v.to_string());
    }
    if let Some(v) = params.status {
        sets.push(format!("status = ?{}", binds.len() + 1));
        binds.push(v.to_string());
    }

    let id_param = binds.len() + 1;
    let sql = format!(
        "UPDATE accounts SET {} WHERE id = ?{}",
        sets.join(", "),
        id_param
    );

    let mut query = sqlx::query(&sql);
    for b in &binds {
        query = query.bind(b);
    }
    query = query.bind(id);

    query
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Archive (soft-delete) an account by setting status to 'archived'.
pub async fn delete_account(pool: &DbPool, id: &str) -> Result<(), StorageError> {
    if id == DEFAULT_ACCOUNT_ID {
        return Err(StorageError::Query {
            source: sqlx::Error::Protocol("cannot delete the default account".into()),
        });
    }

    sqlx::query(
        "UPDATE accounts SET status = 'archived', \
         updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?",
    )
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Check whether an account exists and is active.
pub async fn account_exists(pool: &DbPool, id: &str) -> Result<bool, StorageError> {
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT COUNT(*) FROM accounts WHERE id = ? AND status = 'active'")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(|(c,)| c > 0).unwrap_or(false))
}

// ---- Role management ----

/// Get the role for an actor on an account.
/// Returns `None` if no role is assigned.
pub async fn get_role(
    pool: &DbPool,
    account_id: &str,
    actor: &str,
) -> Result<Option<String>, StorageError> {
    // Default account grants admin to all actors for backward compat.
    if account_id == DEFAULT_ACCOUNT_ID {
        return Ok(Some("admin".to_string()));
    }

    let row: Option<(String,)> =
        sqlx::query_as("SELECT role FROM account_roles WHERE account_id = ? AND actor = ?")
            .bind(account_id)
            .bind(actor)
            .fetch_optional(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(|(r,)| r))
}

/// Set (upsert) a role for an actor on an account.
pub async fn set_role(
    pool: &DbPool,
    account_id: &str,
    actor: &str,
    role: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO account_roles (account_id, actor, role) VALUES (?, ?, ?) \
         ON CONFLICT(account_id, actor) DO UPDATE SET role = excluded.role",
    )
    .bind(account_id)
    .bind(actor)
    .bind(role)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Remove a role assignment.
pub async fn remove_role(pool: &DbPool, account_id: &str, actor: &str) -> Result<(), StorageError> {
    sqlx::query("DELETE FROM account_roles WHERE account_id = ? AND actor = ?")
        .bind(account_id)
        .bind(actor)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// List all roles for an account.
pub async fn list_roles(pool: &DbPool, account_id: &str) -> Result<Vec<AccountRole>, StorageError> {
    sqlx::query_as::<_, AccountRole>(
        "SELECT * FROM account_roles WHERE account_id = ? ORDER BY actor",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

// ---- Path resolution helpers ----

/// Resolve the data directory for an account.
///
/// - Default account: returns `data_dir` itself (root-level, backward compat).
/// - Other accounts: returns `data_dir/accounts/{account_id}/`.
pub fn account_data_dir(data_dir: &Path, account_id: &str) -> PathBuf {
    if account_id == DEFAULT_ACCOUNT_ID {
        data_dir.to_path_buf()
    } else {
        data_dir.join("accounts").join(account_id)
    }
}

/// Resolve the scraper session file path for an account.
pub fn account_scraper_session_path(data_dir: &Path, account_id: &str) -> PathBuf {
    account_data_dir(data_dir, account_id).join("scraper_session.json")
}

/// Resolve the token file path for an account.
pub fn account_token_path(data_dir: &Path, account_id: &str) -> PathBuf {
    account_data_dir(data_dir, account_id).join("tokens.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    #[tokio::test]
    async fn default_account_seeded() {
        let pool = init_test_db().await.expect("init db");
        let account = get_account(&pool, DEFAULT_ACCOUNT_ID)
            .await
            .expect("get")
            .expect("should exist");
        assert_eq!(account.label, "Default");
        assert_eq!(account.status, "active");
    }

    #[tokio::test]
    async fn create_and_list_accounts() {
        let pool = init_test_db().await.expect("init db");
        let id = uuid::Uuid::new_v4().to_string();
        create_account(&pool, &id, "Test Account")
            .await
            .expect("create");

        let accounts = list_accounts(&pool).await.expect("list");
        assert!(accounts.iter().any(|a| a.id == id));
    }

    #[tokio::test]
    async fn update_account_fields() {
        let pool = init_test_db().await.expect("init db");
        let id = uuid::Uuid::new_v4().to_string();
        create_account(&pool, &id, "Original")
            .await
            .expect("create");

        update_account(
            &pool,
            &id,
            UpdateAccountParams {
                label: Some("Updated"),
                x_user_id: Some("12345"),
                x_username: Some("testuser"),
                x_display_name: Some("Test User"),
                x_avatar_url: Some("https://pbs.twimg.com/profile_images/test.jpg"),
                ..Default::default()
            },
        )
        .await
        .expect("update");

        let account = get_account(&pool, &id).await.expect("get").expect("found");
        assert_eq!(account.label, "Updated");
        assert_eq!(account.x_user_id.as_deref(), Some("12345"));
        assert_eq!(account.x_username.as_deref(), Some("testuser"));
        assert_eq!(account.x_display_name.as_deref(), Some("Test User"));
        assert_eq!(
            account.x_avatar_url.as_deref(),
            Some("https://pbs.twimg.com/profile_images/test.jpg")
        );
    }

    #[tokio::test]
    async fn delete_archives_account() {
        let pool = init_test_db().await.expect("init db");
        let id = uuid::Uuid::new_v4().to_string();
        create_account(&pool, &id, "ToDelete")
            .await
            .expect("create");
        delete_account(&pool, &id).await.expect("delete");

        // Archived accounts don't appear in list_accounts (active only)
        let accounts = list_accounts(&pool).await.expect("list");
        assert!(!accounts.iter().any(|a| a.id == id));

        // But still exist in DB
        let account = get_account(&pool, &id).await.expect("get").expect("found");
        assert_eq!(account.status, "archived");
    }

    #[tokio::test]
    async fn cannot_delete_default_account() {
        let pool = init_test_db().await.expect("init db");
        let result = delete_account(&pool, DEFAULT_ACCOUNT_ID).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn default_account_grants_admin_to_all() {
        let pool = init_test_db().await.expect("init db");
        let role = get_role(&pool, DEFAULT_ACCOUNT_ID, "anyone")
            .await
            .expect("get role");
        assert_eq!(role.as_deref(), Some("admin"));
    }

    #[tokio::test]
    async fn role_crud() {
        let pool = init_test_db().await.expect("init db");
        let id = uuid::Uuid::new_v4().to_string();
        create_account(&pool, &id, "RoleTest")
            .await
            .expect("create");

        // New account has dashboard=admin from auto-grant
        let role = get_role(&pool, &id, "dashboard").await.expect("get");
        assert_eq!(role.as_deref(), Some("admin"));

        // Set a viewer role
        set_role(&pool, &id, "mcp", "viewer").await.expect("set");
        let role = get_role(&pool, &id, "mcp").await.expect("get");
        assert_eq!(role.as_deref(), Some("viewer"));

        // Upgrade to approver
        set_role(&pool, &id, "mcp", "approver").await.expect("set");
        let role = get_role(&pool, &id, "mcp").await.expect("get");
        assert_eq!(role.as_deref(), Some("approver"));

        // List roles
        let roles = list_roles(&pool, &id).await.expect("list");
        assert_eq!(roles.len(), 2);

        // Remove role
        remove_role(&pool, &id, "mcp").await.expect("remove");
        let role = get_role(&pool, &id, "mcp").await.expect("get");
        assert!(role.is_none());
    }

    #[tokio::test]
    async fn account_exists_check() {
        let pool = init_test_db().await.expect("init db");
        assert!(account_exists(&pool, DEFAULT_ACCOUNT_ID)
            .await
            .expect("check"));
        assert!(!account_exists(&pool, "nonexistent").await.expect("check"));
    }

    #[test]
    fn account_data_dir_default() {
        let data_dir = Path::new("/home/user/.tuitbot");
        let result = account_data_dir(data_dir, DEFAULT_ACCOUNT_ID);
        assert_eq!(result, PathBuf::from("/home/user/.tuitbot"));
    }

    #[test]
    fn account_data_dir_other() {
        let data_dir = Path::new("/home/user/.tuitbot");
        let id = "abc-123";
        let result = account_data_dir(data_dir, id);
        assert_eq!(
            result,
            PathBuf::from("/home/user/.tuitbot/accounts/abc-123")
        );
    }

    #[test]
    fn scraper_session_path_default() {
        let data_dir = Path::new("/home/user/.tuitbot");
        let result = account_scraper_session_path(data_dir, DEFAULT_ACCOUNT_ID);
        assert_eq!(
            result,
            PathBuf::from("/home/user/.tuitbot/scraper_session.json")
        );
    }

    #[test]
    fn scraper_session_path_other() {
        let data_dir = Path::new("/home/user/.tuitbot");
        let result = account_scraper_session_path(data_dir, "abc-123");
        assert_eq!(
            result,
            PathBuf::from("/home/user/.tuitbot/accounts/abc-123/scraper_session.json")
        );
    }

    #[test]
    fn token_path_default() {
        let data_dir = Path::new("/home/user/.tuitbot");
        let result = account_token_path(data_dir, DEFAULT_ACCOUNT_ID);
        assert_eq!(result, PathBuf::from("/home/user/.tuitbot/tokens.json"));
    }

    #[test]
    fn token_path_other() {
        let data_dir = Path::new("/home/user/.tuitbot");
        let result = account_token_path(data_dir, "abc-123");
        assert_eq!(
            result,
            PathBuf::from("/home/user/.tuitbot/accounts/abc-123/tokens.json")
        );
    }
}
