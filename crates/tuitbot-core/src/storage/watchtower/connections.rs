//! CRUD operations for the `connections` table.
//!
//! Stores remote sync connections (Google Drive, future: OneDrive, etc.)
//! with their non-secret metadata. The `encrypted_credentials` column is
//! intentionally excluded from the `Connection` struct and all queries in
//! this module to prevent accidental secret leakage via API responses.

use super::super::accounts::DEFAULT_ACCOUNT_ID;
use super::super::DbPool;
use crate::error::StorageError;

/// Row type for connections queries.
///
/// **Security:** This struct intentionally omits `encrypted_credentials`.
/// Only the connector module (session 03) reads credentials via a separate
/// dedicated function that requires explicit opt-in.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Connection {
    pub id: i64,
    pub account_id: String,
    pub connector_type: String,
    pub account_email: Option<String>,
    pub display_name: Option<String>,
    pub status: String,
    pub metadata_json: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Row type for connection queries (tuple form for sqlx).
type ConnectionRow = (
    i64,
    String,
    String,
    Option<String>,
    Option<String>,
    String,
    String,
    String,
    String,
);

fn row_to_connection(r: ConnectionRow) -> Connection {
    Connection {
        id: r.0,
        account_id: r.1,
        connector_type: r.2,
        account_email: r.3,
        display_name: r.4,
        status: r.5,
        metadata_json: r.6,
        created_at: r.7,
        updated_at: r.8,
    }
}

/// Insert a new connection (without credentials -- session 03 adds encryption).
///
/// Returns the auto-generated connection ID.
pub async fn insert_connection(
    pool: &DbPool,
    connector_type: &str,
    account_email: Option<&str>,
    display_name: Option<&str>,
) -> Result<i64, StorageError> {
    let row: (i64,) = sqlx::query_as(
        "INSERT INTO connections (account_id, connector_type, account_email, display_name) \
         VALUES (?, ?, ?, ?) \
         RETURNING id",
    )
    .bind(DEFAULT_ACCOUNT_ID)
    .bind(connector_type)
    .bind(account_email)
    .bind(display_name)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

/// Get a connection by ID (never returns encrypted_credentials).
pub async fn get_connection(pool: &DbPool, id: i64) -> Result<Option<Connection>, StorageError> {
    let row: Option<ConnectionRow> = sqlx::query_as(
        "SELECT id, account_id, connector_type, account_email, display_name, \
                status, metadata_json, created_at, updated_at \
         FROM connections WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(row_to_connection))
}

/// List active connections (never returns encrypted_credentials).
pub async fn get_connections(pool: &DbPool) -> Result<Vec<Connection>, StorageError> {
    let rows: Vec<ConnectionRow> = sqlx::query_as(
        "SELECT id, account_id, connector_type, account_email, display_name, \
                status, metadata_json, created_at, updated_at \
         FROM connections WHERE status = 'active' ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(row_to_connection).collect())
}

/// Update the status of a connection.
pub async fn update_connection_status(
    pool: &DbPool,
    id: i64,
    status: &str,
) -> Result<(), StorageError> {
    sqlx::query("UPDATE connections SET status = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Delete a connection by ID.
pub async fn delete_connection(pool: &DbPool, id: i64) -> Result<(), StorageError> {
    sqlx::query("DELETE FROM connections WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Get active connections filtered by connector_type.
pub async fn get_connections_by_type(
    pool: &DbPool,
    connector_type: &str,
) -> Result<Vec<Connection>, StorageError> {
    let rows: Vec<ConnectionRow> = sqlx::query_as(
        "SELECT id, account_id, connector_type, account_email, display_name, \
                status, metadata_json, created_at, updated_at \
         FROM connections WHERE connector_type = ? AND status = 'active' ORDER BY id",
    )
    .bind(connector_type)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(row_to_connection).collect())
}

/// Store encrypted credentials for a connection (explicit opt-in).
///
/// This is the only function that writes to the `encrypted_credentials`
/// column. Separated from `insert_connection` to make credential storage
/// an intentional, auditable action.
pub async fn store_encrypted_credentials(
    pool: &DbPool,
    id: i64,
    ciphertext: &[u8],
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE connections SET encrypted_credentials = ?, updated_at = datetime('now') \
         WHERE id = ?",
    )
    .bind(ciphertext)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Read encrypted credentials for a connection (explicit opt-in).
///
/// Returns None if the connection doesn't exist or has no credentials.
pub async fn read_encrypted_credentials(
    pool: &DbPool,
    id: i64,
) -> Result<Option<Vec<u8>>, StorageError> {
    let row: Option<(Option<Vec<u8>>,)> =
        sqlx::query_as("SELECT encrypted_credentials FROM connections WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.and_then(|r| r.0))
}

/// Update the metadata_json of a connection.
pub async fn update_connection_metadata(
    pool: &DbPool,
    id: i64,
    metadata_json: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE connections SET metadata_json = ?, updated_at = datetime('now') WHERE id = ?",
    )
    .bind(metadata_json)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}
