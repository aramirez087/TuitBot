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

    // -----------------------------------------------------------------------
    // Additional provenance coverage tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn get_links_for_nonexistent_entity() {
        let pool = init_test_db().await.expect("init db");
        let account_id = "00000000-0000-0000-0000-000000000000";

        let links = get_links_for(&pool, account_id, "approval_queue", 9999)
            .await
            .expect("get");
        assert!(links.is_empty());
    }

    #[tokio::test]
    async fn delete_links_for_nonexistent() {
        let pool = init_test_db().await.expect("init db");
        let account_id = "00000000-0000-0000-0000-000000000000";

        let deleted = delete_links_for(&pool, account_id, "approval_queue", 9999)
            .await
            .expect("delete");
        assert_eq!(deleted, 0);
    }

    #[tokio::test]
    async fn copy_links_for_nonexistent_source() {
        let pool = init_test_db().await.expect("init db");
        let account_id = "00000000-0000-0000-0000-000000000000";

        let copied = copy_links_for(
            &pool,
            account_id,
            "approval_queue",
            9999,
            "original_tweet",
            1,
        )
        .await
        .expect("copy");
        assert_eq!(copied, 0);
    }

    #[tokio::test]
    async fn insert_links_with_source_and_snippet() {
        let pool = init_test_db().await.expect("init db");
        let account_id = "00000000-0000-0000-0000-000000000000";

        let refs = vec![ProvenanceRef {
            node_id: None,
            chunk_id: None,
            seed_id: None,
            source_path: Some("notes/full.md".to_string()),
            heading_path: Some("# Full > ## Path".to_string()),
            snippet: Some("Full snippet text".to_string()),
        }];

        insert_links_for(&pool, account_id, "scheduled_content", 100, &refs)
            .await
            .expect("insert");

        let links = get_links_for(&pool, account_id, "scheduled_content", 100)
            .await
            .expect("get");

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].source_path.as_deref(), Some("notes/full.md"));
        assert_eq!(links[0].heading_path.as_deref(), Some("# Full > ## Path"));
        assert_eq!(links[0].snippet.as_deref(), Some("Full snippet text"));
        assert_eq!(links[0].entity_type, "scheduled_content");
        assert_eq!(links[0].entity_id, 100);
    }

    #[tokio::test]
    async fn insert_links_with_no_optional_fields() {
        let pool = init_test_db().await.expect("init db");
        let account_id = "00000000-0000-0000-0000-000000000000";

        let refs = vec![ProvenanceRef {
            node_id: None,
            chunk_id: None,
            seed_id: None,
            source_path: None,
            heading_path: None,
            snippet: None,
        }];

        insert_links_for(&pool, account_id, "thread", 50, &refs)
            .await
            .expect("insert");

        let links = get_links_for(&pool, account_id, "thread", 50)
            .await
            .expect("get");

        assert_eq!(links.len(), 1);
        assert!(links[0].node_id.is_none());
        assert!(links[0].source_path.is_none());
        assert!(links[0].snippet.is_none());
    }

    #[tokio::test]
    async fn multiple_entities_independent() {
        let pool = init_test_db().await.expect("init db");
        let account_id = "00000000-0000-0000-0000-000000000000";

        let refs_a = vec![ProvenanceRef {
            node_id: None,
            chunk_id: None,
            seed_id: None,
            source_path: Some("a.md".to_string()),
            heading_path: None,
            snippet: None,
        }];

        let refs_b = vec![ProvenanceRef {
            node_id: None,
            chunk_id: None,
            seed_id: None,
            source_path: Some("b.md".to_string()),
            heading_path: None,
            snippet: None,
        }];

        insert_links_for(&pool, account_id, "approval_queue", 1, &refs_a)
            .await
            .expect("insert a");
        insert_links_for(&pool, account_id, "approval_queue", 2, &refs_b)
            .await
            .expect("insert b");

        let links_a = get_links_for(&pool, account_id, "approval_queue", 1)
            .await
            .expect("get a");
        let links_b = get_links_for(&pool, account_id, "approval_queue", 2)
            .await
            .expect("get b");

        assert_eq!(links_a.len(), 1);
        assert_eq!(links_b.len(), 1);
        assert_eq!(links_a[0].source_path.as_deref(), Some("a.md"));
        assert_eq!(links_b[0].source_path.as_deref(), Some("b.md"));
    }

    #[tokio::test]
    async fn delete_only_target_entity() {
        let pool = init_test_db().await.expect("init db");
        let account_id = "00000000-0000-0000-0000-000000000000";

        insert_links_for(&pool, account_id, "approval_queue", 1, &sample_refs())
            .await
            .expect("insert 1");
        insert_links_for(&pool, account_id, "approval_queue", 2, &sample_refs())
            .await
            .expect("insert 2");

        delete_links_for(&pool, account_id, "approval_queue", 1)
            .await
            .expect("delete");

        // Entity 1 should be gone, entity 2 should remain
        let links_1 = get_links_for(&pool, account_id, "approval_queue", 1)
            .await
            .expect("get 1");
        let links_2 = get_links_for(&pool, account_id, "approval_queue", 2)
            .await
            .expect("get 2");

        assert!(links_1.is_empty());
        assert_eq!(links_2.len(), 2);
    }

    #[test]
    fn provenance_ref_serde_roundtrip() {
        let pref = ProvenanceRef {
            node_id: Some(1),
            chunk_id: None,
            seed_id: Some(5),
            source_path: Some("test.md".to_string()),
            heading_path: None,
            snippet: Some("hello".to_string()),
        };

        let json = serde_json::to_string(&pref).expect("serialize");
        let deserialized: ProvenanceRef = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.node_id, Some(1));
        assert_eq!(deserialized.chunk_id, None);
        assert_eq!(deserialized.seed_id, Some(5));
    }

    #[test]
    fn provenance_ref_deserialize_defaults() {
        // Empty JSON object should deserialize with all None
        let pref: ProvenanceRef = serde_json::from_str("{}").expect("deserialize");
        assert!(pref.node_id.is_none());
        assert!(pref.chunk_id.is_none());
        assert!(pref.seed_id.is_none());
        assert!(pref.source_path.is_none());
        assert!(pref.heading_path.is_none());
        assert!(pref.snippet.is_none());
    }
}
