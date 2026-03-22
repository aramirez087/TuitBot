//! CRUD operations for note_edges (graph link storage).

use crate::error::StorageError;
use crate::storage::DbPool;

/// Input for inserting a new edge.
#[derive(Debug, Clone)]
pub struct NewEdge {
    pub source_node_id: i64,
    pub target_node_id: i64,
    pub edge_type: String,
    pub edge_label: Option<String>,
    pub source_chunk_id: Option<i64>,
}

/// A stored note edge row.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NoteEdge {
    pub id: i64,
    pub account_id: String,
    pub source_node_id: i64,
    pub target_node_id: i64,
    pub edge_type: String,
    pub edge_label: Option<String>,
    pub source_chunk_id: Option<i64>,
    pub created_at: String,
}

/// Row type for sqlx tuple decoding.
type NoteEdgeRow = (
    i64,
    String,
    i64,
    i64,
    String,
    Option<String>,
    Option<i64>,
    String,
);

impl NoteEdge {
    fn from_row(r: NoteEdgeRow) -> Self {
        Self {
            id: r.0,
            account_id: r.1,
            source_node_id: r.2,
            target_node_id: r.3,
            edge_type: r.4,
            edge_label: r.5,
            source_chunk_id: r.6,
            created_at: r.7,
        }
    }
}

// ============================================================================
// Delete
// ============================================================================

/// Delete all edges originating from a source node (forward links, shared-tag
/// edges, and backlinks created by this node's forward links).
///
/// Idempotency: safe to call even if no edges exist.
pub async fn delete_edges_for_source(
    pool: &DbPool,
    account_id: &str,
    source_node_id: i64,
) -> Result<u64, StorageError> {
    let result = sqlx::query("DELETE FROM note_edges WHERE account_id = ? AND source_node_id = ?")
        .bind(account_id)
        .bind(source_node_id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.rows_affected())
}

// ============================================================================
// Insert
// ============================================================================

/// Insert a single edge, ignoring duplicates (UNIQUE constraint).
pub async fn insert_edge(
    pool: &DbPool,
    account_id: &str,
    edge: &NewEdge,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT OR IGNORE INTO note_edges \
         (account_id, source_node_id, target_node_id, edge_type, edge_label, source_chunk_id) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(account_id)
    .bind(edge.source_node_id)
    .bind(edge.target_node_id)
    .bind(&edge.edge_type)
    .bind(&edge.edge_label)
    .bind(edge.source_chunk_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Insert multiple edges, ignoring duplicates.
pub async fn insert_edges(
    pool: &DbPool,
    account_id: &str,
    edges: &[NewEdge],
) -> Result<(), StorageError> {
    for edge in edges {
        insert_edge(pool, account_id, edge).await?;
    }
    Ok(())
}

// ============================================================================
// Query
// ============================================================================

/// Get all edges originating from a source node.
pub async fn get_edges_for_source(
    pool: &DbPool,
    account_id: &str,
    source_node_id: i64,
) -> Result<Vec<NoteEdge>, StorageError> {
    let rows: Vec<NoteEdgeRow> = sqlx::query_as(
        "SELECT id, account_id, source_node_id, target_node_id, \
                edge_type, edge_label, source_chunk_id, created_at \
         FROM note_edges \
         WHERE account_id = ? AND source_node_id = ? \
         ORDER BY id",
    )
    .bind(account_id)
    .bind(source_node_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(NoteEdge::from_row).collect())
}

/// Get all edges pointing to a target node (backlinks and shared-tag edges).
pub async fn get_edges_for_target(
    pool: &DbPool,
    account_id: &str,
    target_node_id: i64,
) -> Result<Vec<NoteEdge>, StorageError> {
    let rows: Vec<NoteEdgeRow> = sqlx::query_as(
        "SELECT id, account_id, source_node_id, target_node_id, \
                edge_type, edge_label, source_chunk_id, created_at \
         FROM note_edges \
         WHERE account_id = ? AND target_node_id = ? \
         ORDER BY id",
    )
    .bind(account_id)
    .bind(target_node_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(NoteEdge::from_row).collect())
}
