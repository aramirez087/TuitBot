//! CRUD operations for note_tags (normalized tag storage).

use crate::error::StorageError;
use crate::storage::DbPool;

/// Tag source kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagSource {
    Frontmatter,
    Inline,
}

impl TagSource {
    fn as_str(self) -> &'static str {
        match self {
            Self::Frontmatter => "frontmatter",
            Self::Inline => "inline",
        }
    }
}

/// A normalized tag ready for storage.
#[derive(Debug, Clone)]
pub struct NormalizedTag {
    pub tag_text: String,
    pub source: TagSource,
}

/// A stored note tag row.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NoteTag {
    pub id: i64,
    pub account_id: String,
    pub node_id: i64,
    pub tag_text: String,
    pub source: String,
}

/// Row type for sqlx tuple decoding.
type NoteTagRow = (i64, String, i64, String, String);

impl NoteTag {
    fn from_row(r: NoteTagRow) -> Self {
        Self {
            id: r.0,
            account_id: r.1,
            node_id: r.2,
            tag_text: r.3,
            source: r.4,
        }
    }
}

// ============================================================================
// Delete
// ============================================================================

/// Delete all tags for a node. Idempotent.
pub async fn delete_tags_for_node(
    pool: &DbPool,
    account_id: &str,
    node_id: i64,
) -> Result<u64, StorageError> {
    let result = sqlx::query("DELETE FROM note_tags WHERE account_id = ? AND node_id = ?")
        .bind(account_id)
        .bind(node_id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.rows_affected())
}

// ============================================================================
// Insert
// ============================================================================

/// Insert multiple tags for a node, ignoring duplicates (UNIQUE constraint).
pub async fn insert_tags(
    pool: &DbPool,
    account_id: &str,
    node_id: i64,
    tags: &[NormalizedTag],
) -> Result<(), StorageError> {
    for tag in tags {
        sqlx::query(
            "INSERT OR IGNORE INTO note_tags \
             (account_id, node_id, tag_text, source) \
             VALUES (?, ?, ?, ?)",
        )
        .bind(account_id)
        .bind(node_id)
        .bind(&tag.tag_text)
        .bind(tag.source.as_str())
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    }
    Ok(())
}

// ============================================================================
// Query
// ============================================================================

/// Get all tags for a node.
pub async fn get_tags_for_node(
    pool: &DbPool,
    account_id: &str,
    node_id: i64,
) -> Result<Vec<NoteTag>, StorageError> {
    let rows: Vec<NoteTagRow> = sqlx::query_as(
        "SELECT id, account_id, node_id, tag_text, source \
         FROM note_tags \
         WHERE account_id = ? AND node_id = ? \
         ORDER BY tag_text",
    )
    .bind(account_id)
    .bind(node_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(NoteTag::from_row).collect())
}

/// Find nodes sharing tags with the given node, ranked by shared tag count.
///
/// Returns `(target_node_id, shared_tag_text)` pairs, capped at `max_results`.
pub async fn find_shared_tag_neighbors(
    pool: &DbPool,
    account_id: &str,
    node_id: i64,
    max_results: u32,
) -> Result<Vec<(i64, String)>, StorageError> {
    let rows: Vec<(i64, String)> = sqlx::query_as(
        "SELECT nt2.node_id, nt1.tag_text \
         FROM note_tags nt1 \
         JOIN note_tags nt2 \
           ON nt1.account_id = nt2.account_id \
          AND nt1.tag_text = nt2.tag_text \
          AND nt1.node_id != nt2.node_id \
         WHERE nt1.account_id = ? AND nt1.node_id = ? \
         GROUP BY nt2.node_id \
         ORDER BY COUNT(*) DESC \
         LIMIT ?",
    )
    .bind(account_id)
    .bind(node_id)
    .bind(max_results)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows)
}
