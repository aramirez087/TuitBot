//! CRUD operations for the `vault_provenance_links` table.
//!
//! Provides a polymorphic provenance store: any content entity (approval_queue,
//! scheduled_content, original_tweet, thread) can link back to the vault notes
//! and chunks that influenced its generation.

use super::DbPool;
use crate::error::StorageError;

/// A row in the `vault_provenance_links` table.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct ProvenanceLink {
    pub id: i64,
    pub account_id: String,
    pub entity_type: String,
    pub entity_id: i64,
    pub node_id: Option<i64>,
    pub chunk_id: Option<i64>,
    pub seed_id: Option<i64>,
    pub source_path: Option<String>,
    pub heading_path: Option<String>,
    pub snippet: Option<String>,
    pub created_at: String,
}

/// A provenance reference carried through the API layer.
///
/// Derived from `VaultCitation` at creation time; stores snapshot values so
/// provenance survives even if the source note is later deleted.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProvenanceRef {
    #[serde(default)]
    pub node_id: Option<i64>,
    #[serde(default)]
    pub chunk_id: Option<i64>,
    #[serde(default)]
    pub seed_id: Option<i64>,
    #[serde(default)]
    pub source_path: Option<String>,
    #[serde(default)]
    pub heading_path: Option<String>,
    #[serde(default)]
    pub snippet: Option<String>,
}

/// Insert provenance links for a content entity.
///
/// Each `ProvenanceRef` becomes one row in `vault_provenance_links`.
/// Empty refs slice is a no-op.
pub async fn insert_links_for(
    pool: &DbPool,
    account_id: &str,
    entity_type: &str,
    entity_id: i64,
    refs: &[ProvenanceRef],
) -> Result<(), StorageError> {
    if refs.is_empty() {
        return Ok(());
    }

    for r in refs {
        sqlx::query(
            "INSERT INTO vault_provenance_links \
             (account_id, entity_type, entity_id, node_id, chunk_id, seed_id, \
              source_path, heading_path, snippet) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(account_id)
        .bind(entity_type)
        .bind(entity_id)
        .bind(r.node_id)
        .bind(r.chunk_id)
        .bind(r.seed_id)
        .bind(&r.source_path)
        .bind(&r.heading_path)
        .bind(&r.snippet)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    }

    Ok(())
}

/// Retrieve all provenance links for a content entity.
pub async fn get_links_for(
    pool: &DbPool,
    account_id: &str,
    entity_type: &str,
    entity_id: i64,
) -> Result<Vec<ProvenanceLink>, StorageError> {
    sqlx::query_as::<_, ProvenanceLink>(
        "SELECT * FROM vault_provenance_links \
         WHERE account_id = ? AND entity_type = ? AND entity_id = ? \
         ORDER BY id ASC",
    )
    .bind(account_id)
    .bind(entity_type)
    .bind(entity_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// Copy provenance links from one entity to another.
///
/// Used when an approval_queue item is posted and we want the resulting
/// original_tweet to inherit the same provenance links.
pub async fn copy_links_for(
    pool: &DbPool,
    account_id: &str,
    from_type: &str,
    from_id: i64,
    to_type: &str,
    to_id: i64,
) -> Result<u64, StorageError> {
    let result = sqlx::query(
        "INSERT INTO vault_provenance_links \
         (account_id, entity_type, entity_id, node_id, chunk_id, seed_id, \
          source_path, heading_path, snippet) \
         SELECT ?, ?, ?, node_id, chunk_id, seed_id, source_path, heading_path, snippet \
         FROM vault_provenance_links \
         WHERE account_id = ? AND entity_type = ? AND entity_id = ?",
    )
    .bind(account_id)
    .bind(to_type)
    .bind(to_id)
    .bind(account_id)
    .bind(from_type)
    .bind(from_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.rows_affected())
}

/// Delete all provenance links for a content entity.
pub async fn delete_links_for(
    pool: &DbPool,
    account_id: &str,
    entity_type: &str,
    entity_id: i64,
) -> Result<u64, StorageError> {
    let result = sqlx::query(
        "DELETE FROM vault_provenance_links \
         WHERE account_id = ? AND entity_type = ? AND entity_id = ?",
    )
    .bind(account_id)
    .bind(entity_type)
    .bind(entity_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    fn sample_refs() -> Vec<ProvenanceRef> {
        vec![
            ProvenanceRef {
                node_id: None,
                chunk_id: None,
                seed_id: None,
                source_path: Some("notes/rust.md".to_string()),
                heading_path: Some("# Rust > ## Async".to_string()),
                snippet: Some("Async patterns in Rust...".to_string()),
            },
            ProvenanceRef {
                node_id: None,
                chunk_id: None,
                seed_id: None,
                source_path: Some("notes/testing.md".to_string()),
                heading_path: None,
                snippet: Some("Testing best practices...".to_string()),
            },
        ]
    }

    #[tokio::test]
    async fn insert_and_get_provenance_links() {
        let pool = init_test_db().await.expect("init db");
        let refs = sample_refs();
        let account_id = "00000000-0000-0000-0000-000000000000";

        insert_links_for(&pool, account_id, "approval_queue", 42, &refs)
            .await
            .expect("insert");

        let links = get_links_for(&pool, account_id, "approval_queue", 42)
            .await
            .expect("get");

        assert_eq!(links.len(), 2);
        assert_eq!(links[0].source_path.as_deref(), Some("notes/rust.md"));
        assert_eq!(links[0].heading_path.as_deref(), Some("# Rust > ## Async"));
        assert_eq!(links[1].source_path.as_deref(), Some("notes/testing.md"));
    }

    #[tokio::test]
    async fn copy_links_between_entities() {
        let pool = init_test_db().await.expect("init db");
        let refs = sample_refs();
        let account_id = "00000000-0000-0000-0000-000000000000";

        insert_links_for(&pool, account_id, "approval_queue", 42, &refs)
            .await
            .expect("insert");

        let copied = copy_links_for(
            &pool,
            account_id,
            "approval_queue",
            42,
            "original_tweet",
            99,
        )
        .await
        .expect("copy");

        assert_eq!(copied, 2);

        let links = get_links_for(&pool, account_id, "original_tweet", 99)
            .await
            .expect("get");

        assert_eq!(links.len(), 2);
        assert_eq!(links[0].entity_type, "original_tweet");
        assert_eq!(links[0].entity_id, 99);
    }

    #[tokio::test]
    async fn delete_links() {
        let pool = init_test_db().await.expect("init db");
        let refs = sample_refs();
        let account_id = "00000000-0000-0000-0000-000000000000";

        insert_links_for(&pool, account_id, "approval_queue", 42, &refs)
            .await
            .expect("insert");

        let deleted = delete_links_for(&pool, account_id, "approval_queue", 42)
            .await
            .expect("delete");

        assert_eq!(deleted, 2);

        let links = get_links_for(&pool, account_id, "approval_queue", 42)
            .await
            .expect("get");

        assert!(links.is_empty());
    }

    #[tokio::test]
    async fn empty_provenance_is_noop() {
        let pool = init_test_db().await.expect("init db");
        let account_id = "00000000-0000-0000-0000-000000000000";

        insert_links_for(&pool, account_id, "approval_queue", 42, &[])
            .await
            .expect("insert empty");

        let links = get_links_for(&pool, account_id, "approval_queue", 42)
            .await
            .expect("get");

        assert!(links.is_empty());
    }
}
