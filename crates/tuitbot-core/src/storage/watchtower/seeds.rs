//! CRUD operations for draft_seeds.

use super::{DraftSeed, DraftSeedRow, SeedWithContext};
use crate::error::StorageError;
use crate::storage::accounts::DEFAULT_ACCOUNT_ID;
use crate::storage::DbPool;

// ============================================================================
// Account-scoped draft seed functions
// ============================================================================

/// Insert a new draft seed for a specific account and return its ID.
pub async fn insert_draft_seed_for(
    pool: &DbPool,
    account_id: &str,
    node_id: i64,
    seed_text: &str,
    archetype_suggestion: Option<&str>,
    chunk_id: Option<i64>,
) -> Result<i64, StorageError> {
    let row: (i64,) = sqlx::query_as(
        "INSERT INTO draft_seeds (account_id, node_id, seed_text, archetype_suggestion, chunk_id) \
         VALUES (?, ?, ?, ?, ?) \
         RETURNING id",
    )
    .bind(account_id)
    .bind(node_id)
    .bind(seed_text)
    .bind(archetype_suggestion)
    .bind(chunk_id)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

/// Insert a new draft seed and return its ID.
pub async fn insert_draft_seed(
    pool: &DbPool,
    node_id: i64,
    seed_text: &str,
    archetype_suggestion: Option<&str>,
) -> Result<i64, StorageError> {
    insert_draft_seed_for(
        pool,
        DEFAULT_ACCOUNT_ID,
        node_id,
        seed_text,
        archetype_suggestion,
        None,
    )
    .await
}

/// Insert a draft seed with an explicit engagement weight for a specific account.
pub async fn insert_draft_seed_with_weight_for(
    pool: &DbPool,
    account_id: &str,
    node_id: i64,
    seed_text: &str,
    archetype_suggestion: Option<&str>,
    weight: f64,
    chunk_id: Option<i64>,
) -> Result<i64, StorageError> {
    let row: (i64,) = sqlx::query_as(
        "INSERT INTO draft_seeds \
         (account_id, node_id, seed_text, archetype_suggestion, engagement_weight, chunk_id) \
         VALUES (?, ?, ?, ?, ?, ?) \
         RETURNING id",
    )
    .bind(account_id)
    .bind(node_id)
    .bind(seed_text)
    .bind(archetype_suggestion)
    .bind(weight)
    .bind(chunk_id)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

/// Insert a draft seed with an explicit engagement weight.
pub async fn insert_draft_seed_with_weight(
    pool: &DbPool,
    node_id: i64,
    seed_text: &str,
    archetype_suggestion: Option<&str>,
    weight: f64,
) -> Result<i64, StorageError> {
    insert_draft_seed_with_weight_for(
        pool,
        DEFAULT_ACCOUNT_ID,
        node_id,
        seed_text,
        archetype_suggestion,
        weight,
        None,
    )
    .await
}

/// Get pending draft seeds for a specific account, ordered by engagement weight descending.
pub async fn get_pending_seeds_for(
    pool: &DbPool,
    account_id: &str,
    limit: u32,
) -> Result<Vec<DraftSeed>, StorageError> {
    let rows: Vec<DraftSeedRow> = sqlx::query_as(
        "SELECT id, account_id, node_id, seed_text, archetype_suggestion, \
                    engagement_weight, status, created_at, used_at, chunk_id \
             FROM draft_seeds \
             WHERE account_id = ? AND status = 'pending' \
             ORDER BY engagement_weight DESC \
             LIMIT ?",
    )
    .bind(account_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(DraftSeed::from_row).collect())
}

/// Get pending draft seeds ordered by engagement weight descending.
pub async fn get_pending_seeds(pool: &DbPool, limit: u32) -> Result<Vec<DraftSeed>, StorageError> {
    get_pending_seeds_for(pool, DEFAULT_ACCOUNT_ID, limit).await
}

/// Mark a draft seed as used.
pub async fn mark_seed_used(pool: &DbPool, id: i64) -> Result<(), StorageError> {
    sqlx::query("UPDATE draft_seeds SET status = 'used', used_at = datetime('now') WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Retrieve draft seeds for context for a specific account.
pub async fn get_seeds_for_context_for(
    pool: &DbPool,
    account_id: &str,
    limit: u32,
) -> Result<Vec<SeedWithContext>, StorageError> {
    let rows: Vec<(String, Option<String>, Option<String>, f64)> = sqlx::query_as(
        "SELECT ds.seed_text, cn.title, ds.archetype_suggestion, ds.engagement_weight \
         FROM draft_seeds ds \
         JOIN content_nodes cn ON cn.id = ds.node_id \
         WHERE ds.account_id = ? AND ds.status = 'pending' \
         ORDER BY ds.engagement_weight DESC \
         LIMIT ?",
    )
    .bind(account_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(|r| SeedWithContext {
            seed_text: r.0,
            source_title: r.1,
            archetype_suggestion: r.2,
            engagement_weight: r.3,
        })
        .collect())
}

/// Retrieve draft seeds suitable for cold-start context injection.
pub async fn get_seeds_for_context(
    pool: &DbPool,
    limit: u32,
) -> Result<Vec<SeedWithContext>, StorageError> {
    get_seeds_for_context_for(pool, DEFAULT_ACCOUNT_ID, limit).await
}
